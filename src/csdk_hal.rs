use cortex_m_rt::exception;

use py32csdk_hal_sys as csdk;

pub fn init(){
    unsafe {
        csdk::HAL_InitTick(csdk::TICK_INT_PRIORITY);
    }
}

#[exception]
fn SysTick(){
    csdk::HAL_IncTick();
    defmt::println!("Hello, world!");
}