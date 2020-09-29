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
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

const SIG_PIN: u8 = 17;
const DELAY: u64 = 50;

fn main() -> Result<()> {
    println!(
        "09_timer555 started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut sig = setup()?;
    // Used to access counter in main().
    let counter = Arc::new(AtomicU64::new(0));
    // Used in interrupt callback function to update counter.
    let c = counter.clone();
    // Declare an anonymous closure (function) that acts like the count() from
    // the Python code.
    let count = move |_: Level| {
        c.fetch_add(1, Ordering::SeqCst);
    };
    sig.set_async_interrupt(Trigger::RisingEdge, count)?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        println!("counter = {}", counter.load(Ordering::SeqCst));
        sleep(Duration::from_millis(DELAY));
    }
    println!("\n09_timer555 stopped");
    Ok(())
}

fn setup() -> Result<InputPin> {
    let gpio = Gpio::new().context("Failed to get GPIO instance")?;
    let sig = gpio
        .get(SIG_PIN)
        .context("Failed to get led pin")?
        .into_input_pullup();
    Ok(sig)
}
