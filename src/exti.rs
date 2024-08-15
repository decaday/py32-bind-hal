//! External Interrupts (EXTI)


// modified from https://github.com/embassy-rs/embassy
// https://github.com/embassy-rs/embassy/commit/65c085ce910f50903bc5c41ca82eda989810f855
use core::convert::Infallible;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU16, Ordering};
use core::task::{Context, Poll};

extern crate py32csdk_hal_sys as device;
use device::interrupts::interrupt;

use embedded_hal as embedded_hal_1;
use embedded_hal_async;
use embassy_sync::waitqueue::AtomicWaker;

use crate::gpio::{AnyPin, Level, Pull, Speed};
use crate::csdk;


const EXTI_COUNT: usize = 16;
const NEW_AW: AtomicWaker = AtomicWaker::new();
static EXTI_WAKERS: [AtomicWaker; EXTI_COUNT] = [NEW_AW; EXTI_COUNT];

/// Each bit of this flag corresponds to a channel. 
/// 0 means waiting for interrupt, 2 means already awake
static EXTI_POLL_FLAGS: AtomicU16 = AtomicU16::new(0);

/// Each bit of this flag corresponds to a channel. 
/// 0 means not using Async, 1 means using Async
static EXTI_ASYNC_FLAGS: AtomicU16 = AtomicU16::new(0);

