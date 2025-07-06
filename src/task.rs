// src/task.rs
// Defines the core data structures for a Kronos task.

use serde::Deserialize;

// Represents the job to be executed.
#[derive(Deserialize, Debug, Clone)]
pub struct Job {
    pub description: String,
    pub command: String,
}

// Represents the trigger condition for the job.
// Both fields are optional to allow for different trigger types.
#[derive(Deserialize, Debug, Clone)]
pub struct Trigger {
    // A specific datetime for a one-time task. Format: "YYYY-MM-DD HH:MM:SS"
    pub on_calendar: Option<String>,
    // A duration string for a recurring task. Format: "1h30m10s"
    pub every: Option<String>,
}

// The top-level struct representing a single .toml task file.
#[derive(Deserialize, Debug, Clone)]
pub struct Task {
    pub job: Job,
    pub trigger: Trigger,
}
