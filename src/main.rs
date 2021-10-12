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
                    Ok(mut b) => {
                        // For testing purposes
                        //p = 2;

                        if !b {
                            if matches!(state, PowerLevel::CHARGING) {
                                println!("Computer is now on battery power");
                                Notification::new()
                                    .summary(
                                        "Computer is now on battery power"
                                    )
                                    .appname("Battery")
                                    .urgency(Urgency::Normal)
                                    .timeout(6900)
                                    .show();
                            }
                        }

                        if p <= 5 && p > 3 && !matches!(state, PowerLevel::CRITICAL) && !b {
                            Notification::new()
                                .summary("Warning battery charge is critical!")
                                .appname("Battery")
                                .urgency(Urgency::Critical)
                                .show();
                            state = PowerLevel::CRITICAL;
                            println!("Power state is now CRITICAL.");
                        }
                        if p <= 20 && p > 5 && !matches!(state, PowerLevel::LOW) && !b {
                            Notification::new()
                                .summary("Warning battery charge is low!")
                                .appname("Battery")
                                .urgency(Urgency::Critical)
                                .show();
                            state = PowerLevel::LOW;
                            println!("Power state is now LOW.");
                        }

                        if p > 20 && !matches!(state, PowerLevel::NORMAL) && !b {
                            state = PowerLevel::NORMAL;
                            println!("Power state is now NORMAL.");
                        }

                        if p <= 3 && !matches!(state, PowerLevel::SHUTTING_DOWN) && !b {
                            state = PowerLevel::SHUTTING_DOWN;
                            println!("Shutdown initiated!");
                            tx = spawn_shutdown_task();
                        }

                        if b {
                            if matches!(state, PowerLevel::SHUTTING_DOWN) {
                                tx.send(());
                            }
                            if !matches!(state, PowerLevel::CHARGING) {
                                state = PowerLevel::CHARGING;
                                println!("Computer is now on AC power");
                                Notification::new()
                                    .summary(
                                        "Computer is now on AC power"
                                    )
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

    let mut i = 60;

    thread::spawn(move || loop {
        println!("Shutdown...{}", i);
        Notification::new()
            .summary(&format!(
                "Low power shutdown iminent in {} to prevent damage to battery",
                i
            ))
            .appname("Battery")
            .urgency(Urgency::Critical)
            .timeout(1009)
            .show();

        if (i <= 0) {
            match shutdown() {
                Ok(_) => println!("Shutting down, bye!"),
                Err(error) => eprintln!("Failed to shut down: {}", error),
            }
        }

        i = i - 1;
        thread::sleep(Duration::from_millis(1000));
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
