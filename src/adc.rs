//! Analog to Digital Converter (ADC)

// modified from https://github.com/embassy-rs/embassy
// ab4d378dda5a74834dcc1fc0c872824f4a616911

use defmt::bitflags;

use crate::*;
use crate::csdk_hal::check;
use crate::dma;



bitflags! {
    pub struct AdcErrorFlags: u32 {
        const INTERNAL_ERROR = csdk::HAL_ADC_ERROR_INTERNAL;
        const OVERRUN_ERROR = csdk::HAL_ADC_ERROR_OVR;
        #[cfg(feature = "peri-dma")]
        const DMA_ERROR = csdk::HAL_ADC_ERROR_DMA;
        //#[cfg(feature = "register-callbacks")]
        //const INVALID_CALLBACK = HAL_ADC_ERROR_INVALID_CALLBACK;
    }
}

pub struct Adc {
    pub handle: csdk::ADC_HandleTypeDef,
    timeout_ticks: u32,
}

pub struct AdcConfig {
    init: csdk::ADC_InitTypeDef,
    timeout_ticks: u32,
}

impl AdcConfig {
    pub fn new() -> Self {
        Self {
            init: csdk::ADC_InitTypeDef {
                ClockPrescaler: csdk::ADC_CLOCK_SYNC_PCLK_DIV1, // Set ADC clock
                Resolution: csdk::ADC_RESOLUTION_12B,       // 12-bit resolution
                DataAlign: csdk::ADC_DATAALIGN_RIGHT,      // Right alignment
                ScanConvMode: csdk::ADC_SCAN_DIRECTION_FORWARD, // Forward scan direction
                EOCSelection: csdk::ADC_EOC_SINGLE_CONV,       // Single conversion
                LowPowerAutoWait: csdk::FunctionalState_ENABLE,             // Enable low-power auto-wait
                ContinuousConvMode: csdk::FunctionalState_DISABLE,          // Disable continuous conversion
                DiscontinuousConvMode: csdk::FunctionalState_DISABLE,         // Disable discontinuous conversion
                ExternalTrigConv: csdk::ADC_SOFTWARE_START,    // Software trigger
                ExternalTrigConvEdge: csdk::ADC_EXTERNALTRIGCONVEDGE_NONE, // No trigger edge
                DMAContinuousRequests: csdk::FunctionalState_DISABLE,        // Disable DMA
                Overrun: csdk::ADC_OVR_DATA_OVERWRITTEN,   // Overwrite data on overrun
                SamplingTimeCommon: csdk::ADC_SAMPLETIME_13CYCLES_5, // Set sampling time to 41.5 ADC clock cycles
            },
            timeout_ticks: 10000,
        }
    }

    pub fn set_as_blocking(&mut self) {
        self.init.DMAContinuousRequests = csdk::FunctionalState_DISABLE;
        self.init.DiscontinuousConvMode = csdk::FunctionalState_DISABLE;
        self.init.ContinuousConvMode = csdk::FunctionalState_DISABLE;
    }

    pub fn set_as_dma(&mut self) {
        self.init.DMAContinuousRequests = csdk::FunctionalState_ENABLE;
        self.init.DiscontinuousConvMode = csdk::FunctionalState_DISABLE;
        self.init.ContinuousConvMode = csdk::FunctionalState_ENABLE;
    }

    pub fn set_software_start(&mut self) {
        self.init.ExternalTrigConv = csdk::ADC_SOFTWARE_START;
        self.init.ExternalTrigConvEdge = csdk::ADC_EXTERNALTRIGCONVEDGE_NONE;
    }
}

impl Adc {
    pub fn new(config: AdcConfig, instance_num: u8) -> Result<Self, Error<AdcErrorFlags>> {
        let mut adc = Self::new_inner(config, instance_num);
        adc.init_inner()?;
        Ok(adc)
    }

    pub fn new_dma(config: AdcConfig, instance_num: u8, dma: &mut dma::DmaChannel) -> Result<Self, Error<AdcErrorFlags>> {
        let mut adc = Self::new_inner(config, instance_num);
        dma.link(&mut adc);
        adc.init_inner()?;
        Ok(adc)
    }

