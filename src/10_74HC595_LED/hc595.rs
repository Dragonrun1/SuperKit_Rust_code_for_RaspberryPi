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
use rppal::system::DeviceInfo;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};
use superkit_rust_code_for_raspberrypi::HC595;

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
