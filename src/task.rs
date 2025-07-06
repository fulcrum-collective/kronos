// src/task.rs
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Job {
    pub description: String,
    pub command: String,
}

#[derive(Deserialize, Debug)]
pub struct Trigger {
    pub on_calendar: String,
}

#[derive(Deserialize, Debug)]
pub struct Task {
    pub job: Job,
    pub trigger: Trigger,
}
