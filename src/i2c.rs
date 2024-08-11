//! Inter-Integrated-Circuit (I2C)
#![macro_use]

// modified from https://github.com/embassy-rs/embassy/
// 94007ce6e0fc59e374902eadcc31616e56068e43


use embedded_hal as embedded_hal_1;
use crate::csdk;
use crate::mode::{Async, Blocking, Mode};

use core::future::Future;
use core::marker::PhantomData;

#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};


/// I2C error.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration lost
    Arbitration,
    /// ACK not received (either to the address or to a data byte)
    Nack,
    /// Timeout
    Timeout,
    /// CRC error
    Crc,
    /// Overrun error
    Overrun,
    #[cfg(feature = "peri-dma")]
    /// DMA transfer error.
    Dma,
    #[cfg(feature = "peri-dma")]
    /// DMA parameter error.
    DmaParam,
    /// Size management error.
    Size,
    /// CSDK error
    Csdk,
    /// Other error
    Other,
}

/// I2C config
#[non_exhaustive]
#[derive(Copy, Clone)]
pub struct Config {
    /// Specifies the clock frequency.
    /// This parameter must be set to a value lower than 400kHz
    pub clock_speed: u32,
    /// Specifies the I2C fast mode duty cycle.
    /// his parameter can be a value of @ref I2C_duty_cycle_in_fast_mode
    pub duty_cycle: u32,
    /// Specifies the first device own address.
    /// This parameter can be a 7-bit or 10-bit address.
    pub own_address1: u32,
    /// Specifies if general call mode is selected.
    /// This parameter can be a value of @ref I2C_general_call_addressing_mode */
    pub general_call_mode: u32,
    /// Specifies if nostretch mode is selected.
    /// This parameter can be a value of @ref I2C_nostretch_mode
    pub no_stretch_mode: u32,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: embassy_time::Duration,
    #[cfg(not(feature = "time"))]
    pub timeout_tick: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock_speed: 100000,
            duty_cycle: csdk::I2C_DUTYCYCLE_16_9,
            own_address1: 0xA0,
            general_call_mode: csdk::I2C_GENERALCALL_DISABLE,
            no_stretch_mode: csdk::I2C_NOSTRETCH_DISABLE,
            #[cfg(feature = "time")]
            timeout: embassy_time::Duration::from_secs(2),
            #[cfg(not(feature = "time"))]
            timeout_tick: 2000,
        }
    }
}

pub struct I2c<M: Mode> {
    // scl: gpio::AnyPin,
    // sda: gpio::AnyPin,
    handle: csdk::I2C_HandleTypeDef,
    /// Timeout.
    #[cfg(feature = "time")]
    pub timeout: embassy_time::Duration,
    #[cfg(not(feature = "time"))]
    timeout_tick: u32,
    _phantom: PhantomData<M>,
}
impl I2c<Blocking> {
    /// Create a new blocking I2C driver.
    pub fn new_blocking_from_csdk(instance: *mut csdk::I2C_TypeDef, config: Config) -> Result<Self, Error> {
        Self::new_from_csdk(instance, config)
    }

    #[cfg(feature = "peri-i2c0")]
    pub fn new_blocking(config: Config) -> Result<Self, Error> {
        let instance = csdk::I2C;
        Self::new_from_csdk(instance, config)
    }

