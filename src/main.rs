#![no_main]
#![no_std]

use py32csdk_hal_sys as hal;


use core::ptr;
use core::ffi::c_void;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");
    init_pb3();
    defmt::println!("Hello, world!");
    defmt_test::exit()
}

extern "C" {
    pub fn HAL_RCC_GPIOB_CLK_ENABLE();
}

pub fn init_pb3() {
    let mut gpio_init = hal::GPIO_InitTypeDef {
        Pin: 0xFFFF,
        Mode: hal::GPIO_MODE_OUTPUT_PP,
        Pull: hal::GPIO_NOPULL,
        Speed: hal::GPIO_SPEED_FREQ_LOW,
        Alternate: 0,
    };

    unsafe {
        // let gpiob = hal::GPIOB_BASE as *mut hal::GPIO_TypeDef;
        let gpiob = 0x50000400 as *mut hal::GPIO_TypeDef;
        let freq = hal::HAL_RCC_GetHCLKFreq();
        let ver = hal::HAL_GetHalVersion();
        defmt::println!("Freq:{}, ver:{}",freq ,ver);
        defmt::println!("1:{}, 2:{}",gpiob ,&mut gpio_init as *mut hal::GPIO_InitTypeDef);


        HAL_RCC_GPIOB_CLK_ENABLE();

        hal::HAL_GPIO_Init(gpiob, &mut gpio_init as *mut _);
        hal::HAL_GPIO_WritePin(gpiob, 0x0002, 0);
        hal::HAL_GPIO_WritePin(gpiob, 0x0001, 1);
        // hal::HAL_GPIO_TogglePin(hal::GPIOB_BASE as *mut hal::GPIO_TypeDef,0x0008);
    }
}

