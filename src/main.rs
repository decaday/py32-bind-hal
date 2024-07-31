#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use cortex_m_rt;
use {defmt_rtt as _, panic_probe as _};

use embedded_hal as embedded_hal_1;

use embassy_executor::Spawner;
use embassy_time::Timer;

use bind_hal::gpio;
use py32csdk_hal_sys as csdk;
use bind_hal::power;
use bind_hal::csdk_hal;


#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    bind_hal::init();
    defmt::println!("Hello, world!  1");
    init_pb3();
    Timer::after_millis(1000).await;
    defmt::println!("Hello, world!  2");
    // bind_hal::exit();

    loop {
        defmt::info!("Hello World! n");
        Timer::after_secs(1).await;
    }
}

pub fn init_pb3() {
    csdk_hal::init();
    let mut pin = gpio::AnyPin::new_from_c_macros(csdk::GPIOB, csdk::GPIO_PIN_3);
    pin.set_as_output(gpio::Speed::High);
    pin.set_high();

    let mut pin2 = gpio::AnyPin::new('B', 1);
    pin2.set_as_output(gpio::Speed::High);
    pin2.set_low();

    // power::enter_sleep_mode(power::SleepEntry::Wfi);
}

#[embassy_executor::task(pool_size = 2)]
async fn wait_print(delay: u64) {
    loop {
        defmt::info!("high1");
        Timer::after_millis(delay).await;

        defmt::info!("low2");
        Timer::after_millis(delay).await;
    }
}