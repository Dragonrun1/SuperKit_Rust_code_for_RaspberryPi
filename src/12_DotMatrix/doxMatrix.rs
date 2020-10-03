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
use rppal::system::DeviceInfo;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};
use superkit_rust_code_for_raspberrypi::HC595;

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
