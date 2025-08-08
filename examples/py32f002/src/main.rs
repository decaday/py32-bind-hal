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

    let mut rx = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_10).unwrap();
    rx.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);
    let mut tx = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_2).unwrap();
    tx.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let uart_config = uart::Config::default();    
    let mut uart = uart::Uart::new_blocking(1, uart_config).unwrap();   
        
    //uart.set_timeout(Timeout::new_mill(0xFFFFFFFF));// MAX
    uart.set_timeout(Timeout::new_mill(2000));   

    uart.blocking_write(b"Restart!\n").unwrap();
    
    let mut data: [u8; 18] = [0 as u8 ; 18];

    match uart.blocking_read(&mut data) {
        Ok(()) => {            
            defmt::println!("Was read to data {} length.", data.len());            
            let received_data = core::str::from_utf8(&data)
                .unwrap_or("Unexpected UTF-8 data.");
            defmt::println!("Read data: {}", received_data);
        },
        Err(e) => {            
            defmt::println!("Data read error!");
        }
    }        

    uart.blocking_write(&data).unwrap();

    uart.blocking_write(b"\r\nDone").unwrap();

    let mut pin = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_1).unwrap();
    pin.set_as_output(gpio::Speed::High);    

    loop {
        defmt::println!("Hello World!");
        pin.set_low();
        Timer::after_millis(300).await;
        pin.set_high();
        Timer::after_millis(30).await;
    }
}
//--------------------------------------------------------------
//--------------------------------------------------------------
//--------------------------------------------------------------
