// src/main.rs

// Declares the task module, which will be loaded from src/task.rs.
mod task;

// Imports necessary items from other modules and external crates.
use crate::task::{Job, Task};
use chrono::{Duration as ChronoDuration, Local, NaiveDateTime, Timelike};
use log::{error, info, warn};
use std::{fs, path::Path, process::Command, thread, time::Duration as StdDuration};

// A struct to hold a scheduled job's details and its next run time.
#[derive(Debug, Clone)]
struct ScheduledJob {
    job: Job,
    trigger: task::Trigger,
    next_run_at: NaiveDateTime,
}

fn main() {
    // Initializes the logger. Log level can be controlled via RUST_LOG environment variable.
    env_logger::init();

    info!("Kronos v0.2 starting up...");

    // Defines the directory for task configuration files.
    let tasks_dir = "/etc/kronos/tasks.d";
    if !Path::new(tasks_dir).exists() {
        info!("'{tasks_dir}' directory not found. Creating it.");
        if let Err(e) = fs::create_dir_all(tasks_dir) {
            // Log the error and exit gracefully if the essential directory cannot be created.
            error!("Failed to create tasks directory '{tasks_dir}': {e}");
            // Exits with a non-zero status code to indicate failure.
            std::process::exit(1);
        }
    }

    let mut scheduled_jobs = load_and_schedule_tasks(tasks_dir);

    if scheduled_jobs.is_empty() {
        warn!("No tasks found. Kronos will idle.");
    }

    // The main daemon loop, responsible for checking and executing tasks.
    loop {
        // Sorts jobs by their next run time to find the soonest one.
        scheduled_jobs.sort_by_key(|j| j.next_run_at);

        // Gets the current time, truncated to second precision for consistency.
        let now = Local::now().naive_local().with_nanosecond(0).unwrap();

        // Processes all jobs that are due to run at or before the current time.
        while let Some(next_job) = scheduled_jobs.first() {
            if now >= next_job.next_run_at {
                let mut job_to_run = scheduled_jobs.remove(0);
                let description = &job_to_run.job.description;
                let next_run_at = &job_to_run.next_run_at;
                info!("Trigger time reached! Executing: '{description}' (Scheduled for: {next_run_at})");

                // Executes the command in a separate thread to avoid blocking the scheduler.
                let job_clone = job_to_run.job.clone();
                thread::spawn(move || {
                    execute_command(&job_clone);
                });

                // Reschedules the job if it is a recurring task.
                if let Some(every_str) = &job_to_run.trigger.every {
                    if let Some(duration) = parse_duration(every_str) {
                        job_to_run.next_run_at += duration;
                        let description = &job_to_run.job.description;
                        let next_run_at = &job_to_run.next_run_at;
                        info!(
                            "Rescheduling recurring task '{description}' to run at {next_run_at}"
                        );
                        scheduled_jobs.push(job_to_run);
                    }
                }
            } else {
                break;
            }
        }

        // Determines the duration to sleep before the next check.
        let sleep_duration = if let Some(next_job) = scheduled_jobs.first() {
            let duration_to_next = next_job.next_run_at.signed_duration_since(now);
            if duration_to_next > ChronoDuration::zero() {
                duration_to_next
                    .to_std()
                    .unwrap_or(StdDuration::from_secs(60))
            } else {
                StdDuration::from_secs(1)
            }
        } else {
            StdDuration::from_secs(3600)
        };

        let final_sleep = std::cmp::min(sleep_duration, StdDuration::from_secs(60));
        thread::sleep(final_sleep);
    }
}

// Loads all .toml files from a directory, parses them, and calculates their initial run times.
fn load_and_schedule_tasks(dir: &str) -> Vec<ScheduledJob> {
    let mut jobs = Vec::new();
    let now = Local::now().naive_local().with_nanosecond(0).unwrap();

    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                let path = match entry {
                    Ok(e) => e.path(),
                    Err(_) => continue,
                };

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    info!("Loading task from: {path:?}");
                    let content = match fs::read_to_string(&path) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to read task file {path:?}: {e}");
                            continue;
                        }
                    };

                    let task: Task = match toml::from_str(&content) {
                        Ok(t) => t,
                        Err(e) => {
                            error!("Failed to parse task file {path:?}: {e}");
                            continue;
                        }
                    };

                    let next_run_at = if let Some(cal_str) = &task.trigger.on_calendar {
                        NaiveDateTime::parse_from_str(cal_str, "%Y-%m-%d %H:%M:%S").ok()
                    } else if let Some(every_str) = &task.trigger.every {
                        parse_duration(every_str).map(|d| now + d)
                    } else {
                        None
                    };

                    if let Some(run_time) = next_run_at {
                        jobs.push(ScheduledJob {
                            job: task.job.clone(),
                            trigger: task.trigger.clone(),
                            next_run_at: run_time,
                        });
                    } else {
                        warn!("Task in {path:?} has no valid trigger. Skipping.");
                    }
                }
            }
        }
        Err(e) => error!("Failed to read tasks directory '{dir}': {e}"),
    }
    jobs
}

// Executes a command in a shell and logs its standard output and standard error.
fn execute_command(job: &Job) {
    let output = Command::new("sh").arg("-c").arg(&job.command).output();
    let description = &job.description;
    match output {
        Ok(out) => {
            info!("Command finished for: '{description}'");
            if !out.stdout.is_empty() {
                info!(
                    "--- stdout for {description} ---\n{}",
                    String::from_utf8_lossy(&out.stdout)
                );
            }
            if !out.stderr.is_empty() {
                error!(
                    "--- stderr for {description} ---\n{}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
        Err(e) => {
            error!("Failed to execute command for '{description}': {e}");
        }
    }
}

// A simple parser for duration strings like "1h30m10s".
fn parse_duration(s: &str) -> Option<ChronoDuration> {
    let mut total_seconds = 0i64;
    let mut current_number = String::new();

    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current_number.push(ch);
        } else if let Ok(num) = current_number.parse::<i64>() {
            match ch {
                'h' => total_seconds += num * 3600,
                'm' => total_seconds += num * 60,
                's' => total_seconds += num,
                _ => {} // Ignores other characters.
            }
            current_number.clear();
        }
    }

    if total_seconds > 0 {
        Some(ChronoDuration::seconds(total_seconds))
    } else {
        None
    }
}
