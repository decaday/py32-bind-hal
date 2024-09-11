//! Universal Asynchronous Receiver Transmitter (UART)


use core::future::Future;
use core::marker::PhantomData;

use embedded_hal as embedded_hal_1;

#[cfg(feature = "time")]
use embassy_time::Duration;
use defmt::bitflags;

use csdk_hal::check;
use crate::*;
use crate::mode::{Async, Blocking, Mode};

pub struct Config {
    pub init: csdk::UART_InitTypeDef,
    pub advanced_init: csdk::UART_AdvFeatureInitTypeDef,
    timeout: Timeout,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init: csdk::UART_InitTypeDef {
                BaudRate: 115200,
                WordLength: csdk::UART_WORDLENGTH_8B,
                StopBits: csdk::UART_STOPBITS_1,
                Parity: csdk::UART_PARITY_NONE,
                HwFlowCtl: csdk::UART_HWCONTROL_NONE,
                OverSampling: csdk::UART_OVERSAMPLING_16,
                Mode: csdk::UART_MODE_TX_RX,
            },
            advanced_init: csdk::UART_AdvFeatureInitTypeDef {
                AdvFeatureInit: csdk::UART_ADVFEATURE_NO_INIT,
                AutoBaudRateEnable: csdk::UART_ADVFEATURE_AUTOBAUDRATE_DISABLE,
                AutoBaudRateMode: 0,
            },
            timeout:Timeout::new_mill(2000),
        }
    }
}

pub struct Uart<M: Mode> {
    // scl: gpio::AnyPin,
    // sda: gpio::AnyPin,
    pub handle: csdk::UART_HandleTypeDef,
    timeout: Timeout,
    _phantom: PhantomData<M>,
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SerialError {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    /// Buffer too large for DMA
    BufferTooLong,
}


bitflags! {
    pub struct UartErrorFlags: u32 {
        const PARITY_ERROR = csdk::HAL_UART_ERROR_PE;
        const NOISE_ERROR = csdk::HAL_UART_ERROR_NE;
        const FRAME_ERROR = csdk::HAL_UART_ERROR_FE;
        const OVERRUN_ERROR = csdk::HAL_UART_ERROR_ORE;
        #[cfg(feature = "peri-dma")]
        const DMA_ERROR = csdk::HAL_UART_ERROR_DMA;
        //#[cfg(feature = "register-callbacks")]
        //const INVALID_CALLBACK = HAL_UART_ERROR_INVALID_CALLBACK;
    }
}

impl Uart<Blocking> {
    /// Create a new blocking I2C driver.
    pub fn new_blocking_from_csdk(instance: *mut csdk::USART_TypeDef, config: Config) -> Result<Self, Error<UartErrorFlags>> {
        Self::new_from_csdk(instance, config)
    }

    pub fn new_blocking(instance_num: u8, config: Config) -> Result<Self, Error<UartErrorFlags>> {
        let instance = match instance_num {
            // #[cfg(feature = "peri-usart1")]
            1 => csdk::USART1,
            // #[cfg(feature = "peri-usart2")]
            2 => csdk::USART2,
            // TODO
            _ => Err(Error::UserInput(InputError::InvalidInstance))?,
        };
        Self::new_from_csdk(instance, config)
    }
}



impl<M: Mode> Uart<M> {
    pub fn new_from_csdk(instance: *mut csdk::USART_TypeDef,config: Config) -> Result<Self, Error<UartErrorFlags>> {
        let mut this = Self {
            handle: csdk::UART_HandleTypeDef {
                Instance: instance,
                Init: config.init,
                AdvancedInit: config.advanced_init,
                pTxBuffPtr: core::ptr::null_mut(),
                TxXferSize: 0,
                TxXferCount: 0,
                pRxBuffPtr: core::ptr::null_mut(),
                RxXferSize: 0,
                RxXferCount: 0,
                hdmatx: core::ptr::null_mut(),
                hdmarx: core::ptr::null_mut(),
                Lock: 0,
                gState: 0,
                RxState: 0,
                ErrorCode: 0,
            },
            _phantom: PhantomData,
            timeout: config.timeout,
        };

        this.enable_and_init()?;

        Ok(this)
    }

    fn enable_and_init(&mut self) -> Result<(), Error<UartErrorFlags>> {
        unsafe{
            match self.handle.Instance {
                csdk::USART1 => {
                    csdk::HAL_RCC_USART1_CLK_ENABLE();
                    Ok(())
                },
                csdk::USART2 => {
                    csdk::HAL_RCC_USART2_CLK_ENABLE();
                    Ok(())
                },
                _ => Err(Error::UserInput(InputError::InvalidInstance)),
            }?;
            check(csdk::HAL_UART_Init(&mut self.handle),  ||self.gerr())
        }
    }

    fn gerr(&self) -> Error<UartErrorFlags> {
        Error::HalError(UartErrorFlags::from_bits_truncate(self.handle.ErrorCode))
    }

    pub fn blocking_write(&mut self, buffer: &[u8]) -> Result<(), Error<UartErrorFlags>> {
        unsafe {
            check(csdk::HAL_UART_Transmit(&mut self.handle, 
                buffer.as_ptr() as *mut u8, 
                buffer.len() as u16, 
                self.timeout.get_tick()), ||self.gerr())
        }   
    }

    pub fn blocking_read(&mut self, buffer: &mut [u8]) -> Result<(), Error<UartErrorFlags>> {
        unsafe {
            check(csdk::HAL_UART_Receive(&mut self.handle, 
                buffer.as_ptr() as *mut u8, 
                buffer.len() as u16, 
                self.timeout.get_tick()), ||self.gerr())
        }
    }
}
