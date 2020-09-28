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

// Since the Python code went with global variables everywhere and the C code is
// doing its own very different thing here I've decided it time to show a much
// more idiomatic Rust way of doing things.
// I'm introducing a structure with an implantation to contain what up to now
// would have been just global scope functions.
// The constants have been left in global scope as there is no real benefit to
// doing something different with them.

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

const COLORS: [u32; 55] = [
    0x000000, 0x3F0000, 0x7F0000, 0xBF0000, 0xFF0000, // brighten red
    0xFF0000, 0xBF3F00, 0x7F7F00, 0x3FBF00, 0x00FF00, // fade to green
    0x00FF00, 0x00BF00, 0x007F00, 0x003F00, 0x000000, // dim green
    0x000000, 0x003F00, 0x007F00, 0x00BF00, 0x00FF00, // brighten green
    0x00FF00, 0x00BF3F, 0x007F7F, 0x003FBF, 0x0000FF, // fade to blue
    0x0000FF, 0x0000BF, 0x00007F, 0x00003F, 0x000000, // dim blue
    0x000000, 0x00003F, 0x00007F, 0x0000BF, 0x0000FF, // brighten blue
    0x0000FF, 0x3F00BF, 0x7F007F, 0xBF003F, 0xFF0000, // fade to red
    0xFF0000, 0xBF0000, 0x7F0000, 0x3F0000, 0x000000, // dim red
    0x000000, 0x3F3F3F, 0x7F7F7F, 0xBFBFBF, 0xFFFFFF, // brighten white
    0xFFFFFF, 0xBFBFBF, 0x7F7F7F, 0x3F3F3F, 0x000000, // dim white
];
const DELAY: u64 = 500;
const FREQUENCY: f64 = 2000.0;
// Gpio pin numbers.
const PINS: [u8; 3] = [17, 18, 27];

pub struct RgbPwm {
    red: OutputPin,
    green: OutputPin,
    blue: OutputPin,
}

impl RgbPwm {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new().context("Failed to get GPIO instance")?;
        let mut red = gpio
            .get(PINS[0])
            .context("Failed to get red LED")?
            .into_output();
        red.set_high();
        red.set_pwm_frequency(FREQUENCY, 0.0)
            .context("Failed to initialize PWM for red LED")?;
        let mut green = gpio
            .get(PINS[1])
            .context("Failed to get green LED")?
            .into_output();
        green.set_high();
        green
            .set_pwm_frequency(FREQUENCY, 0.0)
            .context("Failed to initialize PWM for green LED")?;
        let mut blue = gpio
            .get(PINS[2])
            .context("Failed to get blue LED")?
            .into_output();
        blue.set_high();
        blue.set_pwm_frequency(FREQUENCY, 0.0)
            .context("Failed to initialize PWM for blue LED")?;
        Ok(RgbPwm { red, green, blue })
    }
    fn scale(x: u32) -> f64 {
        // (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
        // Better (more accurate) to just pre-calculate multiplier where minimums
        // are all zero.
        x as f64 * 3.92156862745098e-3f64
    }
    pub fn set_color(&mut self, color: u32) -> Result<()> {
        // Extract each value from given color.
        // Showing explicit type info only on the first variable.
        let red: u32 = (color & 0xFF0000) >> 16;
        let green = (color & 0x00FF00) >> 8;
        let blue = color & 0x0000FF;
        // Scale from 0-255 range to 0-100 duty cycle.
        // Showing explicit type info only on the first shadow variable.
        let red: f64 = RgbPwm::scale(red);
        let green = RgbPwm::scale(green);
        let blue = RgbPwm::scale(blue);
        // Set the new duty cycles.
        self.red
            .set_pwm_frequency(FREQUENCY, red)
            .context("Failed to change red duty cycle")?;
        self.green
            .set_pwm_frequency(FREQUENCY, green)
            .context("Failed to change green duty cycle")?;
        self.blue
            .set_pwm_frequency(FREQUENCY, blue)
            .context("Failed to change blue duty cycle")?;
        Ok(())
    }
}

fn main() -> Result<()> {
    println!(
        "05_RGB started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut leds: RgbPwm = RgbPwm::new()?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    // Loop until Ctrl-C is received.
    'outer: while running.load(Ordering::SeqCst) {
        for color in COLORS.iter() {
            println!("color = {:#08X?}", color);
            leds.set_color(*color)?;
            sleep(Duration::from_millis(DELAY));
            // Improves Ctrl-C responsiveness.
            if !running.load(Ordering::SeqCst) {
                break 'outer;
            }
        }
        sleep(Duration::from_secs(1));
    }
    println!("\n05_RGB stopped");
    Ok(())
}
