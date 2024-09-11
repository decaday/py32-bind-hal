//! Inter-Integrated-Circuit (I2C)
#![macro_use]

// modified from https://github.com/embassy-rs/embassy/
// 94007ce6e0fc59e374902eadcc31616e56068e43

use core::future::Future;
use core::marker::PhantomData;

use embedded_hal as embedded_hal_1;

#[cfg(feature = "time")]
use embassy_time::Duration;
use defmt::bitflags;

use csdk_hal::check;
use crate::*;
use crate::mode::{Async, Blocking, Mode};





/// I2C error.
// #[derive(Debug, PartialEq, Eq, Copy, Clone)]
// #[cfg_attr(feature = "defmt", derive(defmt::Format))]
// pub enum I2cError {
//     /// Bus error
//     Bus,
//     /// Arbitration lost
//     Arbitration,
//     /// ACK not received (either to the address or to a data byte)
//     Nack,
//     /// Timeout
//     Timeout,
//     /// CRC error
//     Crc,
//     /// Overrun error
//     Overrun,
//     #[cfg(feature = "peri-dma")]
//     /// DMA transfer error.
//     Dma,
//     #[cfg(feature = "peri-dma")]
//     /// DMA parameter error.
//     DmaParam,
//     /// Size management error.
//     Size,
//     /// CSDK error
//     Csdk,
//     /// Other error
//     Other,
// }

bitflags! {
    pub struct I2cErrorFlags: u32 {
        const NONE      = csdk::HAL_I2C_ERROR_NONE;
        const BUS       = csdk::HAL_I2C_ERROR_BERR;
        const ARBITRATION = csdk::HAL_I2C_ERROR_ARLO;
        const NACK      = csdk::HAL_I2C_ERROR_AF;
        const OVERRUN   = csdk::HAL_I2C_ERROR_OVR;
        const TIMEOUT   = csdk::HAL_I2C_ERROR_TIMEOUT;
        const SIZE      = csdk::HAL_I2C_ERROR_SIZE;
        #[cfg(feature = "peri-dma")]
        const DMA       = csdk::HAL_I2C_ERROR_DMA;
        #[cfg(feature = "peri-dma")]
        const DMA_PARAM = csdk::HAL_I2C_ERROR_DMA_PARAM;
    }
}


/// I2C config
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    pub init: csdk::I2C_InitTypeDef,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: Duration,
    #[cfg(not(feature = "time"))]
    pub timeout_tick: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init: csdk::I2C_InitTypeDef {
                ClockSpeed: 100000,
                DutyCycle: csdk::I2C_DUTYCYCLE_16_9,
                OwnAddress1: 0xA0,
                GeneralCallMode: csdk::I2C_GENERALCALL_DISABLE,
                NoStretchMode: csdk::I2C_NOSTRETCH_DISABLE,
            },
            #[cfg(feature = "time")]
            timeout: Duration::from_secs(2),
            #[cfg(not(feature = "time"))]
            timeout_tick: 2000,
        }
    }
}

pub struct I2c<M: Mode> {
    // scl: gpio::AnyPin,
    // sda: gpio::AnyPin,
    pub handle: csdk::I2C_HandleTypeDef,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: Duration,
    #[cfg(not(feature = "time"))]
    timeout_tick: u32,
    _phantom: PhantomData<M>,
}

impl I2c<Blocking> {
    /// Create a new blocking I2C driver.
    pub fn new_blocking_from_csdk(instance: *mut csdk::I2C_TypeDef, config: Config) -> Result<Self, Error<I2cErrorFlags>> {
        Self::new_from_csdk(instance, config)
    }

    #[cfg(feature = "peri-i2c0")]
    pub fn new_blocking(config: Config) -> Result<Self, Error<I2cErrorFlags>> {
        let instance = csdk::I2C;
        Self::new_from_csdk(instance, config)
    }

    #[cfg(not(feature = "peri-i2c0"))]
    pub fn new_blocking(instance_num: u8, config: Config) -> Result<Self, Error<I2cErrorFlags>> {
        let instance = match instance_num {
            #[cfg(feature = "peri-i2c1")]
            1 => csdk::I2C1,
            #[cfg(feature = "peri-i2c2")]
            2 => csdk::I2C2,
            // TODO
            _ => panic("unknown i2c id"),
        };
        let instance = csdk::I2C;
        Self::new_from_csdk(instance, config)
    }
}

// impl I2c<Async> {
//     /// Create a new I2C driver.
//     pub fn new_blocking(instance: *mut csdk::I2C_TypeDef, config: Config) -> Self {
//         Self::new_inner(instance, config)
//     }
// }

impl<M: Mode> I2c<M> {
    /// Create a new I2C driver.
    fn new_from_csdk(instance: *mut csdk::I2C_TypeDef, config: Config) -> Result<Self, Error<I2cErrorFlags>> {
        let handle = csdk::I2C_HandleTypeDef {
            Instance: instance,
            Init: config.init,
            pBuffPtr: core::ptr::null_mut(),
            XferSize: 0,
            XferCount: 0,
            XferOptions: 0,
            PreviousState: 0,
            hdmatx: core::ptr::null_mut(),
            hdmarx: core::ptr::null_mut(),
            Lock: 0,
            State: 0,
            Mode: 0,
            ErrorCode: 0,
            Devaddress: 0,
            Memaddress: 0,
            MemaddSize: 0,
            EventCount: 0,
        };
        
        let mut this = Self {
            handle,
            #[cfg(feature = "time")]
            timeout: config.timeout,
            #[cfg(not(feature = "time"))]
            timeout_tick: config.timeout_tick,
            _phantom: Default::default(),
        };
        this.enable_and_init()?;
        Ok(this)
    }

