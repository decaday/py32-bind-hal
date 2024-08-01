//! Power

// use core::convert::Infallible;
// use embedded_hal as embedded_hal_1;

use crate::csdk;


pub enum StopEntry {
    Wfi = csdk::PWR_STOPENTRY_WFI as isize,
    Wfe = csdk::PWR_STOPENTRY_WFE as isize,
}

pub enum SleepEntry {
    Wfi = csdk::PWR_SLEEPENTRY_WFI as isize,
    Wfe = csdk::PWR_SLEEPENTRY_WFE as isize,
}

pub enum RegulatorMode {
    MainRegulatorOn = csdk::PWR_MAINREGULATOR_ON as isize,
    LowPowerRegulatorOn = csdk::PWR_LOWPOWERREGULATOR_ON as isize,
}


#[inline]
pub fn enter_sleep_mode(entry: SleepEntry) {
    unsafe {
        csdk::HAL_PWR_EnterSLEEPMode(entry as u8);
    }
}

#[inline]
pub fn enter_stop_mode(regulator_mode: RegulatorMode, entry: StopEntry) {
    unsafe {
        csdk::HAL_PWR_EnterSTOPMode(regulator_mode as u32, entry as u8);
    }
}
