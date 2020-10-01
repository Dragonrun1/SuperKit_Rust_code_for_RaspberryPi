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
    gpio::{Gpio, InputPin, OutputPin},
    system::DeviceInfo,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

const SDI: u8 = 17;
const RCLK: u8 = 18;
const SRCLK: u8 = 27;
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
    let mut rng = thread_rng();
    let mut hc595 = HC595::new(SDI, RCLK, SRCLK, BUTTON)?;
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
            if hc595.button.is_low() {
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

pub struct HC595 {
    sdi: OutputPin,
    rclk: OutputPin,
    srclk: OutputPin,
    // Not strictly part of 75HC595 but easier to manage button by including
    // with the rest of the pins.
    button: InputPin,
}

impl HC595 {
    /// Takes place of setup() from Python code.
    pub fn new(sdi: u8, rclk: u8, srclk: u8, button: u8) -> Result<Self> {
        let gpio = Gpio::new().context("Failed to get GPIO instance")?;
        let mut sdi = gpio
            .get(sdi)
            .context("Failed to get sdi pin")?
            .into_output();
        sdi.set_low();
        let mut rclk = gpio
            .get(rclk)
            .context("Failed to get rclk pin")?
            .into_output();
        rclk.set_low();
        let mut srclk = gpio
            .get(srclk)
            .context("Failed to get srclk pin")?
            .into_output();
        srclk.set_low();
        // Not strictly part of 75HC595.
        let button = gpio
            .get(button)
            .context("Failed to get button pin")?
            .into_input_pullup();
        Ok(HC595 {
            sdi,
            rclk,
            srclk,
            button,
        })
    }
    /// Some function as hc595_in() from Python code.
    pub fn serial_in(&mut self, data: u8) {
        // Switch from bit shifting data around to iterating pre-calculated mask
        // values.
        for mask in ([0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01]).iter() {
            if data & mask > 0 {
                self.sdi.set_high();
            } else {
                self.sdi.set_low();
            }
            // Strobe shift register clock.
            self.srclk.set_high();
            sleep(Duration::from_micros(1));
            self.srclk.set_low();
        }
    }
    /// Same as hc595_out() function from Python code.
    pub fn parallel_out(&mut self) {
        // Strobe output latch clock.
        self.rclk.set_high();
        sleep(Duration::from_micros(1));
        self.rclk.set_low();
    }
}

/// Insure output on 75HC595 is all zero (off) before exiting.
impl Drop for HC595 {
    fn drop(&mut self) {
        self.serial_in(0);
        self.parallel_out();
        self.sdi.set_low();
        self.rclk.set_low();
        self.srclk.set_low();
    }
}
