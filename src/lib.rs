// src/lib.rs

// Declares the `task` module.
pub mod task;

// Re-exports primary data structures for library consumers.
pub use task::{Job, Task, Trigger};

// Imports items from modules and external crates.
use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveDateTime, TimeZone, Timelike, Utc};
use log::{error, info, warn};
use std::{fs, process::Command};

// A data structure holding job details and its next execution time.
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub job: Job,
    pub trigger: Trigger,
    pub next_run_at: DateTime<Utc>,
}

// Loads and parses all .toml files from a directory into a vector of ScheduledJob.
pub fn load_and_schedule_tasks(dir: &str) -> Vec<ScheduledJob> {
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

        // Deserializes the TOML content into a Task struct.
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
pub fn execute_command(job: &Job) {
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
pub fn parse_duration(s: &str) -> Option<ChronoDuration> {
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
