use core::fmt::Debug;
use embedded_hal as hal;
use hal::blocking::i2c;

/// An error originating from this crate.
///
/// This cannot represent hardware (e.g. I²C) errors, so it is only used by functions that do not
/// (and will never) interact with hardware.
#[derive(Debug, Clone, Copy)]
pub enum HytError {
    MeasurementTakenInCommandMode,
    ScaleValueOutOfBounds,
}

/// A general error type, including errors originating from this crate (as [`HytError`]) and I²C
/// bus errors.
///
/// [`HytError`]: ./enum.HytError.html
pub enum Error<I2C>
where
    I2C: i2c::Read + i2c::Write,
{
    I2CRead(<I2C as i2c::Read>::Error),
    I2CWrite(<I2C as i2c::Write>::Error),
    Hyt(HytError),
}

impl<I2C> From<HytError> for Error<I2C>
where
    I2C: i2c::Read + i2c::Write,
{
    fn from(other: HytError) -> Self {
        Self::Hyt(other)
    }
}

impl<I2C, I2CReadError, I2CWriteError> Debug for Error<I2C>
where
    I2C: i2c::Read<Error = I2CReadError> + i2c::Write<Error = I2CWriteError>,
    I2CReadError: Debug,
    I2CWriteError: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::I2CRead(e) => write!(f, "I2CReadError({:?})", e),
            Self::I2CWrite(e) => write!(f, "I2CWriteError({:?})", e),
            Self::Hyt(e) => write!(f, "Hyt({:?})", e),
        }
    }
}
