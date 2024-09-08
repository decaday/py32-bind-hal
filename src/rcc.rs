//! Reset and Clock Control

// use core::convert::Infallible;
// use embedded_hal as embedded_hal_1;

use crate::csdk;

pub struct RccConfig{
    pub osc_init: csdk::RCC_OscInitTypeDef,
    pub clk_init: csdk::RCC_ClkInitTypeDef,
}

impl Default for RccConfig {
    fn default() -> Self {
        Self {
            osc_init: csdk::RCC_OscInitTypeDef {
                OscillatorType: csdk::RCC_OSCILLATORTYPE_HSE |
                    csdk::RCC_OSCILLATORTYPE_HSI |
                    csdk::RCC_OSCILLATORTYPE_LSI |
                    csdk::RCC_OSCILLATORTYPE_LSE,
                HSIState: csdk::RCC_HSI_ON,
                HSIDiv: csdk::RCC_HSI_DIV1,
                HSICalibrationValue: unsafe { csdk::RCC_GET_HSICALIBRATION_16MHz() },
                HSEState: csdk::RCC_HSE_OFF,
                HSEFreq: csdk::RCC_HSE_16_32MHz,
                LSIState: csdk::RCC_LSI_OFF,
                LSEState: csdk::RCC_LSE_OFF,
                LSEDriver: csdk::RCC_LSEDRIVE_MEDIUM,
                PLL: csdk::RCC_PLLInitTypeDef {
                    PLLState: csdk::RCC_PLL_ON,
                    PLLSource: csdk::RCC_PLLSOURCE_HSI,
                },
            },
            clk_init: csdk::RCC_ClkInitTypeDef {
                ClockType: csdk::RCC_CLOCKTYPE_HCLK |
                    csdk::RCC_CLOCKTYPE_SYSCLK |
                    csdk::RCC_CLOCKTYPE_PCLK1,
                SYSCLKSource: csdk::RCC_SYSCLKSOURCE_PLLCLK,
                AHBCLKDivider: csdk::RCC_SYSCLK_DIV1,
                APB1CLKDivider: csdk::RCC_HCLK_DIV1,
            },
        }
    }
}

impl RccConfig {
    pub fn new_from_csdk(osc_init: csdk::RCC_OscInitTypeDef,
        clk_init: csdk::RCC_ClkInitTypeDef
    ) -> Self {
        Self {
            osc_init,
            clk_init
        }
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn apply(& mut self) -> Result<(), crate::Error> {
        unsafe{
            match csdk::HAL_RCC_OscConfig(&mut self.osc_init) {
                csdk::HAL_StatusTypeDef_HAL_OK => Ok::<(), crate::Error>(()),
                err => Err(err.into())
            }?;
            match csdk::HAL_RCC_ClockConfig(&mut self.clk_init, csdk::FLASH_LATENCY_1) {
                csdk::HAL_StatusTypeDef_HAL_OK => Ok(()),
                err => Err(err.into())
            }
        }
    }
}

#[cfg(feature = "py32f030")]
pub fn into_48_mhz_hsi() -> Result<(), crate::Error> {
    let mut rcc = RccConfig::new();
    rcc.osc_init.HSICalibrationValue = unsafe { csdk::RCC_GET_HSICALIBRATION_24MHz() };
    rcc.osc_init.PLL.PLLState = csdk::RCC_PLL_ON;
    rcc.apply()
}

#[cfg(feature = "py32f030")]
pub fn into_32_mhz_hsi() -> Result<(), crate::Error> {
    let mut rcc = RccConfig::new();
    rcc.osc_init.HSICalibrationValue = unsafe { csdk::RCC_GET_HSICALIBRATION_16MHz() };
    rcc.osc_init.PLL.PLLState = csdk::RCC_PLL_ON;
    rcc.apply()
}

#[cfg(feature = "py32f030")]
pub fn into_8_mhz_hsi() -> Result<(), crate::Error> {
    let mut rcc = RccConfig::new();
    rcc.osc_init.HSICalibrationValue = unsafe { csdk::RCC_GET_HSICALIBRATION_8MHz() };
    rcc.osc_init.PLL.PLLState = csdk::RCC_PLL_OFF;
    rcc.apply()
}

#[cfg(feature = "py32f030")]
pub fn into_1_mhz_hsi() -> Result<(), crate::Error> {
    let mut rcc = RccConfig::new();
    rcc.osc_init.HSICalibrationValue = unsafe { csdk::RCC_GET_HSICALIBRATION_8MHz() };
    rcc.osc_init.PLL.PLLState = csdk::RCC_PLL_OFF;
    rcc.osc_init.HSIDiv = csdk::RCC_HSI_DIV8;
    rcc.clk_init.SYSCLKSource = csdk::RCC_SYSCLKSOURCE_HSI;
    rcc.apply()
}

pub fn get_sys_clock_freq() -> u32 {
    unsafe {
        csdk::HAL_RCC_GetSysClockFreq()
    }
}