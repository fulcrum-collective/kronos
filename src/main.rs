// src/main.rs
mod task;

use crate::task::Task;
use chrono::{Local, NaiveDateTime};
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    println!("[Kronos] Reading task configuration...");

    let config_path = "task.toml";
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading config file '{}': {}", config_path, e);
            return;
        }
    };

    let task: Task = match toml::from_str(&config_content) {
        Ok(task) => task,
        Err(e) => {
            eprintln!("Error parsing TOML: {}", e);
            return;
        }
    };

    println!("[Kronos] Task loaded: '{}'", task.job.description);

    let trigger_time =
    match NaiveDateTime::parse_from_str(&task.trigger.on_calendar, "%Y-%m-%d %H:%M:%S") {
        Ok(time) => time,
        Err(e) => {
            eprintln!("Error parsing 'on_calendar' time: {}", e);
            return;
        }
    };

    println!("[Kronos] Task scheduled to run at: {}", trigger_time);

    loop {
        let now = Local::now().naive_local();
        if now >= trigger_time {
            println!("[Kronos] Trigger time reached! Executing command...");

            let output = Command::new("sh")
            .arg("-c")
            .arg(&task.job.command)
            .output();

            match output {
                Ok(out) => {
                    println!("[Kronos] Command finished.");

                    if !out.stdout.is_empty() {
                        println!("--- stdout ---\n{}", String::from_utf8_lossy(&out.stdout));
                    }
                    if !out.stderr.is_empty() {
                        eprintln!("--- stderr ---\n{}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                }
            }

            break;
        }

        thread::sleep(Duration::from_secs(1));
    }

    println!("[Kronos] Task complete. Shutting down.");
}
