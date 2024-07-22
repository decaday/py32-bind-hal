//! General-purpose Input/Output (GPIO)

// modified from https://github.com/embassy-rs/embassy/
// 4b0615957fe86218eb0529ef35e52305ab9e29c4

#[cfg(feature = "csdk-hal")]
pub mod csdk_hal;
#[cfg(feature = "csdk-hal")]
pub use csdk_hal::*;



/// Digital input or output level.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Level {
    /// Low
    Low,
    /// High
    High,
}

impl From<bool> for Level {
    fn from(val: bool) -> Self {
        match val {
            true => Self::High,
            false => Self::Low,
        }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}


/// Pull setting for an input.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Pull {
    /// No pull
    None,
    /// Pull up
    Up,
    /// Pull down
    Down,
}



/// Speed setting for an output.
///
/// These vary depending on the chip, check the reference manual and datasheet ("I/O port
/// characteristics") for details.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Speed {
    #[doc = "Output speed 00"]
    Low,
    #[doc = "Output speed 01"]
    Medium,
    #[doc = "Output speed 10"]
    High,
    #[doc = "Output speed 11"]
    VeryHigh,
}