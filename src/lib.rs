//! IST HYT221 / HYT271 / HYT939 humidity and temperature sensor driver.
//!
//! These sensors have an "I²C-compatible" interface supporting bit rates up to 400kHz.
//!
//! This driver uses the I²C traits from `embedded_hal`, which currently only support blocking
//! accesses. Most member functions execute at most one transaction, the longest of which have four
//! bytes.
//!
//! # Examples
//!
//! ```
//! let hyt = ist_hyt::Hyt::new(i2c).start_measurement().unwrap();
//! ... // Wait
//! // The measurement is specified to take 60-100ms, but empirically, it's often ready before
//! // that (e.g. ~40ms). The optimal time to start polling, and the interval between polls,
//! // depends on the application.
//! let measurement = loop {
//!     // Poll until the measurement is ready.
//!     let m = hyt.read().unwrap();
//!     if !m.is_stale() {
//!         break m;
//!     }
//!     // Real code should provide a timeout mechanism; this example will enter an infinite
//!     // loop if the sensor becomes unresponsive whilst we're waiting for a result.
//!     ...
//! };
//! let humidity = measurement.humidity();
//! let temperature = measurement.temperature();
//! ```

#![no_std]
pub mod error;
pub mod measurement;
pub mod mode;

pub use error::Error;
pub use measurement::Measurement;

use core::marker::PhantomData;
use embedded_hal as hal;

pub struct Hyt<I2C, Mode>
where
    I2C: hal::blocking::i2c::Read + hal::blocking::i2c::Write,
{
    _mode: PhantomData<Mode>,
    i2c: I2C,
    address: u8,
}

impl<I2C> Hyt<I2C, mode::Normal>
where
    I2C: hal::blocking::i2c::Read + hal::blocking::i2c::Write,
{
    /// Construct a new `Hyt` interface with the factory default I²C address (0x28).
    pub fn new(i2c: I2C) -> Self {
        Self {
            _mode: PhantomData,
            i2c,
            address: 0x28,
        }
    }

    /// Set the I²C address explicitly.
    pub fn with_address(self, address: u8) -> Self {
        Self { address, ..self }
    }

    /// Attempt to enter command mode.
    ///
    /// This consumes `self` in order to enforce the (rather simple) state machine. On success, a
    /// new `Hyt` instance is returned, in command mode. On failure, `self` is returned again (in
    /// normal mode).
    ///
    /// Command mode can only be entered within 10ms of the sensor being powered on. The sensor may
    /// be powered on for some time before the MCU reset. For example, most debuggers and
    /// programmers reset the MCU using a dedicated nRESET pin, without interrupting the power
    /// supply. If you need to _reliably_ enter command mode, some external logic will be required
    /// so that the sensor can be properly power-cycled.
    ///
    /// _**TODO**: Currently unimplemented._
    pub fn enter_command_mode(self) -> Result<Hyt<I2C, mode::Command>, (Self, Error<I2C>)> {
        todo!()
    }

    /// Start a measurement.
    ///
    /// According to the datasheet, it takes 60-100ms for the result to be ready, though it is
    /// possible to poll the sensor if latency is important.
    pub fn start_measurement(&mut self) -> Result<(), Error<I2C>> {
        // "MR (Measurement Request)"
        // This is a simple I²C write, but with no data.
        // TODO: If we haven't already read the last measurement, read it now, otherwise it won't
        // appear stale and we won't be able to tell when this measurement is done.
        self.i2c
            .write(self.address, &[])
            .map_err(Error::<I2C>::I2CWrite)
    }

    /// Read the most recent measurement from the sensor.
    ///
    /// If it has already been read (for example because a recently-started measurement has not yet
    /// completed), the result will be "stale".
    pub fn read(&mut self) -> Result<Measurement, Error<I2C>> {
        // "DF (Data Fetch)"
        // We will read four bytes from the sensor.
        // TODO: Add support for abandoning stale reads after the first byte, or reading just the
        // humidity result.
        let mut raw = [0u8; 4];
        self.i2c
            .read(self.address, &mut raw)
            .map_err(Error::<I2C>::I2CRead)?;
        Ok(Measurement::from_raw(raw)?)
    }
}

impl<I2C> Hyt<I2C, mode::Command>
where
    I2C: hal::blocking::i2c::Read + hal::blocking::i2c::Write,
{
    pub fn enter_normal_mode(self) -> Result<Hyt<I2C, mode::Normal>, (Self, Error<I2C>)> {
        todo!()
    }
}
