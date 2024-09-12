#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use cortex_m_rt;
use cortex_m;
use defmt;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m_semihosting::debug;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use py32_bind_hal::{csdk, gpio, power, i2c, exti, rcc, adc, dma, uart, timer};
use embedded_hal::i2c::I2c;

static mut ADC_DATA: [u32; 1] = [3; 1];

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    py32_bind_hal::init();

    init_pb3();

    rcc_test();

    adc_blocking_test();
        
    adc_dma_test();

    uart_test();

    timpwm_test();
    
    i2c_test();

    exti_test().await;

    loop {
        // defmt::println!("Hello World! n");
        Timer::after_millis(100).await;
    }
}


/// Sets the PB3 pin to output mode and sets it high.
/// This has the effect of enabling the external power supply.
fn init_pb3() {
    let mut pin = gpio::AnyPin::new_from_csdk(csdk::GPIOB, csdk::GPIO_PIN_3).unwrap();
    pin.set_as_output(gpio::Speed::High);
    pin.set_high();

    let mut pin2 = gpio::AnyPin::new('B', 2).unwrap();
    pin2.set_as_output(gpio::Speed::High);
    pin2.set_low();

    // power::enter_sleep_mode(power::SleepEntry::Wfi);
}

/// Tests the I2C interface by writing to the address 0x53 with the data "3".
/// This will send the data to an I2C device on the bus.
fn i2c_test() {
    let mut scl = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_8).unwrap();
    scl.set_as_af_od(csdk::GPIO_AF12_I2C, gpio::Pull::Up, gpio::Speed::VeryHigh);
    let mut sda = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_7).unwrap();
    sda.set_as_af_od(csdk::GPIO_AF12_I2C, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut config: i2c::Config = Default::default();
    config.init.OwnAddress1 = 0x58;
    config.timeout = Duration::from_millis(2000);
    let mut i2c1 = i2c::I2c::new_blocking(config).unwrap();
    let data: [u8; 5] = [3; 5];
    i2c1.write(0x53, &data).unwrap();
    loop{
        unsafe {   
            csdk::HAL_Delay(100);
        }
    }
}

/// Tests the EXTI (External Interrupts) interface by waiting for an edge on the
/// PB6 pin.
async fn exti_test() {
    let mut pin = exti::ExtiInput::new(
        gpio::AnyPin::new('B', 6).unwrap(), 
        gpio::Pull::None, 
        gpio::Speed::High);
    pin.wait_for_any_edge().await;
    defmt::println!("wait_for_any_edge  1");
    pin.wait_for_any_edge().await;
    defmt::println!("wait_for_any_edge  2");
}

/// Tests the HSE (High Speed External) clock source.
/// This is the clock source used by the system clock.
fn rcc_test() {
    rcc::into_48_mhz_hsi().unwrap();

    let freq = rcc::get_sys_clock_freq();
    defmt::println!("HAL_RCC_GetSysClockFreq  {}", freq);
}

/// Tests the ADC interface in blocking mode.
/// This means that the function will block until the ADC conversion is finished.
fn adc_blocking_test() {
    let mut adc_config = adc::AdcConfig::new();
    adc_config.set_as_blocking();
    let mut adc = adc::Adc::new(1, adc_config).unwrap();
    adc.new_regular_channel(csdk::ADC_CHANNEL_VREFINT).unwrap();
    adc.start_blocking().unwrap();
    let result = adc.blocking_read();
    defmt::println!("adc value  {}", result);
    adc.stop_blocking().unwrap();
}

/// Tests the ADC interface in DMA mode.
/// This means that the function will return immediately and the ADC conversion
/// will be done in the background.
fn adc_dma_test() {
    let dma_config = dma::Config::new_peri_to_mem();
    let mut dma_channel = dma::DmaChannel::new(dma_config, 1, 0).unwrap();

    let mut adc_config = adc::AdcConfig::new();
    adc_config.set_as_dma();
    let mut adc = adc::Adc::new_dma(1, adc_config, &mut dma_channel).unwrap();
    adc.new_regular_channel(csdk::ADC_CHANNEL_VREFINT).unwrap();

    unsafe {
        adc.start_dma(&mut ADC_DATA).unwrap();
    }
    
    unsafe { csdk::HAL_Delay(100); }
    unsafe{
        defmt::println!("adc dma value  {}", ADC_DATA);
    }

    unsafe { csdk::HAL_Delay(100); }
    // defmt::println!("adc dma value  {}", ADC_DATA);

    adc.stop_dma().unwrap();
}

/// Tests the UART interface by writing the string "a" to the serial port.
fn uart_test() {
    let mut scl = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_3).unwrap();
    scl.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);
    let mut sda = gpio::AnyPin::new_from_csdk(csdk::GPIOA, csdk::GPIO_PIN_2).unwrap();
    sda.set_as_af_pp(csdk::GPIO_AF1_USART1, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut uart_config = uart::Config::default();
    let mut uart = uart::Uart::new_blocking(1, uart_config).unwrap();
    let data: [u8; 5] = ['a' as u8 ; 5];
    uart.blocking_write(&data).unwrap();
}

/// Tests the TIM3 peripheral by setting the frequency to 1000 Hz and the pulse
/// width to 100.
fn timpwm_test() {
    let freq = rcc::get_pclk_freq();
    defmt::println!("HAL_RCC_GetPclkFreq  {}", freq);

    let config = timer::simple_pwm::Config::new(1000, 100);
    let mut tim3 = timer::simple_pwm::SimplePWM::new_from_csdk(csdk::TIM3, config).unwrap();

    let mut pin = gpio::AnyPin::new_from_csdk(csdk::GPIOB, csdk::GPIO_PIN_1).unwrap();
    pin.set_as_af_pp(csdk::GPIO_AF1_TIM3, gpio::Pull::Up, gpio::Speed::VeryHigh);

    let mut config = timer::simple_pwm::ChannelConfig::default();
    config.init.Pulse = 40;
    tim3.new_channel(timer::Channel::Ch4, config).unwrap();
}

