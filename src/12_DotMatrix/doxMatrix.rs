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

const CODE_H: [u8; 20] = [
    0x01, 0xff, 0x80, 0xff, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff,
];
const CODE_L: [u8; 20] = [
    0x00, 0x7f, 0x00, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfe, 0xfd, 0xfb, 0xf7,
    0xef, 0xdf, 0xbf, 0x7f,
];

fn main() -> Result<()> {
    println!(
        "12_DotMatrix started on a {}",
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
        println!("forward ...");
        for i in 0..CODE_H.len() {
            hc595.serial_in(CODE_L[i]);
            hc595.serial_in(CODE_H[i]);
            hc595.parallel_out();
            sleep(Duration::from_millis(DELAY));
        }
        // Improves Ctrl-C responsiveness.
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        println!("... reverse");
        for i in (0..CODE_H.len()).rev() {
            hc595.serial_in(CODE_L[i]);
            hc595.serial_in(CODE_H[i]);
            hc595.parallel_out();
            sleep(Duration::from_millis(DELAY));
        }
        sleep(Duration::from_millis(DELAY));
    }
    println!("\n12_DotMatrix stopped");
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
