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