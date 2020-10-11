# IST HYT Sensor Driver

An I²C driver for [IST]'s [HYT-series][HYT] temperature and humidity sensor
modules (HYT221, HYT271 and HYT939).

[IST]: https://www.ist-ag.com/
[HYT]: https://www.ist-ag.com/sites/default/files/AHHYTM_E.pdf

## Minimal example

```
let i2c = ...;  // Some embedded_hal I2C device.
let hyt = ist_hyt::Hyt::new(i2c).start_measurement().unwrap();
let measurement = loop {
    // Poll until the measurement is ready.
    let m = hyt.read().unwrap();
    if !m.is_stale() {
        break m;
    }
};
let humidity = measurement.humidity();
let temperature = measurement.temperature();
```

## Measurement formats

For simplicity, the basic `humidity()` and `temperature()` functions return
integer results. However, the HYT-series sensors provide measurements with
useful resolution significantly smaller than 1°C or 1%RH. Most microcontrollers
lack floating-point capabilities, so this crate provides two separate mechanisms
for increasing the precision of the result:

- Values scaled by a user-defined constant (e.g. `temperature_scaled(100)`).
- Fixed-point ([I8F24]) values (e.g. `temperature_i8f24()`) based on the [fixed]
  crate, but only if the `i8f24` feature is enabled.

[I8F24]: https://docs.rs/fixed/1.1.0/fixed/types/type.I8F24.html
[fixed]: https://crates.io/crates/fixed

## Status

This crate is in early development and its API should be considered to be
unstable.

Known issues:

- Whilst the I²C interface is the same for the whole HYT family, this crate is
  only known to have been tested with the HYT271.
- Support for "command mode" is not yet unimplemented. Command mode is not
  required for normal operation, but allows configuration, for example, of the
  sensor's I²C address.
- There is not yet any support for non-blocking operations. To mitigate
  this, the `start\_measurement()` and `read()` functions are separate, so that
  calling code can do other work whilst the sensor is busy. Note that the
  [embedded-hal] crate doesn't currently provide a non-blocking I²C API.
- Floating-point results are not supported at all, even on microcontrollers that
  can handle them.
- `cargo test` doesn't do anything useful at the moment.

[embedded-hal]: https://crates.io/crates/embedded-hal

## Licence

This project is licensed under the terms of the MIT Licence. See [LICENCE] for
the licence text.

[LICENCE]: LICENCE
