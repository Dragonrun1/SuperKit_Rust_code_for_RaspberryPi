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

// Gpio pin numbers.
const PINS: [u8; 8] = [17, 18, 27, 22, 23, 24, 25, 4];
// Led on time in milliseconds.
const DELAY: u64 = 50;

fn main() -> Result<()> {
    println!(
        "03_8Led started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut leds = setup(PINS)?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        // Flash LEDs in sequence.
        println!("forward ...");
        for led in leds.iter_mut() {
            led.set_low();
            sleep(Duration::from_millis(DELAY));
            led.set_high();
        }
        // Flash LEDs in reverse sequence.
        println!("... reverse");
        for led in leds.iter_mut().rev() {
            led.set_low();
            sleep(Duration::from_millis(DELAY));
            led.set_high();
        }
    }
    println!("\n03_8Led stopped");
    Ok(())
}

fn setup(pins: [u8; 8]) -> Result<Vec<OutputPin>> {
    let gpoi = Gpio::new().context("Failed to get GPIO instance")?;
    let mut outputs = Vec::new();
    for pin in pins.iter() {
        let mut led = gpoi.get(*pin).context("Failed to get pin")?.into_output();
        led.set_high();
        outputs.push(led);
    }
    Ok(outputs)
}
