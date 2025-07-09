// src/main.rs

// Imports all public items from the `kronos` library crate.
use kronos::*;

use chrono::{Duration as ChronoDuration, Timelike, Utc};
use log::{error, info, warn};
use std::{path::Path, thread, time::Duration as StdDuration};

fn main() {
    // Initializes the global logger.
    env_logger::init();

    info!("Kronos starting up...");

    // Defines the directory path for task configuration files.
    let tasks_dir = "/etc/kronos/tasks.d";
    if !Path::new(tasks_dir).exists() {
        info!("'{tasks_dir}' directory not found. Creating it.");
        if let Err(e) = std::fs::create_dir_all(tasks_dir) {
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
