#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use cortex_m_rt;
use cortex_m;
use {defmt_rtt as _, panic_probe as _};

use embedded_hal::{self as embedded_hal_1, i2c::I2c};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use bind_hal::{csdk, gpio, power, i2c, exti, rcc, adc, dma};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    bind_hal::init();
    defmt::println!("Hello, world!  1");
    init_pb3();
    defmt::println!("Hello, world!  2");
    rcc_test();
    adc_blocking_test();
    adc_dma_test();

    i2c_test();
    exti_test().await;
    
    // unsafe{
    //     let imr_value = (*csdk::EXTI).IMR;
    //     defmt::println!("IMR: {:X}", imr_value);
    // }

    loop {
        // defmt::println!("Hello World! n");
        Timer::after_millis(100).await;
    }
}


fn init_pb3() {
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
    config.init.OwnAddress1 = 0x58;
    config.timeout = Duration::from_millis(2000);
    let mut i2c1 = i2c::I2c::new_blocking(config).unwrap();
    unsafe { defmt::println!("SR1  {:?}", (*csdk::I2C).SR1) };
    unsafe { defmt::println!("SR2  {:?}", (*csdk::I2C).SR2) };
    let data: [u8; 5] = [3; 5];
    i2c1.write(0x53, &data).unwrap();
    loop{
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
    defmt::println!("wait_for_any_edge  1");
    pin.wait_for_any_edge().await;
    defmt::println!("wait_for_any_edge  2");
}

fn rcc_test() {
    rcc::into_48_mhz_hsi().unwrap();

    let freq = rcc::get_sys_clock_freq();
    defmt::println!("HAL_RCC_GetSysClockFreq  {}", freq);
}

fn adc_blocking_test() {
    let mut adc_config = adc::AdcConfig::new();
    adc_config.set_as_blocking();
    let mut adc = adc::Adc::new(adc_config, 1).unwrap();
    adc.new_regular_channel(csdk::ADC_CHANNEL_VREFINT).unwrap();
    adc.start_blocking().unwrap();
    let result = adc.blocking_read();
    defmt::println!("adc value  {}", result);
}

fn adc_dma_test() {
    let dma_config = dma::Config::new_peri_to_mem();
    let mut dma_channel = dma::DmaChannel::new(dma_config, 1, 0).unwrap();

    let mut adc_config = adc::AdcConfig::new();
    adc_config.set_as_dma();
    let mut adc = adc::Adc::new_dma(adc_config, 1, &mut dma_channel).unwrap();
    adc.new_regular_channel(csdk::ADC_CHANNEL_VREFINT).unwrap();


    let mut data: [u32; 1] = [3; 1];
    adc.start_dma(&mut data).unwrap();
    
    unsafe { csdk::HAL_Delay(100); }
    defmt::println!("adc dma value  {}", data);
    unsafe { csdk::HAL_Delay(100); }
    defmt::println!("adc dma value  {}", data);
}