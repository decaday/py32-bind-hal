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

pub fn check<F, T>(status: csdk::HAL_StatusTypeDef, gerr: F) -> Result<(), crate::Error<T>> 
where
    F: FnOnce() -> crate::Error<T>,
{
    match status {
        csdk::HAL_StatusTypeDef_HAL_OK => Ok(()),
        csdk::HAL_StatusTypeDef_HAL_ERROR => Err(gerr()),
        csdk::HAL_StatusTypeDef_HAL_BUSY => Err(crate::Error::Busy),
        csdk::HAL_StatusTypeDef_HAL_TIMEOUT => Err(crate::Error::Timeout),
        _ => panic!(),
    }
}