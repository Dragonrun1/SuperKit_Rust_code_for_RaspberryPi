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

const MOTOR_PIN1: u8 = 17;
const MOTOR_PIN2: u8 = 18;
const MOTOR_ENABLE: u8 = 27;
const DELAY: u64 = 5000;

fn main() -> Result<()> {
    println!(
        "07_Motor started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let (mut motor1, mut motor2, mut enable) = setup()?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    'outer: while running.load(Ordering::SeqCst) {
        println!("motor clockwise ...");
        motor1.set_high();
        motor2.set_low();
        enable.set_high();
        sleep(Duration::from_millis(DELAY));
        // Improves Ctrl-C responsiveness.
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        println!("stopped");
        enable.set_low();
        sleep(Duration::from_millis(DELAY));
        // Improves Ctrl-C responsiveness.
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        println!("motor counter-clockwise ...");
        motor1.set_low();
        motor2.set_high();
        enable.set_high();
        sleep(Duration::from_millis(DELAY));
        // Improves Ctrl-C responsiveness.
        if !running.load(Ordering::SeqCst) {
            break 'outer;
        }
        println!("stopped");
        enable.set_low();
        sleep(Duration::from_millis(DELAY));
    }
    enable.set_low();
    println!("\n07_Motor ended");
    Ok(())
}

fn setup() -> Result<(OutputPin, OutputPin, OutputPin)> {
    let gpio = Gpio::new().context("Failed to get GPIO instance")?;
    let mut enable = gpio
        .get(MOTOR_ENABLE)
        .context("Failed to get enable pin")?
        .into_output();
    enable.set_low();
    let motor1 = gpio
        .get(MOTOR_PIN1)
        .context("Failed to get motor1 pin")?
        .into_output();
    let motor2 = gpio
        .get(MOTOR_PIN2)
        .context("Failed to get motor2 pin")?
        .into_output();
    Ok((motor1, motor2, enable))
}
