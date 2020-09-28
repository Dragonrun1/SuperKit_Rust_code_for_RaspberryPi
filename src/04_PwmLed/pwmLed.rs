// MIT License
//
// Copyright © 2020-present, Michael Cummings <mgcummings@yahoo.com>.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use anyhow::{Context, Result};
use rppal::{
    gpio::{Gpio, OutputPin},
    system::DeviceInfo,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

const LED_PIN: u8 = 18;
const FREQUENCY: f64 = 1000.0;
const DELAY: u64 = 50;

fn main() -> Result<()> {
    println!(
        "04_PwmLed started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut led = setup()?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        println!("brighter ...");
        // Using inclusive end point for range.
        for i in (0..=100).step_by(4) {
            led.set_pwm_frequency(FREQUENCY, i as f64 / 100.0)
                .context("Failed to change duty cycle")?;
            sleep(Duration::from_millis(DELAY));
        }
        sleep(Duration::from_secs(1));
        println!("... dimmer");
        for i in (0..=100).rev().step_by(4) {
            led.set_pwm_frequency(FREQUENCY, i as f64 / 100.0)
                .context("Failed to change duty cycle")?;
            sleep(Duration::from_millis(DELAY));
        }
        sleep(Duration::from_secs(1));
    }
    println!("\n04_PwmLed stopped");
    Ok(())
}

fn setup() -> Result<OutputPin> {
    let gpio = Gpio::new().context("Failed to get GPIO instance")?;
    let mut led = gpio
        .get(LED_PIN)
        .context("Failed to get led pin")?
        .into_output();
    led.set_low();
    led.set_pwm_frequency(FREQUENCY, 0.0)
        .context("Failed to initialize PWM for led pin")?;
    Ok(led)
}
