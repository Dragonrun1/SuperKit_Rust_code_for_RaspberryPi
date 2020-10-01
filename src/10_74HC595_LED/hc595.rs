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

// Once again decided it would be better to some more idiomatic Rust by using
// methods (functions) on a structure. Generally it better to represent a
// hardware device as some kind of abstraction (object) that contain some state
// and provide ways to change that state. The Python code could have done the
// same but for whatever reason they chose not to.

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

const SDI: u8 = 17;
const RCLK: u8 = 18;
const SRCLK: u8 = 27;
const DELAY: u64 = 100;
// Use a two dimensional array to hold several sequences of LED modes.
const MODES: [[u8; 8]; 4] = [
    [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80], // original mode
    [0x01, 0x03, 0x07, 0x0f, 0x1f, 0x3f, 0x7f, 0xff], // blink mode 1
    [0x01, 0x05, 0x15, 0x55, 0xb5, 0xf5, 0xfb, 0xff], // blink mode 2
    [0x02, 0x03, 0x0b, 0x0f, 0x2f, 0x3f, 0xbf, 0xff], // blink mode 3
];

fn main() -> Result<()> {
    println!(
        "10_74HC595_LED started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut hc595 = HC595::new()?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    'outer: while running.load(Ordering::SeqCst) {
        // Unlike the Python code this code cycles through all the mode patterns.
        for (row, mode) in MODES.iter().enumerate() {
            println!("mode = {}", row);
            println!("forward ...");
            for data in mode.iter() {
                hc595.serial_in(*data);
                hc595.parallel_out();
                sleep(Duration::from_millis(DELAY));
            }
            // Improves Ctrl-C responsiveness.
            if !running.load(Ordering::SeqCst) {
                break 'outer;
            }
            sleep(Duration::from_millis(DELAY));
            println!("... reverse");
            for data in mode.iter().rev() {
                hc595.serial_in(*data);
                hc595.parallel_out();
                sleep(Duration::from_millis(DELAY));
            }
        }
    }
    println!("\n10_74HC595_LED stopped");
    Ok(())
}

pub struct HC595 {
    sdi: OutputPin,
    rclk: OutputPin,
    srclk: OutputPin,
}

impl HC595 {
    /// Takes place of setup() from Python code.
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new().context("Failed to get GPIO instance")?;
        let mut sdi = gpio
            .get(SDI)
            .context("Failed to get sdi pin")?
            .into_output();
        sdi.set_low();
        let mut rclk = gpio
            .get(RCLK)
            .context("Failed to get rclk pin")?
            .into_output();
        rclk.set_low();
        let mut srclk = gpio
            .get(SRCLK)
            .context("Failed to get srclk pin")?
            .into_output();
        srclk.set_low();
        Ok(HC595 { sdi, rclk, srclk })
    }
    /// Some function as hc595_in() from Python code.
    pub fn serial_in(&mut self, data: u8) {
        // Switch from bit shifting data around to iterating pre-calculated mask
        // values.
        for mask in ([0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80]).iter() {
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
    }
}