unsafe fn on_irq() {
    let bits: u32 = (*csdk::EXTI).PR;

    // We don't handle or change any EXTI lines above 16.
    let bits = bits & 0x0000FFFF;

    let async_flags = EXTI_ASYNC_FLAGS.load(Ordering::Relaxed) as u32;
    if ( async_flags & bits ) != 0 {
        // thembv6m has not fetch_or
        let new_flags = ( EXTI_POLL_FLAGS.load(Ordering::Relaxed) as u32 | bits ) as u16; 
        EXTI_POLL_FLAGS.store(new_flags, Ordering::Relaxed);

        // Mask all the channels that fired.
        (*csdk::EXTI).IMR &= !bits;

        // Wake the tasks
        for pin in BitIter(bits) {
            EXTI_WAKERS[pin as usize].wake();
        }

        let async_flags = ( EXTI_ASYNC_FLAGS.load(Ordering::Relaxed) as u32 & (!bits) ) as u16;
        EXTI_ASYNC_FLAGS.store(async_flags, Ordering::Relaxed);
    }

    // Clear the EXTI's line pending bits
    (*csdk::EXTI).PR = csdk::GPIO_PIN_All as _;
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

/// EXTI input driver.
///
/// This driver augments a GPIO `Input` with EXTI functionality. EXTI is not
/// built into `Input` itself because it needs to take ownership of the corresponding
/// EXTI channel, which is a limited resource.
///
/// Pins PA5, PB5, PC5... all use EXTI channel 5, so you can't use EXTI on, say, PA5 and PC5 at the same time.
pub struct ExtiInput {
    pin: AnyPin,
}

impl Unpin for ExtiInput {}

impl ExtiInput {
    /// Create an EXTI input.
    pub fn new(
        mut pin: AnyPin,
        pull: Pull,
        speed: Speed
    ) -> Self {
        let pin_num = pin.pin.trailing_zeros();

        #[cfg(feature = "py32f030")]
        let irqn: i32 = match pin_num {
            0..=1 => csdk::IRQn_Type_EXTI0_1_IRQn,
            2..=3 => csdk::IRQn_Type_EXTI2_3_IRQn,
            4..=15 => csdk::IRQn_Type_EXTI4_15_IRQn,
            _ => panic!(),
        };

        pin.c_init_type.Speed = speed.into();
        pin.c_init_type.Mode = csdk::GPIO_MODE_IT_RISING_FALLING;
        pin.c_init_type.Pull = pull.into();
        unsafe {
            csdk::HAL_GPIO_Init(pin.port,
                                &mut pin.c_init_type as *mut csdk::GPIO_InitTypeDef);
        }

        unsafe{
            csdk::HAL_NVIC_EnableIRQ(irqn);
            csdk::HAL_NVIC_SetPriority(irqn, 0, 0);
        }
        
        Self {
            pin,
        }
    }

    pub fn set_as_it(&mut self, rising: bool, falling: bool) {
        critical_section::with(|_| {
            let mut gpio_mode = csdk::GPIO_MODE_IT_RISING_FALLING;
            if rising & falling {
                gpio_mode = csdk::GPIO_MODE_IT_RISING_FALLING;
            }
            else if rising & (!falling) {
                gpio_mode = csdk::GPIO_MODE_IT_RISING;
            }
            else if (!rising) & falling {
                gpio_mode = csdk::GPIO_MODE_IT_FALLING;
            };
            self.pin.c_init_type.Mode = gpio_mode;
            
            unsafe {
                csdk::HAL_GPIO_Init(self.pin.port,
                    &mut self.pin.c_init_type as *mut csdk::GPIO_InitTypeDef);
            }
        });
    }

    pub fn set_as_event(&mut self, rising: bool, falling: bool) {
        critical_section::with(|_| {
            let mut gpio_mode = csdk::GPIO_MODE_EVT_RISING_FALLING;
            if rising & falling {
                gpio_mode = csdk::GPIO_MODE_EVT_RISING_FALLING;
            }
            else if rising & (!falling) {
                gpio_mode = csdk::GPIO_MODE_EVT_RISING;
            }
            else if (!rising) & falling {
                gpio_mode = csdk::GPIO_MODE_EVT_FALLING;
            };
            self.pin.c_init_type.Mode = gpio_mode;
            
            unsafe {
                csdk::HAL_GPIO_Init(self.pin.port,
                    &mut self.pin.c_init_type as *mut csdk::GPIO_InitTypeDef);
            }
        });
    }


    /// Get whether the pin is high.
    pub fn is_high(&self) -> bool {
        self.pin.is_high()
    }

    /// Get whether the pin is low.
    pub fn is_low(&self) -> bool {
        self.pin.is_low()
    }

    /// Get the pin level.
    pub fn get_level(&self) -> Level {
        self.pin.get_level()
    }

    /// Asynchronously wait until the pin is high.
    ///
    /// This returns immediately if the pin is already high.
    pub async fn wait_for_high(&mut self) {
        self.set_as_it(true, false);
        let fut = ExtiInputFuture::new(self.pin.pin);
        if self.is_high() {
            return;
        }
        fut.await
    }

    /// Asynchronously wait until the pin is low.
    ///
    /// This returns immediately if the pin is already low.
    pub async fn wait_for_low(&mut self) {
        self.set_as_it(false, true);
        let fut = ExtiInputFuture::new(self.pin.pin);
        if self.is_low() {
            return;
        }
        fut.await
    }

    /// Asynchronously wait until the pin sees a rising edge.
    ///
    /// If the pin is already high, it will wait for it to go low then back high.
    pub async fn wait_for_rising_edge(&mut self) {
        self.set_as_it(true, false);
        ExtiInputFuture::new(self.pin.pin).await
    }

    /// Asynchronously wait until the pin sees a falling edge.
    ///
    /// If the pin is already low, it will wait for it to go high then back low.
    pub async fn wait_for_falling_edge(&mut self) {
        self.set_as_it(false, true);
        ExtiInputFuture::new(self.pin.pin).await
    }

    /// Asynchronously wait until the pin sees any edge (either rising or falling).
    pub async fn wait_for_any_edge(&mut self) {
        self.set_as_it(true, true);
        ExtiInputFuture::new(self.pin.pin).await
    }
}

// impl embedded_hal_02::digital::v2::InputPin for ExtiInput {
//     type Error = Infallible;

//     fn is_high(&self) -> Result<bool, Self::Error> {
//         Ok(self.is_high())
//     }

//     fn is_low(&self) -> Result<bool, Self::Error> {
//         Ok(self.is_low())
//     }
// }

impl embedded_hal_1::digital::ErrorType for ExtiInput {
    type Error = Infallible;
}

impl embedded_hal_1::digital::InputPin for ExtiInput {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl embedded_hal_async::digital::Wait for ExtiInput {
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await;
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await;
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_rising_edge().await;
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_falling_edge().await;
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        self.wait_for_any_edge().await;
        Ok(())
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
struct ExtiInputFuture {
    pin: u16,
}

impl ExtiInputFuture {
    fn new(pin: u16) -> Self {

        let poll_flags = EXTI_POLL_FLAGS.load(Ordering::Relaxed) & (!pin);
        EXTI_POLL_FLAGS.store(poll_flags, Ordering::Relaxed);

        let async_flags = EXTI_ASYNC_FLAGS.load(Ordering::Relaxed) | pin; 
        EXTI_ASYNC_FLAGS.store(async_flags, Ordering::Relaxed);

        Self {
            pin,
        }
    }
}


impl Future for ExtiInputFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        EXTI_WAKERS[self.pin.trailing_zeros() as usize].register(cx.waker());
        let flags = EXTI_POLL_FLAGS.load(Ordering::Relaxed) as u32;
        if (flags & self.pin as u32) != 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn EXTI0_1() {
    on_irq()
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn EXTI2_3() {
    on_irq()
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn EXTI4_15() {
    on_irq()
}