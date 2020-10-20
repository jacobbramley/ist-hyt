//! An I²C driver for [Innovative Sensor Technology IST AG][IST]'s [HYT-series][HYT] temperature
//! and humidity sensor modules (HYT221, HYT271 and HYT939).
//!
//! These sensors have an "I²C-compatible" interface supporting bit rates up to 400kHz.
//!
//! This driver uses the I²C traits from [`embedded-hal`][hal_i2c], which currently only support
//! blocking accesses. To minimise blocking, each function in this crate executes at most one
//! transaction, the longest of which transfer four bytes.
//!
//! Note that I²C devices can [lock up the bus], preventing these blocking I²C functions from
//! returning. This crate cannot strictly guarantee that its blocking I²C functions will return at
//! all.
//!
//! [lock up the bus]: https://www.i2c-bus.org/i2c-primer/analysing-obscure-problems/blocked-bus/
//! [hal_i2c]: https://docs.rs/embedded-hal/0.2.4/embedded_hal/blocking/i2c/index.html
//! [IST]: https://www.ist-ag.com/
//! [HYT]: https://www.ist-ag.com/sites/default/files/AHHYTM_E.pdf
//!
//! # Examples
//!
//! ```
//! let i2c = ...;  // Some embedded-hal I²C device.
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
//!
//! # Status
//!
//! This crate is in early development and its API should be considered to be
//! unstable.
//!
//! Known issues:
//!
//! - Whilst the I²C interface is the same for the whole HYT family, this crate is
//!   only known to have been tested with the HYT221.
//! - Support for "command mode" is not yet implemented. Command mode is not
//!   required for normal operation, but allows configuration, for example, of the
//!   sensor's I²C address.
//! - There is not yet any support for non-blocking operations. To mitigate
//!   this, the `start_measurement()` and `read()` functions are separate, so that
//!   calling code can do other work whilst the sensor is busy. Note that the
//!   [embedded-hal] crate doesn't currently provide a non-blocking I²C API.
//! - Floating-point results are not supported at all, even on microcontrollers that
//!   can handle them.
//! - `cargo test` doesn't do anything useful at the moment.
//!
//! [embedded-hal]: https://docs.rs/embedded-hal/0.2.4/embedded_hal/

#![no_std]

mod error;
mod measurement;

/// Marker types used to represent the state of the sensor's interface.
pub mod mode {
    /// Normal mode, used for starting and retrieving measurements.
    pub struct Normal;

    /// Command mode, used for configuring the sensor.
    pub struct Command;
}

pub use error::Error;
pub use error::HytError;
pub use measurement::Measurement;

use core::marker::PhantomData;
use embedded_hal as hal;

/// The main sensor interface.
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

    /// Construct a new `Hyt` interface with the specified I²C address.
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
    /// supply. If you need to reliably enter command mode, some external logic will be required so
    /// that the sensor can be properly power-cycled.
    ///
    /// _**TODO**: Currently unimplemented._
    pub fn enter_command_mode(self) -> Result<Hyt<I2C, mode::Command>, (Self, Error<I2C>)> {
        todo!()
    }

    /// Start a measurement.
    ///
    /// According to the datasheet, it takes 60-100ms for the result to be ready, but in practice
    /// it is often ready after about 40ms.
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
    /// completed), the result will be [_stale_](./struct.Measurement.html#method.is_stale).
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
    /// Attempt to return to normal mode.
    ///
    /// _**TODO**: Currently unimplemented._
    pub fn enter_normal_mode(self) -> Result<Hyt<I2C, mode::Normal>, (Self, Error<I2C>)> {
        todo!()
    }
}
