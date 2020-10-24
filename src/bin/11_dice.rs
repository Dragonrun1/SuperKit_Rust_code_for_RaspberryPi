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

// Once again decided it would be better to have more idiomatic Rust by using
// methods (functions) on a structure. Generally it better to represent a
// hardware device as some kind of abstraction (object) that contain some state
// and provide ways to change that state. The Python code could have done the
// same but for whatever reason they chose not to.

use anyhow::{Context, Result};
use rand::{thread_rng, Rng};
use rppal::{
    gpio::{Gpio, InputPin},
    system::DeviceInfo,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};
use superkit_rust_code_for_raspberrypi::HC595;

const BUTTON: u8 = 22;
const DELAY: u64 = 10;
// Digits 1-6
const SEG_CODES: [u8; 6] = [0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d];

fn main() -> Result<()> {
    println!(
        "11_Dice started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    // Random number generator.
    let mut rng = thread_rng();
    let (button, mut hc595) = setup()?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    println!("Press button to roll ...");
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        // Flash numbers in sequence.
        for code in SEG_CODES.iter() {
            hc595.serial_in(*code);
            hc595.parallel_out();
            if button.is_low() {
                // New random number between 0 and 5 for index into SEG_CODES.
                // Also displayed for user after adding 1 to it.
                let num = rng.gen_range(0, 6);
                hc595.serial_in(SEG_CODES[num]);
                hc595.parallel_out();
                println!("number = {}", num + 1);
                sleep(Duration::from_secs(2));
            } else {
                sleep(Duration::from_millis(DELAY));
            }
        }
    }
    println!("\n11_Dice stopped");
    Ok(())
}

fn setup() -> Result<(InputPin, HC595)> {
    let hc595 = HC595::new()?;
    let gpio = Gpio::new().context("Failed to get GPIO instance")?;
    let button = gpio
        .get(BUTTON)
        .context("Failed to get button pin")?
        .into_input_pullup();
    Ok((button, hc595))
}
