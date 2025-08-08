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
use embassy_time::{Duration, Timer};

use py32_bind_hal::{csdk, gpio, uart};
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

    let mut led = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_1).unwrap();
    led.set_as_output(gpio::Speed::High);

    let mut rx = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_10).unwrap();
    rx.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);
    let mut tx = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_2).unwrap();
    tx.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut uart_config = uart::Config::default();

    uart_config.init.BaudRate = 9600;

    let mut uart = uart::Uart::new_blocking(1, uart_config).unwrap();

    //uart.set_timeout(Timeout::new_mill(0xFFFFFFFF)); // MAX

    uart.blocking_write(b"Restart!\n").unwrap();

    const TIMEOUT_MS: u32 = 10;
    const PACKET_SIZE: usize = 18;

    let mut data_buffer: [u8; PACKET_SIZE] = [0; PACKET_SIZE];
    let mut current_pos = 0;

    // Timeout'u her bir byte için ayarla
    uart.set_timeout(Timeout::new_mill(TIMEOUT_MS));

    loop {
        let mut byte: [u8; 1] = [0];

        match uart.blocking_read(&mut byte) {
            Ok(()) => {
                // Byte okundu, tampona ekle
                data_buffer[current_pos] = byte[0];
                current_pos += 1;

                // Eğer 18 byte'a ulaştıysak, paketi tamamla
                if current_pos == PACKET_SIZE {
                    defmt::println!("Packet received with length: {}", current_pos);
                    let received_data =
                        core::str::from_utf8(&data_buffer).unwrap_or("Unexpected UTF-8 data.");
                    defmt::println!("Read data: {}", received_data);
                    
                    // Başarıyla okunan veriyi geri gönder
                    //uart.blocking_write(&data_buffer).unwrap();

                    // Yeni bir paket için hazırlan
                    current_pos = 0;

                    for i in 0..6 {                        
                        led.toggle();                        
                        Timer::after_millis(30).await;
                    }
                }
            }
            Err(e) => {
                // Okuma sırasında timeout oldu.
                if current_pos > 0 {
                    // Eğer paket ortasındaysak, paketi bozuk kabul et
                    defmt::println!("Packet broken due to timeout. {} bytes lost.", current_pos);
                }
                // Senkronizasyonu yeniden sağlamak için paketi sıfırla
                current_pos = 0;
            }
        }
    }
}
//--------------------------------------------------------------
//--------------------------------------------------------------
//--------------------------------------------------------------
