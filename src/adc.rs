//! Analog to Digital Converter (ADC)

// modified from https://github.com/embassy-rs/embassy
// ab4d378dda5a74834dcc1fc0c872824f4a616911

use crate::csdk;
use crate::csdk_hal::check;

pub struct Adc {
    handle: csdk::ADC_HandleTypeDef,
    timeout_ticks: u32,
}

pub struct AdcConfig {
    init: csdk::ADC_InitTypeDef,
    timeout_ticks: u32,
}

impl Default for AdcConfig {
    fn default() -> Self {
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
}

impl Adc {
    pub fn new(config: AdcConfig, instance_num: u8) -> Result<Self, crate::Error> {
        let instance = Self::new_instance_from_num(instance_num);
        let mut adc = Self {
            handle: csdk::ADC_HandleTypeDef {
                Instance: instance,
                Init: config.init,
                DMA_Handle: core::ptr::null_mut(),
                Lock: 0,
                State: 0,
                ErrorCode: 0,
            },
            timeout_ticks: config.timeout_ticks,
        };
        adc.open_clock();
        unsafe {
            check(csdk::HAL_ADC_Init(&mut adc.handle))?;
        }
        Ok(adc)
    }

    fn open_clock(&self) {
        unsafe{
            match self.handle.Instance {
                csdk::ADC1 => {
                    csdk::HAL_RCC_ADC_FORCE_RESET();
                    csdk::HAL_RCC_ADC_RELEASE_RESET();
                    csdk::HAL_RCC_ADC_CLK_ENABLE();
                    },
                _ => todo!(),
            }
        }
    }

    fn new_instance_from_num(instance_num: u8) -> *mut csdk::ADC_TypeDef {
        match instance_num {
            1 => csdk::ADC1,
            _ => panic!(),
        }
    }

    pub fn new_regular_channel(&mut self, channel: u32) -> Result<(), crate::Error> {
        let mut channel_config = csdk::ADC_ChannelConfTypeDef {
            Channel: channel,
            Rank: csdk::ADC_RANK_CHANNEL_NUMBER,
            // Obsolete parameter
            SamplingTime: self.handle.Init.SamplingTimeCommon,
        };
        unsafe {
            check(csdk::HAL_ADC_ConfigChannel(&mut self.handle, &mut channel_config))
        }
    }

    pub fn start_blocking(&mut self) -> Result<(), crate::Error> {
        unsafe {
            check(csdk::HAL_ADC_Start(&mut self.handle))
        }
    }

    pub fn stop_blocking(&mut self) -> Result<(), crate::Error> {
        unsafe {
            check(csdk::HAL_ADC_Start(&mut self.handle))
        }
    }

    pub fn blocking_for_conversion(&mut self) -> Result<(), crate::Error> {
        unsafe {
            check(csdk::HAL_ADC_PollForConversion(&mut self.handle, self.timeout_ticks))
        }
    }

    pub fn blocking_read(&mut self) -> u32 {
        unsafe {
            csdk::HAL_ADC_GetValue(&mut self.handle)
        }
    }
}