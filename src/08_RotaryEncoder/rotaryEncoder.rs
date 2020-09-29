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
use rppal::gpio::Trigger;
use rppal::{
    gpio::{Gpio, InputPin, Level},
    system::DeviceInfo,
};
use std::{
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};
// Once again Python code is using global mutable state which doesn't really
// work well in Rust. Python code made decoding of encoder overly hard so
// changed to something simpler with only one piece of shared state for counter.

const DT_PIN: u8 = 17;
const CLK_PIN: u8 = 18;
const SW_PIN: u8 = 27;
const DELAY: u64 = 10;

fn main() -> Result<()> {
    println!(
        "08_RotaryEncoder started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let (clk, dt, mut sw) = setup()?;
    let counter = Arc::new(AtomicI32::new(0));
    let c = counter.clone();
    println!("counter = {}", c.load(Ordering::SeqCst));
    sw.set_async_interrupt(Trigger::FallingEdge, move |_: Level| {
        c.store(0, Ordering::SeqCst);
        println!("counter = {}", c.load(Ordering::SeqCst));
    })?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Initialize current clk as last clk.
    let mut last_clk = clk.read();
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        // Read the current pin values.
        let current_clk = clk.read();
        let current_dt = dt.read();
        // Check if clk has change value.
        if current_clk != last_clk {
            // If clk and dt aren't the same encoder was rotated clockwise else
            // rotated counter-clockwise.
            if current_dt != current_clk {
                counter.fetch_add(1, Ordering::SeqCst);
            } else {
                counter.fetch_add(-1, Ordering::SeqCst);
            }
            println!("counter = {}", counter.load(Ordering::SeqCst));
        }
        // Copy current clock value to last clock to use for next loop.
        last_clk = current_clk;
        sleep(Duration::from_millis(DELAY));
    }
    println!("\n08_RotaryEncoder stopped");
    Ok(())
}

fn setup() -> Result<(InputPin, InputPin, InputPin)> {
    let gpio = Gpio::new().context("Failed to get GPIO instance")?;
    let dt = gpio
        .get(DT_PIN)
        .context("Failed to get dt pin")?
        .into_input();
    let clk = gpio
        .get(CLK_PIN)
        .context("Failed to get clk pin")?
        .into_input();
    let sw = gpio
        .get(SW_PIN)
        .context("Failed to get sw pin")?
        .into_input_pullup();
    Ok((clk, dt, sw))
}
