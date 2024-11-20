extern crate system_shutdown;

use error::Error;
use notify_rust::{Notification, Urgency};
use std::fs::File;
use std::io::{stdin, BufRead, Read};
use std::matches;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::{thread, time};
use system_shutdown::shutdown;
use time::Duration;

const UPDATES_PER_SECOND: i32 = 8; // The number of times to update the shutdown notification per second

pub mod error;

// Charging may also signify the battery is full
pub enum PowerLevel {
    SHUTTING_DOWN,
    CRITICAL,
    LOW,
    NORMAL,
    CHARGING,
    UNKNOWN,
}

fn main() {
    let mut state: PowerLevel = PowerLevel::UNKNOWN;
    let (txx, rxx) = mpsc::channel(); // Default initialisation
    let mut tx: Sender<()> = txx;

    loop {
        match read_battery_charge() {
            Ok(mut p) => {
                match on_ac() {
                    Ok(mut on_ac) => {
                        // For testing purposes
                        //p = 2;

                        //if !on_ac {
                        //    if matches!(state, PowerLevel::CHARGING) {
                        //        println!("Computer is now on battery power");
                        //        Notification::new()
                        //            .summary("Computer is now on battery power")
                        //            .appname("Battery")
                        //            .urgency(Urgency::Normal)
                        //            .timeout(6900)
                        //            .show();
                        //    }
                        //}

                        //if p <= 5 && p > 3 && !matches!(state, PowerLevel::CRITICAL) && !on_ac {
                        //    Notification::new()
                        //        .summary("Battery Management")
                        //        .body("Warning battery charge is critical!")
                        //        .appname("Battery")
                        //        .urgency(Urgency::Critical)
                        //        .show();
                        //    state = PowerLevel::CRITICAL;
                        //    println!("Power state is now CRITICAL.");
                        //}
                        //if p <= 20 && p > 5 && !matches!(state, PowerLevel::LOW) && !on_ac {
                        //    Notification::new()
                        //        .summary("Battery Management")
                        //        .body("Warning battery charge is low!")
                        //        .appname("Battery")
                        //        .urgency(Urgency::Critical)
                        //        .timeout(9000)
                        //        .show();
                        //    state = PowerLevel::LOW;
                        //    println!("Power state is now LOW.");
                        //}

                        //if p > 20 && !matches!(state, PowerLevel::NORMAL) && !on_ac {
                        //    state = PowerLevel::NORMAL;
                        //    println!("Power state is now NORMAL.");
                        //}

                        if p <= 90 && !matches!(state, PowerLevel::SHUTTING_DOWN) && !on_ac {
                            state = PowerLevel::SHUTTING_DOWN;
                            println!("Shutdown initiated!");
                            tx = spawn_shutdown_task();
                        }

                        if on_ac {
                            if matches!(state, PowerLevel::SHUTTING_DOWN) {
                                tx.send(());
                            }
                            if !matches!(state, PowerLevel::CHARGING) {
                                state = PowerLevel::CHARGING;
                                println!("Computer is now on AC power");
                                Notification::new()
                                    .summary("Computer is now on AC power")
                                    .appname("Battery")
                                    .urgency(Urgency::Normal)
                                    .timeout(6900)
                                    .show();
                            }
                        }
                    }
                    Err(_) => {
                        println!("Failed to check if laptop is on battery");
                    }
                }
            }
            Err(_) => {
                println!("Failed to check battery charge level");
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

pub fn spawn_shutdown_task() -> Sender<()> {
    let (tx, rx) = mpsc::channel();

    let mut i = 60 * UPDATES_PER_SECOND;

    thread::spawn(move || loop {
        println!("Shutdown...{}", i);
        let percent_to_shutdown = ((i as f32)/(60.0 * UPDATES_PER_SECOND as f32))*100.0;
        let seconds_to_shutdown = i / UPDATES_PER_SECOND;
        println!("Percent: {}", percent_to_shutdown);
        Notification::new()
            .summary("Battery Management")
            .body(&format!(
                "Shutdown imminent in {} seconds to prevent damage to battery",
                seconds_to_shutdown
            ))
            .hint(notify_rust::Hint::CustomInt("value".to_string(), percent_to_shutdown.round() as i32))
            .hint(notify_rust::Hint::Custom("x-dunst-stack-tag".to_string(), "shutdown".to_string()))
            .appname("Battery")
            .icon({match (seconds_to_shutdown % 2 == 0) {
                true => { "/usr/share/icons/Papirus/16x16/panel/battery-000.svg" },
                _ => { "/usr/share/icons/Papirus/16x16/panel/battery-010.svg" }
            }})
            .urgency(Urgency::Critical)
            .timeout(1008)
            .show();

        if (i <= 0) {
            match shutdown() {
                Ok(_) => println!("Shutting down, bye!"),
                Err(error) => eprintln!("Failed to shut down: {}", error),
            }
        }

        i = i - 1;
        thread::sleep(Duration::from_millis(((1000 / UPDATES_PER_SECOND)) as u64));
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                println!("Terminating shutdown thread.");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    });

    let mut line = String::new();
    let stdin = stdin();

    tx
}

pub fn on_ac() -> Result<bool, Error> {
    let mut path = "AC0";
    if !Path::new("/sys/class/power_supply/AC0/online").exists() {
        if !Path::new("/sys/class/power_supply/AC/online").exists() {
            println!("Unexpected, the directory /sys/class/power_supple/(Neither AC nor AC0)/online doesn't exist? Do you not have a power source?");
            return Ok(true);
        } else {
            path = "AC";
        }
    }

    let mut pwr_str: String = String::new();
    File::open(format!("/sys/class/power_supply/{}/online", path))?.read_to_string(&mut pwr_str)?;

    // Remove the \n char
    pwr_str.pop();

    return Ok(pwr_str == "1");
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
