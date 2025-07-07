// src/main.rs

// Declares the `task` module.
mod task;

// Imports items from modules and external crates.
use crate::task::{Job, Task};
use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveDateTime, TimeZone, Timelike, Utc};
use log::{error, info, warn};
use std::{fs, path::Path, process::Command, thread, time::Duration as StdDuration};

// A data structure holding job details and its next execution time.
#[derive(Debug, Clone)]
struct ScheduledJob {
    job: Job,
    trigger: task::Trigger,
    next_run_at: DateTime<Utc>,
}

fn main() {
    // Initializes the global logger.
    env_logger::init();

    info!("Kronos starting up...");

    // Defines the directory path for task configuration files.
    let tasks_dir = "/etc/kronos/tasks.d";
    if !Path::new(tasks_dir).exists() {
        info!("'{tasks_dir}' directory not found. Creating it.");
        if let Err(e) = fs::create_dir_all(tasks_dir) {
            error!("Failed to create tasks directory '{tasks_dir}': {e}");
            std::process::exit(1);
        }
    }

    // Loads and schedules tasks from the configuration directory.
    let mut scheduled_jobs = load_and_schedule_tasks(tasks_dir);

    if scheduled_jobs.is_empty() {
        warn!("No tasks found. Kronos will idle.");
    }

    // The main daemon loop for task scheduling and execution.
    loop {
        // Sorts the job list by the next execution time.
        scheduled_jobs.sort_by_key(|j| j.next_run_at);

        // Gets the current time in UTC, truncated to second precision.
        let now_utc = Utc::now().with_nanosecond(0).unwrap();

        // Iterates through and processes all jobs that are due.
        while let Some(next_job) = scheduled_jobs.first() {
            if now_utc >= next_job.next_run_at {
                let mut job_to_run = scheduled_jobs.remove(0);
                let description = &job_to_run.job.description;
                let next_run_at = &job_to_run.next_run_at;
                info!("Trigger time reached! Executing: '{description}' (Scheduled for: {next_run_at})");

                // Spawns a new thread to execute the command non-blockingly.
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

        // Determines the sleep duration until the next scheduled job.
        let sleep_duration = if let Some(next_job) = scheduled_jobs.first() {
            let duration_to_next = next_job.next_run_at.signed_duration_since(now_utc);
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

        // Pauses the loop, with a maximum cap to ensure responsiveness.
        let final_sleep = std::cmp::min(sleep_duration, StdDuration::from_secs(60));
        thread::sleep(final_sleep);
    }
}

// Loads and parses all .toml files from a directory into a vector of ScheduledJob.
fn load_and_schedule_tasks(dir: &str) -> Vec<ScheduledJob> {
    let mut jobs = Vec::new();
    let now_utc = Utc::now().with_nanosecond(0).unwrap();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read tasks directory '{dir}': {e}");
            return jobs;
        }
    };

    for entry in entries {
        let path = match entry {
            Ok(e) => e.path(),
            Err(_) => continue,
        };
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

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

        // Determines the next run time and converts it to a UTC timestamp.
        let next_run_at_utc: Option<DateTime<Utc>> =
            if let Some(cal_str) = &task.trigger.on_calendar {
                match NaiveDateTime::parse_from_str(cal_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_time) => Local
                        .from_local_datetime(&naive_time)
                        .single()
                        .map(|local_time| local_time.with_timezone(&Utc)),
                    Err(_) => {
                        warn!("Invalid on_calendar format in {path:?}: \"{cal_str}\"");
                        None
                    }
                }
            } else if let Some(every_str) = &task.trigger.every {
                parse_duration(every_str).map(|d| now_utc + d)
            } else {
                None
            };

        if let Some(run_time_utc) = next_run_at_utc {
            // Skips one-time tasks scheduled in the past.
            if task.trigger.on_calendar.is_some() && run_time_utc < now_utc {
                warn!("Skipping past one-time task in {path:?}: scheduled for {run_time_utc}");
                continue;
            }

            jobs.push(ScheduledJob {
                job: task.job.clone(),
                trigger: task.trigger.clone(),
                next_run_at: run_time_utc,
            });
        } else {
            warn!("Task in {path:?} has no valid trigger. Skipping.");
        }
    }

    jobs
}

// Executes a command in a shell and logs its output.
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

// Parses a duration string (e.g., "1h30m10s") into a `ChronoDuration`.
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
