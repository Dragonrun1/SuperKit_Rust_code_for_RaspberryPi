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
    gpio::{Gpio, InputPin, OutputPin},
    system::DeviceInfo,
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

const BTN_PIN: u8 = 18;
const LED_PIN: u8 = 17;

fn main() -> Result<()> {
    println!(
        "02_BtnAndLed started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let (button, mut led) = setup()?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        // Like the C code the button acts as a momentary switch with no latching.
        if button.is_high() {
            println!("led off ...");
            led.set_high();
        } else {
            println!("... led on");
            led.set_low();
        }
        // Acts as a crude form of debounce.
        sleep(Duration::from_millis(200));
    }
    println!("\n02_BtnAndLed stopped");
    Ok(())
}

fn setup() -> Result<(InputPin, OutputPin)> {
    let gpio = Gpio::new().context("Failed to get GPIO instance")?;
    let button = gpio
        .get(BTN_PIN)
        .context("Failed to get button pin")?
        .into_input_pullup();
    let mut led = gpio
        .get(LED_PIN)
        .context("Failed to get led pin")?
        .into_output();
    led.set_high();
    Ok((button, led))
}
