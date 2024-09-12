use crate::*;
use csdk_hal::check;
use defmt::bitflags;

pub struct Timer {
    pub handle: csdk::TIM_HandleTypeDef,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Channel {
    Ch1 = csdk::TIM_CHANNEL_1 as isize,
    Ch2 = csdk::TIM_CHANNEL_2 as isize,
    Ch3 = csdk::TIM_CHANNEL_3 as isize,
    Ch4 = csdk::TIM_CHANNEL_4 as isize,
}

pub mod simple_pwm {
    use crate::*;
    use csdk_hal::check;
    use super::Channel;
    
    pub struct Config {
        pub init: csdk::TIM_Base_InitTypeDef,
    }

    pub struct ChannelConfig {
        pub init: csdk::TIM_OC_InitTypeDef,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                init: csdk::TIM_Base_InitTypeDef {
                    Period: 50,
                    Prescaler: 4800 - 1,
                    ClockDivision: csdk::TIM_CLOCKDIVISION_DIV1,
                    CounterMode: csdk::TIM_COUNTERMODE_UP,
                    RepetitionCounter: 1 - 1,
                    AutoReloadPreload: csdk::TIM_AUTORELOAD_PRELOAD_DISABLE,
                }
            }
        }
    }

    impl Config {
        pub fn new(freq_hz: u32, period: u32) -> Self {
            let pclk_freq = rcc::get_pclk_freq();
            let mut config = Self::default();
            config.init.Period = period;
            config.init.Prescaler = (pclk_freq / freq_hz) / period - 1;
            config
        }
    }

    impl Default for ChannelConfig {
        fn default() -> Self {
            Self {
                init: csdk::TIM_OC_InitTypeDef {
                    OCMode: csdk::TIM_OCMODE_PWM1,
                    OCPolarity: csdk::TIM_OCPOLARITY_HIGH,
                    OCFastMode: csdk::TIM_OCFAST_DISABLE,
                    OCNPolarity: csdk::TIM_OCNPOLARITY_HIGH,
                    OCNIdleState: csdk::TIM_OCNIDLESTATE_RESET,
                    OCIdleState: csdk::TIM_OCIDLESTATE_RESET,
                    Pulse: 0,
                },
            }
        }
    }

    pub struct SimplePWM {
        pub handle: csdk::TIM_HandleTypeDef,
    }

    impl SimplePWM {
        pub fn new_from_csdk(instance: *mut csdk::TIM_TypeDef, config: Config) -> Result<Self, Error<()>> {
            let mut handle = csdk::TIM_HandleTypeDef {
                Instance: instance,
                Init: config.init,
                State: 0,
                Channel: 0,
                hdma: [core::ptr::null_mut(); 7],
                Lock: 0,
            };

            Self::open_clk(instance);
            unsafe {
                check(csdk::HAL_TIM_PWM_Init(&mut handle), ||Self::gerr())?;
            }
            Ok(Self { handle })
        }

        pub fn open_clk(instance: *mut csdk::TIM_TypeDef){
            unsafe {
                    match instance {
                    csdk::TIM1 => {
                        csdk::HAL_RCC_TIM1_CLK_ENABLE();
                    },
                    csdk::TIM3 => {
                        csdk::HAL_RCC_TIM3_CLK_ENABLE();
                    },
                    csdk::TIM14 => {
                        csdk::HAL_RCC_TIM14_CLK_ENABLE();
                    },
                    csdk::TIM16 => {
                        csdk::HAL_RCC_TIM16_CLK_ENABLE();
                    },
                    csdk::TIM17 => {
                        csdk::HAL_RCC_TIM16_CLK_ENABLE();
                    },
                    _ => panic!()
                }
            }
        }

        // pub fn new_channel(&mut self, channel: u32, config: ChannelConfig) {
        //     HAL_TIM_PWM_ConfigChannel(&TimHandle, &sConfig, TIM_CHANNEL_4)
        // }

        pub fn new_channel(&mut self, channel: Channel, mut config: ChannelConfig) -> Result<(), Error<()>> {
            unsafe {
                check(
                    csdk::HAL_TIM_PWM_ConfigChannel(&mut self.handle, &mut config.init, channel as u32), 
                    ||Self::gerr())?;
                check(csdk::HAL_TIM_PWM_Start(&mut self.handle, channel as u32), ||Self::gerr())
            }
        }

        pub fn update_channel(&mut self, channel: Channel, mut config: ChannelConfig) -> Result<(), Error<()>> {
            unsafe {
                check(
                    csdk::HAL_TIM_PWM_ConfigChannel(&mut self.handle, &mut config.init, channel as u32), 
                    ||Self::gerr())
            }   
        }

        pub fn set_channel_duty(&mut self, channel: Channel, duty: u32) {
            unsafe {
                let mut instance = *self.handle.Instance;
                match channel {
                    Channel::Ch1 => instance.CCR1 = duty,
                    Channel::Ch2 => instance.CCR2 = duty,
                    Channel::Ch3 => instance.CCR3 = duty,
                    Channel::Ch4 => instance.CCR4 = duty,
                }
            }
        }

        pub fn get_max_duty(&self) -> u32 {
            self.handle.Init.Period
        }

        pub fn stop_channel(&mut self, channel: Channel) -> Result<(), Error<()>> {
            unsafe {
                check(csdk::HAL_TIM_PWM_Stop(&mut self.handle, channel as u32), ||Self::gerr())
            }
        }

        pub fn gerr() -> Error<()> {
            Error::HalError(())
        }
    }


}