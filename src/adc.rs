//! Analog to Digital Converter (ADC)

// modified from https://github.com/embassy-rs/embassy
// ab4d378dda5a74834dcc1fc0c872824f4a616911

use embassy_hal_internal::into_ref;

use super::blocking_delay_us;
use crate::adc::{Adc, AdcChannel, Instance, Resolution, SampleTime};
use crate::peripherals::ADC1;
use crate::time::Hertz;
use crate::{rcc, Peripheral};

mod ringbuffered_v2;
pub use ringbuffered_v2::{RingBufferedAdc, Sequence};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

pub struct VrefInt;
impl AdcChannel<ADC1> for VrefInt {}
impl super::SealedAdcChannel<ADC1> for VrefInt {
    fn channel(&self) -> u8 {
        17
    }
}

impl VrefInt {
    /// Time needed for internal voltage reference to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Temperature;
impl AdcChannel<ADC1> for Temperature {}
impl super::SealedAdcChannel<ADC1> for Temperature {
    fn channel(&self) -> u8 {
        cfg_if::cfg_if! {
            if #[cfg(any(stm32f2, stm32f40x, stm32f41x))] {
                16
            } else {
                18
            }
        }
    }
}

impl Temperature {
    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Vbat;
impl AdcChannel<ADC1> for Vbat {}
impl super::SealedAdcChannel<ADC1> for Vbat {
    fn channel(&self) -> u8 {
        18
    }
}

enum Prescaler {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl Prescaler {
    fn from_pclk2(freq: Hertz) -> Self {
        // Datasheet for F2 specifies min frequency 0.6 MHz, and max 30 MHz (with VDDA 2.4-3.6V).
        #[cfg(stm32f2)]
        const MAX_FREQUENCY: Hertz = Hertz(30_000_000);
        // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
        #[cfg(not(stm32f2))]
        const MAX_FREQUENCY: Hertz = Hertz(36_000_000);
        let raw_div = freq.0 / MAX_FREQUENCY.0;
        match raw_div {
            0..=1 => Self::Div2,
            2..=3 => Self::Div4,
            4..=5 => Self::Div6,
            6..=7 => Self::Div8,
            _ => panic!("Selected PCLK2 frequency is too high for ADC with largest possible prescaler."),
        }
    }

    fn adcpre(&self) -> crate::pac::adccommon::vals::Adcpre {
        match self {
            Prescaler::Div2 => crate::pac::adccommon::vals::Adcpre::DIV2,
            Prescaler::Div4 => crate::pac::adccommon::vals::Adcpre::DIV4,
            Prescaler::Div6 => crate::pac::adccommon::vals::Adcpre::DIV6,
            Prescaler::Div8 => crate::pac::adccommon::vals::Adcpre::DIV8,
        }
    }
}

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(adc: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(adc);
        rcc::enable_and_reset::<T>();

        let presc = Prescaler::from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc.adcpre()));
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        blocking_delay_us(3);

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        }
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cr1().modify(|reg| reg.set_res(resolution.into()));
    }

    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vrefint(&self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(true);
        });

        VrefInt {}
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(true);
        });

        Temperature {}
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbate(true);
        });

        Vbat {}
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        // clear end of conversion flag
        T::regs().sr().modify(|reg| {
            reg.set_eoc(false);
        });

        // Start conversion
        T::regs().cr2().modify(|reg| {
            reg.set_swstart(true);
        });

        while T::regs().sr().read().strt() == false {
            // spin //wait for actual start
        }
        while T::regs().sr().read().eoc() == false {
            // spin //wait for finish
        }

        T::regs().dr().read().0 as u16
    }

    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        channel.setup();

        // Configure ADC
        let channel = channel.channel();

        // Select channel
        T::regs().sqr3().write(|reg| reg.set_sq(0, channel));

        // Configure channel
        Self::set_channel_sample_time(channel, self.sample_time);

        self.convert()
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });

        rcc::disable::<T>();
    }
}


#![macro_use]
#![allow(missing_docs)] // TODO
#![cfg_attr(adc_f3_v2, allow(unused))]

#[cfg(not(adc_f3_v2))]
#[cfg_attr(adc_f1, path = "f1.rs")]
#[cfg_attr(adc_f3, path = "f3.rs")]
#[cfg_attr(adc_f3_v1_1, path = "f3_v1_1.rs")]
#[cfg_attr(adc_v1, path = "v1.rs")]
#[cfg_attr(adc_l0, path = "v1.rs")]
#[cfg_attr(adc_v2, path = "v2.rs")]
#[cfg_attr(any(adc_v3, adc_g0, adc_h5, adc_u0), path = "v3.rs")]
#[cfg_attr(adc_v4, path = "v4.rs")]
#[cfg_attr(adc_g4, path = "g4.rs")]
mod _version;

use core::marker::PhantomData;

#[allow(unused)]
#[cfg(not(adc_f3_v2))]
pub use _version::*;
#[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
use embassy_sync::waitqueue::AtomicWaker;

#[cfg(not(any(adc_f1, adc_f3_v2)))]
pub use crate::pac::adc::vals::Res as Resolution;
pub use crate::pac::adc::vals::SampleTime;
use crate::peripherals;

dma_trait!(RxDma, Instance);

/// Analog to Digital driver.
pub struct Adc<'d, T: Instance> {
    #[allow(unused)]
    adc: crate::PeripheralRef<'d, T>,
    #[cfg(not(any(adc_f3_v2, adc_f3_v1_1)))]
    sample_time: SampleTime,
}