    fn enable_and_init(&mut self) -> Result<(), Error<I2cErrorFlags>> {
        unsafe{
            match self.handle.Instance {
                #[cfg(feature = "peri-i2c0")]
                csdk::I2C => {
                    csdk::HAL_RCC_I2C_CLK_ENABLE();
                    csdk::HAL_RCC_I2C_FORCE_RESET();
                    csdk::HAL_RCC_I2C_RELEASE_RESET();
                    Ok(())
                },
                #[cfg(feature = "peri-i2c1")]
                csdk::I2C1 => {
                    csdk::HAL_RCC_I2C1_CLK_ENABLE();
                    csdk::HAL_RCC_I2C1_FORCE_RESET();
                    csdk::HAL_RCC_I2C1_RELEASE_RESET();
                    Ok(())
                },
                #[cfg(feature = "peri-i2c2")]
                csdk::I2C2 => {
                    csdk::HAL_RCC_I2C2_CLK_ENABLE();
                    csdk::HAL_RCC_I2C2_FORCE_RESET();
                    csdk::HAL_RCC_I2C2_RELEASE_RESET();
                    Ok(())
                },
                _ => Err(Error::UserInput(InputError::InvalidInstant)),
            }?;
            check(csdk::HAL_I2C_Init(&mut self.handle), ||self.gerr())
        }
    }
}

// fn gerr(errorcode: u32) -> Error<I2cError> {
//     // TODO: multi error code
//     let err = match errorcode {
//         csdk::HAL_I2C_ERROR_NONE => I2cError::Csdk,
//         csdk::HAL_I2C_ERROR_BERR => I2cError::Bus,
//         csdk::HAL_I2C_ERROR_ARLO => I2cError::Arbitration,
//         csdk::HAL_I2C_ERROR_AF => I2cError::Nack,
//         csdk::HAL_I2C_ERROR_OVR => I2cError::Overrun,
//         #[cfg(feature = "peri-dma")]
//         csdk::HAL_I2C_ERROR_DMA => I2cError::Dma,
//         #[cfg(feature = "peri-dma")]
//         csdk::HAL_I2C_ERROR_DMA_PARAM => I2cError::DmaParam,
//         csdk::HAL_I2C_ERROR_TIMEOUT => I2cError::Timeout,
//         csdk::HAL_I2C_ERROR_SIZE => I2cError::Size,
//         _ => I2cError::Other,
//     };
//     Error::Error(err)
// }


impl<M: Mode> I2c<M> {
    fn gerr(&self) -> Error<I2cErrorFlags> {
        Error::HalError(I2cErrorFlags::from_bits_truncate(self.handle.ErrorCode))
    }

    fn get_timeout_tick(&self) -> u32 {
        #[cfg(feature = "time")]
        let timout_tick = self.timeout.as_ticks() as u32;
        #[cfg(not(feature = "time"))]
        let timout_tick = self.timeout_tick;
        timout_tick
    }

    fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error<I2cErrorFlags>> {
        let result = unsafe {
            csdk::HAL_I2C_Master_Receive(
                &mut self.handle,
                (address as u16) << 1,
                read.as_mut_ptr(),
                read.len() as u16,
                self.get_timeout_tick(),
            )
        };
        check(result, ||self.gerr())
    }

    fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error<I2cErrorFlags>> {
        let result = unsafe {
            csdk::HAL_I2C_Master_Transmit(
                &mut self.handle,
                (address as u16) << 1,
                write.as_ptr() as *mut u8,
                write.len() as u16,
                self.get_timeout_tick(),
            )
        };
        check(result, ||self.gerr())
    }

    fn blocking_write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Error<I2cErrorFlags>> {
        self.blocking_read(address, read)?;
        self.blocking_write(address, write)?;
        Ok(())
    }

    fn blocking_transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Error<I2cErrorFlags>> {
        for op in operations {
            match op {
                embedded_hal_1::i2c::Operation::Read(read) => self.blocking_read(address, read)?,
                embedded_hal_1::i2c::Operation::Write(write) => self.blocking_write(address, write)?,
            }
        }
        Ok(())
    }
}

impl embedded_hal_1::i2c::Error for Error<I2cErrorFlags> {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match self {
            Error::HalError(error_flags) => match *error_flags {
                I2cErrorFlags::BUS => embedded_hal_1::i2c::ErrorKind::Bus,
                I2cErrorFlags::ARBITRATION => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
                I2cErrorFlags::NACK => embedded_hal_1::i2c::ErrorKind::NoAcknowledge(embedded_hal_1::i2c::NoAcknowledgeSource::Unknown),
                I2cErrorFlags::OVERRUN => embedded_hal_1::i2c::ErrorKind::Overrun,
                _ => embedded_hal_1::i2c::ErrorKind::Other,
            },
            _ => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<M: Mode> embedded_hal_1::i2c::ErrorType for I2c<M> {
    type Error = Error<I2cErrorFlags>;
}

impl<M: Mode> embedded_hal_1::i2c::I2c for I2c<M> {
    fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_read(address, read)
    }

    fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(address, write)
    }

    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.blocking_write_read(address, write, read)
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.blocking_transaction(address, operations)
    }
}

// impl<'d> embedded_hal_async::i2c::I2c for I2c<Async> {
//     async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
//         self.read(address, read).await
//     }

//     async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
//         self.write(address, write).await
//     }

//     async fn write_read(
//         &mut self,
//         address: u8,
//         write: &[u8],
//         read: &mut [u8],
//     ) -> Result<(), Self::Error> {
//         self.write_read(address, write, read).await
//     }

//     async fn transaction(
//         &mut self,
//         address: u8,
//         operations: &mut [embedded_hal_1::i2c::Operation<'_>],
//     ) -> Result<(), Self::Error> {
//         self.transaction(address, operations).await
//     }
// }