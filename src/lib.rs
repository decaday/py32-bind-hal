#![no_main]
#![no_std]

use cortex_m_semihosting::debug;

use defmt_rtt as _; // global logger

use panic_probe as _;

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Error {
    Error,
    Busy,
    Timeout,
    UserInput(InputError),
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InputError {
    InvalidInstant,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InputErrorType {
    Instant,
}


pub fn init(){
    crate::csdk_hal::init();

    unsafe {
        csdk::HAL_RCC_SYSCFG_CLK_ENABLE();
        csdk::HAL_RCC_PWR_CLK_ENABLE();
    }

    #[cfg(feature = "embassy")]
    crate::time_driver::init();
}


pub struct Timeout {
    #[cfg(feature = "time")]
    pub timeout: embassy_time::Duration,
    #[cfg(not(feature = "time"))]
    pub timeout_tick: u32,
}

impl Timeout {
    pub fn new_mill(mill: u32) -> Self {
        Self {
            #[cfg(feature = "time")]
            timeout: embassy_time::Duration::from_millis(mill as u64),
            #[cfg(not(feature = "time"))]
            timeout_tick: mill,
        }
    }


    #[inline]
    pub fn get_tick(&self) -> u32 {
        #[cfg(feature = "time")]
        let timout_tick = self.timeout.as_ticks() as u32;
        #[cfg(not(feature = "time"))]
        let timout_tick = self.timeout_tick;
        timout_tick
    }
}

pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    macro_rules! impl_mode {
        ($name:ident) => {
            impl SealedMode for $name {}
            impl Mode for $name {}
        };
    }

    /// Blocking mode.
    pub struct Blocking;
    /// Async mode.
    pub struct Async;

    impl_mode!(Blocking);
    impl_mode!(Async);
}


pub use py32csdk_hal_sys as csdk;

pub mod gpio;

pub mod power;

#[cfg(feature = "peri-i2c")]
pub mod i2c;

pub mod uart;

pub mod exti;

pub mod rcc;

pub mod adc;

pub mod dma;

pub mod csdk_hal;

mod time_driver;