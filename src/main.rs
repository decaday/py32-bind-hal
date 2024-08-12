#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use cortex_m_rt;
use cortex_m;
use {defmt_rtt as _, panic_probe as _};

use embedded_hal::{self as embedded_hal_1, i2c::I2c};

use embassy_executor::Spawner;
use embassy_time::Timer;

use bind_hal::gpio;
use py32csdk_hal_sys as csdk;
use bind_hal::power;
use bind_hal::i2c;
use bind_hal::exti;
use bind_hal::csdk_hal;


#[embassy_executor::main(entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) -> ! {
    bind_hal::init();
    defmt::println!("Hello, world!  1");
    init_pb3();
    defmt::println!("Hello, world!  2");
    // bind_hal::exit();
    // i2c_test();
    exti_test().await;
    defmt::println!("Hello, world!  3");

    loop {
        // defmt::println!("Hello World! n");
        Timer::after_millis(100).await;
    }
}


fn init_pb3() {
    csdk_hal::init();
    let mut pin = gpio::AnyPin::new_from_csdk(csdk::GPIOB, csdk::GPIO_PIN_3);
    pin.set_as_output(gpio::Speed::High);
    pin.set_high();

    let mut pin2 = gpio::AnyPin::new('B', 1);
    pin2.set_as_output(gpio::Speed::High);
    pin2.set_low();

    // power::enter_sleep_mode(power::SleepEntry::Wfi);
}

fn i2c_test() {
    let mut scl = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_3);
    scl.set_as_af_od(csdk::GPIO_AF12_I2C, gpio::Pull::Up, gpio::Speed::VeryHigh);
    let mut sda = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_2);
    sda.set_as_af_od(csdk::GPIO_AF12_I2C, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut config: i2c::Config = Default::default();
    config.own_address1 = 0x58;
    let mut i2c1 = i2c::I2c::new_blocking(config).unwrap();
    let data: [u8; 5] = [3; 5];
    loop{
        i2c1.write(0x53, &data).unwrap();
        unsafe {
            csdk::HAL_Delay(100);
        }
    }
}

async fn exti_test() {
    let mut pin = exti::ExtiInput::new(
        gpio::AnyPin::new('B', 6), 
        gpio::Pull::None, 
        gpio::Speed::High);
    pin.wait_for_any_edge().await;
}