use error::Error;
use std::fs::{File};
use std::io::Read;
use std::path::Path;
use std::{thread, time};

pub mod error;

fn main() {
    loop {
        match read_battery_charge() {
            Ok(p) => {
                println!("Power {}", p);
            }
            Err(_) => {
                println!("Failed");
            }
        }
        thread::sleep(time::Duration::from_millis(5000));
    }
}

pub fn read_battery_charge() -> Result<i8, Error> {
    if !Path::new("/sys/class/power_supply/BAT0/capacity").exists() {
        println!("Battery does not exist, did you seriously try running an application for managing battery charge on a computer that does not contain a battery.");
        return Err(Error::BatteryMissing);
    }

    let mut cap_str: String = String::new();
    File::open("/sys/class/power_supply/BAT0/capacity")?.read_to_string(&mut cap_str)?;

    // Remove the \n char
    cap_str.pop();

    Ok(cap_str.parse::<i8>().unwrap())
}
