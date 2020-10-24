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
use hd44780_ntb::{DisplayMode, EntryMode, FunctionMode, GpioDriver, HD44780};
use linux_embedded_hal::{sysfs_gpio::Direction, Delay, Pin};
use rppal::system::DeviceInfo;
use std::{io::Write, thread::sleep, time::Duration};

// The 4 bit data bus pins.
const PIN_D4: u64 = 25;
const PIN_D5: u64 = 24;
const PIN_D6: u64 = 23;
const PIN_D7: u64 = 18;
// The control pins.
const PIN_E: u64 = 22;
const PIN_RS: u64 = 27;
// Message delay.
const DELAY: u64 = 2;
// Messages to be displayed.
const MESSAGES: [&str; 5] = [
    " LCD 1602 Test \n123456789ABCDEF",
    "   SUNFOUNDER \nHello World ! :)",
    "Welcome to --->\n  sunfounder.com",
    "May the Rust ...\n... be with you!",
    "Ferris says \"Hi\"\n   rust-lang.org",
];

fn main() -> Result<()> {
    println!(
        "13_LCD1602 started on a {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut lcd = setup()?;
    display_loop(&mut lcd)?;
    // lcd.return_home().context("Failed to home the display")?;
    println!("\n12_DotMatrix stopped");
    destroy()
}

/// Resets GPIO pins as inputs and releases them back to the OS.
fn destroy() -> Result<()> {
    let rs = Pin::new(PIN_RS);
    let e = Pin::new(PIN_E);
    rs.set_direction(Direction::In)
        .context("Failed to set direction on register select pin")?;
    e.set_direction(Direction::In)
        .context("Failed to set direction on enable pin")?;
    rs.unexport()
        .context("Failed to un-export register select pin")?;
    e.unexport().context("Failed to un-export enable pin")?;
    let mut pin: Pin;
    let pin_numbers = [PIN_D4, PIN_D5, PIN_D6, PIN_D7];
    for num in pin_numbers.iter() {
        pin = Pin::new(*num);
        pin.set_direction(Direction::In)
            .context(format!("Failed to set direction on data pin: {}", num))?;
        pin.unexport()
            .context(format!("Failed to export data pin: {}", num))?;
    }
    Ok(())
}

/// Main display loop for messages.
fn display_loop(lcd: &mut GpioDriver<Pin, Pin, Pin, Delay>) -> Result<()> {
    for _ in 0..3 {
        for message in MESSAGES.iter() {
            // First clear the display.
            lcd.clear_display().context("Failed to clear the display")?;
            let lines: Vec<&str> = message.split('\n').collect();
            println!("{}", lines[0]);
            lcd.write(lines[0].as_bytes())
                .context("Failed to write string")?;
            if lines.len() == 2 {
                // Write the second line.
                lcd.set_dd_ram_addr(0x40)
                    .context("Failed to move to second line")?;
                println!("{}", lines[1]);
                lcd.write(lines[1].as_bytes())
                    .context("Failed to write string")?;
            }
            // Wait a couple seconds so message can be seen.
            sleep(Duration::from_secs(DELAY));
        }
        println!();
    }
    Ok(())
}

/// Gets the GPIO pins from OS and setup LCD display.
fn setup() -> Result<GpioDriver<Pin, Pin, Pin, Delay>> {
    let rs = Pin::new(PIN_RS);
    let e = Pin::new(PIN_E);
    rs.export()
        .context("Failed to export register select pin")?;
    e.export().context("Failed to export enable pin")?;
    rs.set_direction(Direction::High)
        .context("Failed to set direction and level on register select pin")?;
    e.set_direction(Direction::Low)
        .context("Failed to set direction and level on enable pin")?;
    let mut data = Vec::<Pin>::new();
    let pin_numbers = [PIN_D4, PIN_D5, PIN_D6, PIN_D7];
    for num in pin_numbers.iter() {
        let pin = Pin::new(*num);
        pin.export()
            .context(format!("Failed to export data pin: {}", num))?;
        pin.set_direction(Direction::Out)
            .context(format!("Failed to set direction on data pin: {}", num))?;
        data.push(pin);
    }
    let mut lcd = GpioDriver::new(rs, e, data, Delay);
    let dc = Some(DisplayMode::DISPLAY_ON);
    let ems = Some(EntryMode::ENTRY_LEFT | EntryMode::ENTRY_SHIFT_CURSOR);
    let fm = Some(FunctionMode::LINES_2);
    lcd.init(fm, dc, ems)
        .context("Failed to initialize display instance")?;
    Ok(lcd)
}