#[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
pub struct State {
    pub waker: AtomicWaker,
}

#[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
impl State {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

trait SealedInstance {
    #[allow(unused)]
    fn regs() -> crate::pac::adc::Adc;
    #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3_v2, adc_f3_v1_1, adc_g0)))]
    #[allow(unused)]
    fn common_regs() -> crate::pac::adccommon::AdcCommon;
    #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
    fn state() -> &'static State;
}

pub(crate) trait SealedAdcChannel<T> {
    #[cfg(any(adc_v1, adc_l0, adc_v2, adc_g4, adc_v4))]
    fn setup(&mut self) {}

    #[allow(unused)]
    fn channel(&self) -> u8;
}

/// Performs a busy-wait delay for a specified number of microseconds.
#[allow(unused)]
pub(crate) fn blocking_delay_us(us: u32) {
    #[cfg(feature = "time")]
    embassy_time::block_for(embassy_time::Duration::from_micros(us as u64));
    #[cfg(not(feature = "time"))]
    {
        let freq = unsafe { crate::rcc::get_freqs() }.sys.to_hertz().unwrap().0 as u64;
        let us = us as u64;
        let cycles = freq * us / 1_000_000;
        cortex_m::asm::delay(cycles as u32);
    }
}

/// ADC instance.
#[cfg(not(any(
    adc_f1,
    adc_v1,
    adc_l0,
    adc_v2,
    adc_v3,
    adc_v4,
    adc_g4,
    adc_f3,
    adc_f3_v1_1,
    adc_g0,
    adc_u0,
    adc_h5
)))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::Peripheral<P = Self> {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}
/// ADC instance.
#[cfg(any(
    adc_f1,
    adc_v1,
    adc_l0,
    adc_v2,
    adc_v3,
    adc_v4,
    adc_g4,
    adc_f3,
    adc_f3_v1_1,
    adc_g0,
    adc_u0,
    adc_h5
))]
#[allow(private_bounds)]
pub trait Instance: SealedInstance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// ADC channel.
#[allow(private_bounds)]
pub trait AdcChannel<T>: SealedAdcChannel<T> + Sized {
    #[allow(unused_mut)]
    fn degrade_adc(mut self) -> AnyAdcChannel<T> {
        #[cfg(any(adc_v1, adc_l0, adc_v2, adc_g4, adc_v4))]
        self.setup();

        AnyAdcChannel {
            channel: self.channel(),
            _phantom: PhantomData,
        }
    }
}

/// A type-erased channel for a given ADC instance.
///
/// This is useful in scenarios where you need the ADC channels to have the same type, such as
/// storing them in an array.
pub struct AnyAdcChannel<T> {
    channel: u8,
    _phantom: PhantomData<T>,
}

impl<T: Instance> AdcChannel<T> for AnyAdcChannel<T> {}
impl<T: Instance> SealedAdcChannel<T> for AnyAdcChannel<T> {
    fn channel(&self) -> u8 {
        self.channel
    }
}

foreach_adc!(
    ($inst:ident, $common_inst:ident, $clock:ident) => {
        impl crate::adc::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::adc::Adc {
                crate::pac::$inst
            }

            #[cfg(not(any(adc_f1, adc_v1, adc_l0, adc_f3_v2, adc_f3_v1_1, adc_g0)))]
            fn common_regs() -> crate::pac::adccommon::AdcCommon {
                return crate::pac::$common_inst
            }

            #[cfg(any(adc_f1, adc_f3, adc_v1, adc_l0, adc_f3_v1_1))]
            fn state() -> &'static State {
                static STATE: State = State::new();
                &STATE
            }
        }

        impl crate::adc::Instance for peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::GLOBAL;
        }
    };
);

macro_rules! impl_adc_pin {
    ($inst:ident, $pin:ident, $ch:expr) => {
        impl crate::adc::AdcChannel<peripherals::$inst> for crate::peripherals::$pin {}
        impl crate::adc::SealedAdcChannel<peripherals::$inst> for crate::peripherals::$pin {
            #[cfg(any(adc_v1, adc_l0, adc_v2, adc_g4, adc_v4))]
            fn setup(&mut self) {
                <Self as crate::gpio::SealedPin>::set_as_analog(self);
            }

            fn channel(&self) -> u8 {
                $ch
            }
        }
    };
}

/// Get the maximum reading value for this resolution.
///
/// This is `2**n - 1`.
#[cfg(not(any(adc_f1, adc_f3_v2)))]
pub const fn resolution_to_max_count(res: Resolution) -> u32 {
    match res {
        #[cfg(adc_v4)]
        Resolution::BITS16 => (1 << 16) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS14 => (1 << 14) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS14V => (1 << 14) - 1,
        #[cfg(adc_v4)]
        Resolution::BITS12V => (1 << 12) - 1,
        Resolution::BITS12 => (1 << 12) - 1,
        Resolution::BITS10 => (1 << 10) - 1,
        Resolution::BITS8 => (1 << 8) - 1,
        #[cfg(any(adc_v1, adc_v2, adc_v3, adc_l0, adc_g0, adc_f3, adc_f3_v1_1, adc_h5))]
        Resolution::BITS6 => (1 << 6) - 1,
        #[allow(unreachable_patterns)]
        _ => core::unreachable!(),
    }
}