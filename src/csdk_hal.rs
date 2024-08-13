use cortex_m_rt::exception;

use crate::csdk;

pub fn init(){
    unsafe {
        csdk::HAL_InitTick(csdk::TICK_INT_PRIORITY);
    }
}

#[exception]
fn SysTick(){
    unsafe {
        csdk::HAL_IncTick();
    }
    #[cfg(feature = "embassy")]
    crate::time_driver::on_interrupt();
}

impl From<csdk::HAL_StatusTypeDef> for crate::Error {
    fn from(status: csdk::HAL_StatusTypeDef) -> Self {
        match status {
            csdk::HAL_StatusTypeDef_HAL_ERROR => crate::Error::Error,
            csdk::HAL_StatusTypeDef_HAL_BUSY => crate::Error::Busy,
            csdk::HAL_StatusTypeDef_HAL_TIMEOUT => crate::Error::Timeout,
            csdk::HAL_StatusTypeDef_HAL_OK => panic!(),
            _ => panic!(),
        }
    }
}

pub fn check(operation: csdk::HAL_StatusTypeDef) -> Result<(), crate::Error> {
    match operation {
        csdk::HAL_StatusTypeDef_HAL_OK => Ok(()),
        err => Err(err.into()),
    }
}