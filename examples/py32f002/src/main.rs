#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use cortex_m_rt;
use defmt;
use defmt_rtt as _;
use panic_probe as _;
use py32_bind_hal::Timeout;

use cortex_m_semihosting::debug;

use embassy_executor::Spawner;
use embassy_time::Timer;

use py32_bind_hal::{csdk, gpio, uart, iwdg};

const TIMEOUT_MS: u32 = 100;

const fn get_timeout_from_ms(ms: u32) -> u32 {
    (ms + TIMEOUT_MS - 1) / TIMEOUT_MS
}
const SHORT_TERM_TIMEOUT: u32 = get_timeout_from_ms(5000);
const LONG_TERM_TIMEOUT: u32 = get_timeout_from_ms(60000);

const BARCODE_SIZE: usize = 18;

const BUFFER_SIZE: usize = 128;

use py32csdk_hal_sys::IWDG_PRESCALER_16; // ~2sn
//--------------------------------------------------------------
//--------------------------------------------------------------
//--------------------------------------------------------------
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}
//--------------------------------------------------------------
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    py32_bind_hal::init();   
    let mut hiwdg = iwdg::iwdg_init(IWDG_PRESCALER_16).expect("IWDG does not initalize!");
    
    Timer::after_millis(500).await;

    let mut output = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_1).unwrap();
    output.set_as_output(gpio::Speed::High);
    output.set_high();

    let mut input = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_2).unwrap();
    input.set_as_input(gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut rx = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_10).unwrap();
    rx.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut uart_config = uart::Config::default();

    uart_config.init.BaudRate = 9600;

    let mut uart = uart::Uart::new_blocking(1, uart_config).unwrap();

    let mut new_barcode: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut current_pos = 0;

    uart.set_timeout(Timeout::new_mill(TIMEOUT_MS));

    let mut short_timeout: u32 = LONG_TERM_TIMEOUT;
    let mut long_timeout: u32 = LONG_TERM_TIMEOUT;

    defmt::println!("Restart V1.1");

    loop {
        
        iwdg::iwdg_refresh(&mut hiwdg);

        if input.is_low() {
            // test basladı mı?
            // evet
            short_timeout = SHORT_TERM_TIMEOUT;
            long_timeout = LONG_TERM_TIMEOUT;
        }
        //-------------------
        if short_timeout > 0 {
            short_timeout -= 1;
            if short_timeout == 0 {
                output.set_high(); // testi engelle!
                defmt::println!("Short term timeout expired!");
            }
        }
        //-------------------
        if long_timeout > 0 {
            long_timeout -= 1;
            if long_timeout == 0 {
                output.set_high(); // testi engelle!
                defmt::println!("Long term timeout expired!");
            }
        }
        //-------------------
        let mut byte: [u8; 1] = [0];

        match uart.blocking_read(&mut byte) {
            Ok(()) => {
                new_barcode[current_pos] = byte[0];
                current_pos += 1;
            }
            Err(_e) => {
                if current_pos == BARCODE_SIZE {
                    defmt::println!("Packet received with length: {}", current_pos);
                    let received_data =
                        core::str::from_utf8(&new_barcode).unwrap_or("Unexpected UTF-8 data.");
                    defmt::println!("Read new barcode data: {}", received_data);

                    output.set_high();
                    Timer::after_millis(200).await;
                    output.set_low(); // teste izin ver!

                    short_timeout = LONG_TERM_TIMEOUT;
                    long_timeout = LONG_TERM_TIMEOUT;
                } else if current_pos > 0 {
                    defmt::println!("Packet broken due to timeout. {} bytes lost.", current_pos);
                }
                current_pos = 0;
            }
        }
    }
}
//--------------------------------------------------------------
//--------------------------------------------------------------
//--------------------------------------------------------------
