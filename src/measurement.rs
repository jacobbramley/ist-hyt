use crate::error::HytError;
use core::convert::TryFrom;

#[cfg(feature = "i8f24")]
use fixed::types::I8F24;

/// A single measurement response.
///
/// This represents a reading of both temperature and humidity, plus some metadata. Accessors are
/// provided for all meaningful fields.
#[derive(Clone, Copy, Debug)]
pub struct Measurement {
    raw: [u8; 4],
}

const RAW_VALUE_MAX: u16 = 0x3fff;

impl Measurement {
    /// Create a new `Measurement` abstraction based on raw data from a sensor.
    ///
    /// The format is as follows:
    ///
    /// ```text
    /// Element: [raw[0]         |raw[1]         |raw[2]         |raw[3]         ]
    ///     Bit: [7 6 5         0|7             0|7             0|7         2 1 0]
    ///           | | \_________________________/ \_________________________/ \_/
    ///       CMode |    Humidity<13:0> (raw)       Temperature<13:0> (raw)    |
    ///           Stale                                                      Unused
    /// ```
    ///
    /// This fails (with an appropriate [`HytError`]) if `CMode` (command mode) is set. All other
    /// bit patterns are considered valid. The unused bits (`raw[3]<1:0>`) are ignored.
    ///
    /// [`HytError`]: ./enum.HytError.html
    pub fn from_raw(raw: [u8; 4]) -> Result<Self, HytError> {
        if raw[0] & 0b1000_0000 == 0 {
            Ok(Self { raw })
        } else {
            Err(HytError::MeasurementTakenInCommandMode)
        }
    }

    /// A measurement is "stale" if it has already been read from the sensor. This typically occurs
    /// when a pending measurement is not yet complete, in which case the previous reading is
    /// returned.
    ///
    /// The staleness of a measurement is decided when it is read from the sensor, and is not
    /// related to the age or history of the Rust object.
    pub fn is_stale(&self) -> bool {
        (self.raw[0] & 0b0100_0000) != 0
    }

    /// Calculate the relative humidity, in %RH, returning the result as a fixed-point value.
    ///
    /// The fixed-point result can represent the whole range of results with a worst-case error of
    /// about 0.00024 %RH, which is insignificant compared to the resolution of the reading (about
    /// 0.006 %RH) and the accuracy of the sensor (±1.8 %RH).
    ///
    ///
    /// _This requires the "i8f24" feature._
    #[cfg(feature = "i8f24")]
    pub fn humidity_i8f24(&self) -> I8F24 {
        I8F24::from_bits(self.humidity_scaled(1 << 24).unwrap())
    }

    /// Calculate the temperature, in °C, returning the result as a fixed-point value.
    ///
    /// The fixed-point result can represent the whole range of results with a worst-case error of
    /// about 0.00031°C, which is insignificant compared to the resolution of the reading (about
    /// 0.01°C) and the accuracy of the sensor (±0.2°C).
    ///
    /// _This requires the "i8f24" feature._
    #[cfg(feature = "i8f24")]
    pub fn temperature_i8f24(&self) -> I8F24 {
        I8F24::from_bits(self.temperature_scaled(1 << 24).unwrap())
    }

    /// Calculate the relative humidity, in %RH, returning the result as a scaled integer.
    ///
    /// This is less convenient than [`humidity_i8f24()`], but passing a `scale` like `10` or `100`
    /// makes makes printing decimal results efficient.
    ///
    /// If `scale` is too large for all possible values of the result to be represented, this
    /// returns `Err(HytError::ScaleValueOutOfBounds)`, even if the actual reading obtained
    /// _could_ be represented.
    ///
    /// [`humidity_i8f24()`]: #method.humidity
    // TODO: Example
    pub fn humidity_scaled(&self, scale: u32) -> Result<i32, HytError> {
        Self::value_scaled(self.humidity_raw(), 0, 100, scale)
    }

    /// Calculate the temperature, in °C, returning the result as a scaled integer.
    ///
    /// This is less convenient than [`temperature_i8f24()`], but passing a `scale` like `10` or
    /// `100` makes printing decimal results efficient.
    ///
    /// If `scale` is too large for all possible values of the result to be represented, this
    /// returns `Err(HytError::ScaleValueOutOfBounds)`, even if the actual reading obtained
    /// _could_ be represented.
    ///
    /// [`temperature_i8f24()`]: #method.temperature
    // TODO: Example
    pub fn temperature_scaled(&self, scale: u32) -> Result<i32, HytError> {
        Self::value_scaled(self.temperature_raw(), -40, 125, scale)
    }

    /// Calculate the humidity, in %RH, to the nearest integer.
    // TODO: Make the return type generic.
    pub fn humidity(&self) -> i32 {
        self.humidity_scaled(1).unwrap()
    }

    /// Calculate the temperature, in °C, to the nearest integer.
    // TODO: Make the return type generic.
    pub fn temperature(&self) -> i32 {
        self.temperature_scaled(1).unwrap()
    }

    // For constant arguments, the representability check can be optimised out, but only when this
    // is inlined. Empirically, marking this inline typically results in much smaller code overall.
    #[inline(always)]
    fn value_scaled(raw: u16, min: i16, max: i16, scale: u32) -> Result<i32, HytError> {
        assert!(max > min);
        assert!(raw <= RAW_VALUE_MAX);
        // Multiply as much as we can before dividing, to maximise precision.
        let range = max.wrapping_sub(min) as u32;
        let raw = u32::from(raw);
        let min_scaled = i64::from(scale) * i64::from(min);
        let round = u64::from(RAW_VALUE_MAX / 2);
        let num = u64::from(scale) * u64::from(raw * range) + round;
        let max_num = u64::from(scale) * u64::from(u32::from(RAW_VALUE_MAX) * range) + round;

        // Check that all possible values can be represented.
        i32::try_from(((max_num / u64::from(RAW_VALUE_MAX)) as i64) + min_scaled)
            .map_err(|_| HytError::ScaleValueOutOfBounds)?;
        // The actual calculation is now infallible.
        Ok((((num / u64::from(RAW_VALUE_MAX)) as i64) + min_scaled) as i32)
    }

    /// Extract the raw 14-bit humidity reading.
    fn humidity_raw(&self) -> u16 {
        (((self.raw[0] & 0b0011_1111) as u16) << 8) | (self.raw[1] as u16)
    }

    /// Extract the raw 14-bit temperature reading.
    fn temperature_raw(&self) -> u16 {
        ((self.raw[2] as u16) << 6) | ((self.raw[3] as u16) >> 2)
    }
}
