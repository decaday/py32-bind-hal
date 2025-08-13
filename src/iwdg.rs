//! IWDG

use crate::csdk::{self, HAL_StatusTypeDef_HAL_OK, IWDG_HandleTypeDef, IWDG_InitTypeDef};    

#[inline]
pub fn iwdg_init(prescaler: u32) -> Result<IWDG_HandleTypeDef, csdk::HAL_StatusTypeDef> {
    let mut hiwdg: IWDG_HandleTypeDef = IWDG_HandleTypeDef {
        Instance: csdk::IWDG,
        Init: IWDG_InitTypeDef {
            Prescaler: prescaler,
            Reload: 4095,
        },
    };

    unsafe {
        csdk::HAL_RCC_LSI_ENABLE();
    }

    let status = unsafe { csdk::HAL_IWDG_Init(&mut hiwdg) };

    if status == HAL_StatusTypeDef_HAL_OK {
        Ok(hiwdg)
    } else {
        Err(status)
    }
}

#[inline]
pub fn iwdg_refresh(hiwdg: &mut IWDG_HandleTypeDef) {
    unsafe {
        csdk::HAL_IWDG_Refresh(hiwdg);
    }
}
