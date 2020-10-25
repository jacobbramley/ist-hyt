//! Log measurements over [RTT].
//!
//! This run on an LPC824 ("824m201jdh20") with the sensor connected to the preferred (true
//! open-drain) I²C pins, P0_10 and P0_11.
//!
//! [RTT]: https://crates.io/crates/rtt-target

#![no_main]
#![no_std]

use ist_hyt::Hyt;
use lpc8xx_hal::{delay::Delay, i2c, prelude::*, CorePeripherals, Peripherals};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let cp = CorePeripherals::take().unwrap();
    let p = Peripherals::take().unwrap();
    let mut swm = p.SWM.split();
    let mut syscon = p.SYSCON.split();

    let (scl, _) = swm
        .fixed_functions
        .i2c0_scl
        .assign(p.pins.pio0_10.into_swm_pin(), &mut swm.handle);
    let (sda, _) = swm
        .fixed_functions
        .i2c0_sda
        .assign(p.pins.pio0_11.into_swm_pin(), &mut swm.handle);
    let i2c0 = p
        .I2C0
        .enable(&(), scl, sda, &mut syscon.handle)
        .enable_master_mode(&i2c::Clock::new_400khz());
    let mut delayer = Delay::new(cp.SYST);

    let mut hyt = Hyt::new(i2c0.master);

    const DELAY_REPEAT_MS: u16 = 1000;
    loop {
        rprintln!("Starting measurement...");
        hyt.start_measurement()
            .expect("Failed to start measurement.");
        // The measurement is specified to take 60-100ms, but it's often ready before that. For
        // demonstration purposes, we'll poll quite aggressively.
        const DELAY_START_MS: u16 = 30;
        const DELAY_STEP_MS: u16 = 1;
        delayer.delay_ms(DELAY_START_MS);
        let mut count = 0;
        let m = loop {
            // Poll until the measurement is ready.
            let m = hyt.read().expect("Failed to read measurement.");
            if !m.is_stale() {
                break m;
            }
            count += 1;
            // Real code should provide a timeout mechanism; this example will enter an infinite
            // loop if the sensor becomes unresponsive whilst we're waiting for a result.
            delayer.delay_ms(DELAY_STEP_MS);
        };
        rprintln!(
            "Measurements retrieved after approximately {} ms.",
            DELAY_START_MS + DELAY_STEP_MS * count
        );

        // Integer (rounded) results are available for convenience.
        let (t_rounded, h_rounded) = (m.temperature(), m.humidity());

        // Scaled results are useful for formatting decimal results with minimal code size
        // overhead.
        let t_scaled = m.temperature_scaled(100).unwrap();
        let t_int = t_scaled / 100;
        let t_frac = t_scaled % 100;
        let h_scaled = m.humidity_scaled(100).unwrap();
        let h_int = h_scaled / 100;
        let h_frac = h_scaled % 100;

        // Fixed-point results are convenient to work with, but require an external dependency, and
        // the implementation (mainly for `fmt`) requires about 4kB of code, which is all of the
        // Flash memory on some microcontrollers.
        #[cfg(feature = "i8f24")]
        let (t_fixed, h_fixed) = (m.temperature_i8f24(), m.humidity_i8f24());

        rprintln!("      Temperature (rounded): {} °C", t_rounded);
        rprintln!("       Temperature (scaled): {}.{:02} °C", t_int, t_frac);
        #[cfg(feature = "i8f24")]
        rprintln!("  Temperature (fixed-point): {:.2} °C", t_fixed);

        rprintln!("         Humidity (rounded): {} %RH", h_rounded);
        rprintln!("          Humidity (scaled): {}.{:02} %RH", h_int, h_frac);
        #[cfg(feature = "i8f24")]
        rprintln!("     Humidity (fixed-point): {:.2} %RH", h_fixed);

        delayer.delay_ms(DELAY_REPEAT_MS);
    }
}
