#![no_main]
#![no_std]

use bind_hal::gpio;
use py32csdk_hal_sys as csdk;
use bind_hal::power;
use embedded_hal as embedded_hal_1;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");
    init_pb3();
    defmt::println!("Hello, world!");
    bind_hal::exit()
}

pub fn init_pb3() {
    let mut pin = gpio::AnyPin::new_from_c_macros(csdk::GPIOB, csdk::GPIO_PIN_3);
    pin.set_as_output(gpio::Speed::High);
    pin.set_high();

    let mut pin2 = gpio::AnyPin::new('B', 1);
    pin2.set_as_output(gpio::Speed::High);
    pin2.set_high();

    power::enter_sleep_mode(power::SleepEntry::Wfi);


}