    #[cfg(not(feature = "peri-i2c0"))]
    pub fn new_blocking(instance_num: u8, config: Config) -> Result<Self, Error> {
        let instance = match instance_num {
            #[cfg(feature = "peri-i2c1")]
            1 => csdk::I21,
            #[cfg(feature = "peri-i2c2")]
            2 => csdk::I22,
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
    fn new_from_csdk(instance: *mut csdk::I2C_TypeDef, config: Config) -> Result<Self, Error> {
        let handle = csdk::I2C_HandleTypeDef {
            Instance: instance,
            Init: csdk::I2C_InitTypeDef {
                ClockSpeed: config.clock_speed,
                DutyCycle: config.duty_cycle,
                OwnAddress1: config.own_address1,
                GeneralCallMode: config.general_call_mode,
                NoStretchMode: config.no_stretch_mode,
            },
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

    fn enable_and_init(&mut self) -> Result<(), Error> {
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
                _ => Err(Error::Csdk),
            }?;
            match csdk::HAL_I2C_Init(&mut self.handle) {
                csdk::HAL_StatusTypeDef_HAL_OK => Ok(()),
                _ => Err(Error::Csdk),
            }
        }
    }

    fn timeout(&self) -> Timeout {
        Timeout {
            #[cfg(feature = "time")]
            deadline: Instant::now() + self.timeout,
        }
    }
}

impl<M: Mode> I2c<M> {
    fn get_error_code(&self, result: u32) -> Result<(), Error> {
        // defmt::println!("get_error_code:result: {} code: {}", result, self.handle.ErrorCode);
        if result == csdk::HAL_StatusTypeDef_HAL_OK {
            Ok(())
        } else {
            // TODO: multi error code
            let err = match self.handle.ErrorCode {
                csdk::HAL_I2C_ERROR_NONE => Error::Csdk,
                csdk::HAL_I2C_ERROR_BERR => Error::Bus,
                csdk::HAL_I2C_ERROR_ARLO => Error::Arbitration,
                csdk::HAL_I2C_ERROR_AF => Error::Nack,
                csdk::HAL_I2C_ERROR_OVR => Error::Overrun,
                #[cfg(feature = "peri-dma")]
                csdk::HAL_I2C_ERROR_DMA => Error::Dma,
                #[cfg(feature = "peri-dma")]
                csdk::HAL_I2C_ERROR_DMA_PARAM => Error::DmaParam,
                csdk::HAL_I2C_ERROR_TIMEOUT => Error::Timeout,
                csdk::HAL_I2C_ERROR_SIZE => Error::Size,
                _ => Error::Other,
            };
            Err(err)
        }
    }

    fn get_timeout_tick(&self) -> u32 {
        #[cfg(feature = "time")]
        let timout_tick = self.timeout.as_ticks() as u32;
        #[cfg(not(feature = "time"))]
        let timout_tick = self.timeout_tick;
        timout_tick
    }

    fn blocking_read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Error> {
        let result = unsafe {
            csdk::HAL_I2C_Master_Receive(
                &mut self.handle,
                (address as u16) << 1,
                read.as_mut_ptr(),
                read.len() as u16,
                self.get_timeout_tick(),
            )
        };
        self.get_error_code(result)
    }

    fn blocking_write(&mut self, address: u8, write: &[u8]) -> Result<(), Error> {
        let result = unsafe {
            csdk::HAL_I2C_Master_Transmit(
                &mut self.handle,
                (address as u16) << 1,
                write.as_ptr() as *mut u8,
                write.len() as u16,
                self.get_timeout_tick(),
            )
        };
        self.get_error_code(result)
    }

    fn blocking_write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Error> {
        self.blocking_read(address, read)?;
        self.blocking_write(address, write)?;
        Ok(())
    }

    fn blocking_transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_1::i2c::Operation<'_>],
    ) -> Result<(), Error> {
        for op in operations {
            match op {
                embedded_hal_1::i2c::Operation::Read(read) => self.blocking_read(address, read)?,
                embedded_hal_1::i2c::Operation::Write(write) => self.blocking_write(address, write)?,
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct Timeout {
    #[cfg(feature = "time")]
    deadline: Instant,
}

#[allow(dead_code)]
impl Timeout {
    #[inline]
    fn check(self) -> Result<(), Error> {
        #[cfg(feature = "time")]
        if Instant::now() > self.deadline {
            return Err(Error::Timeout);
        }

        Ok(())
    }

    #[inline]
    fn with<R>(
        self,
        fut: impl Future<Output = Result<R, Error>>,
    ) -> impl Future<Output = Result<R, Error>> {
        #[cfg(feature = "time")]
        {
            use futures_util::FutureExt;

            embassy_futures::select::select(embassy_time::Timer::at(self.deadline), fut).map(|r| {
                match r {
                    embassy_futures::select::Either::First(_) => Err(Error::Timeout),
                    embassy_futures::select::Either::Second(r) => r,
                }
            })
        }

        #[cfg(not(feature = "time"))]
        fut
    }
}

impl embedded_hal_1::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_1::i2c::ErrorKind {
        match *self {
            Self::Bus => embedded_hal_1::i2c::ErrorKind::Bus,
            Self::Arbitration => embedded_hal_1::i2c::ErrorKind::ArbitrationLoss,
            Self::Nack => embedded_hal_1::i2c::ErrorKind::NoAcknowledge(
                embedded_hal_1::i2c::NoAcknowledgeSource::Unknown,
            ),
            Self::Timeout => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Crc => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Overrun => embedded_hal_1::i2c::ErrorKind::Overrun,
            #[cfg(feature = "peri-dma")]
            Self::Dma => embedded_hal_1::i2c::ErrorKind::Other,
            #[cfg(feature = "peri-dma")]
            Self::DmaParam => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Size => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Csdk => embedded_hal_1::i2c::ErrorKind::Other,
            Self::Other => embedded_hal_1::i2c::ErrorKind::Other,
        }
    }
}

impl<M: Mode> embedded_hal_1::i2c::ErrorType for I2c<M> {
    type Error = Error;
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