    fn new_inner(config: AdcConfig, instance_num: u8) -> Self {
        let instance = Self::new_instance_from_num(instance_num);
        Self {
            handle: csdk::ADC_HandleTypeDef {
                Instance: instance,
                Init: config.init,
                DMA_Handle: core::ptr::null_mut(),
                Lock: 0,
                State: 0,
                ErrorCode: 0,
            },
            timeout_ticks: config.timeout_ticks,
        }
    }

    fn init_inner(&mut self) -> Result<(), Error<AdcErrorFlags>> {
        self.open_clock();

        unsafe {
            check(csdk::HAL_ADC_Calibration_Start(&mut self.handle), ||self.gerr())?;
            check(csdk::HAL_ADC_Init(&mut self.handle), ||self.gerr())?;
        }
        Ok(())
    }

    fn open_clock(&self) {
        unsafe{
            match self.handle.Instance {
                csdk::ADC1 => {
                    csdk::HAL_RCC_ADC_FORCE_RESET();
                    csdk::HAL_RCC_ADC_RELEASE_RESET();
                    csdk::HAL_RCC_ADC_CLK_ENABLE();
                    },
                _ => panic!(),
            }
        }
    }

    fn new_instance_from_num(instance_num: u8) -> *mut csdk::ADC_TypeDef {
        match instance_num {
            1 => csdk::ADC1,
            _ => panic!(),
        }
    }

    pub fn new_regular_channel(&mut self, channel: u32) -> Result<(), Error<AdcErrorFlags>> {
        let mut channel_config = csdk::ADC_ChannelConfTypeDef {
            Channel: channel,
            Rank: csdk::ADC_RANK_CHANNEL_NUMBER,
            // Obsolete parameter
            SamplingTime: self.handle.Init.SamplingTimeCommon,
        };
        unsafe {
            check(csdk::HAL_ADC_ConfigChannel(&mut self.handle, &mut channel_config), ||self.gerr())
        }
    }

    fn gerr(&self) -> Error<AdcErrorFlags> {
        Error::HalError(AdcErrorFlags::from_bits_truncate(self.handle.ErrorCode))
    }

    pub fn start_blocking(&mut self) -> Result<(), Error<AdcErrorFlags>> {
        unsafe {
            check(csdk::HAL_ADC_Start(&mut self.handle), ||self.gerr())
        }
    }

    pub fn stop_blocking(&mut self) -> Result<(), Error<AdcErrorFlags>> {
        unsafe {
            check(csdk::HAL_ADC_Start(&mut self.handle), ||self.gerr())
        }
    }

    pub fn blocking_for_conversion(&mut self) -> Result<(), Error<AdcErrorFlags>> {
        unsafe {
            check(csdk::HAL_ADC_PollForConversion(&mut self.handle, self.timeout_ticks), ||self.gerr())
        }
    }

    pub fn blocking_read(&mut self) -> u32 {
        unsafe {
            csdk::HAL_ADC_GetValue(&mut self.handle)
        }
    }

    /// You need to provide a 'static mutable borrow.
    /// If you are executing this function in a task that never ends, use `start_dma_unsafe`
    pub fn start_dma(&mut self, read: &'static mut [u32]) -> Result<(), Error<AdcErrorFlags>> {
        unsafe {
            check(csdk::HAL_ADC_Start_DMA(
                &mut self.handle,
                read.as_mut_ptr(),
                read.len() as u32), ||self.gerr())
        }
    }

    /// Please make sure that read is not destroyed before DMA stops.
    pub unsafe fn start_dma_unsafe(&mut self, read: & mut [u32]) -> Result<(), Error<AdcErrorFlags>> {
        check(csdk::HAL_ADC_Start_DMA(
            &mut self.handle,
            read.as_mut_ptr(),
            read.len() as u32), ||self.gerr())
    }

    pub fn stop_dma(&mut self) -> Result<(), Error<AdcErrorFlags>> {
        unsafe {
            check(csdk::HAL_ADC_Stop_DMA(&mut self.handle), ||self.gerr())
        }
    }
}

impl dma::HasDmaField for Adc {
    fn set_dma_field(&mut self, dma_handle: &mut dma::DmaChannel){
        self.handle.DMA_Handle = &mut dma_handle.handle;
    }
    
    fn get_handle_ptr(&mut self) -> *mut core::ffi::c_void {
        &mut self.handle 
            as *mut csdk::ADC_HandleTypeDef 
            as *mut core::ffi::c_void
    }
}