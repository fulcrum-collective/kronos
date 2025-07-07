# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

---

## [0.2.3] - 2025-07-07

This release focuses on establishing an automated quality assurance pipeline and improving general code hygiene.

### Added
- **Continuous Integration**: Introduced a CI workflow using GitHub Actions. It automatically checks code formatting (`fmt`), static analysis lints (`clippy`), and runs tests on every pull request to the `dev` branch.

### Removed
- **Unused Files**: Removed unused source files from the codebase to improve clarity and maintainability.

## [0.2.2] - 2025-07-06

This release introduces proper timezone handling to ensure scheduling is accurate and predictable worldwide.

### Changed
- **Timezone Handling**: All internal time processing now converts local time from configuration files into UTC. This ensures scheduling is consistent and correct regardless of the server's geographical location.

## [0.2.1] - 2025-07-06

This release focuses on fixing a critical scheduling logic bug and improving overall code quality.

### Fixed
- **Scheduler Delay Bug**: Fixed a logic flaw in the scheduling loop that could cause due tasks to be executed with a significant delay (often up to a minute) after their scheduled time.

## [0.2.0] - 2025-07-06

This release marks the evolution of Kronos from a simple prototype into a functional, multi-tasking scheduler daemon. The focus was on architectural improvements, core feature implementation, and enhancing robustness.

### Added
- **Multi-Task Loading**: The scheduler now loads all `.toml` task files from a dedicated directory (`tasks.d/`), enabling it to manage multiple jobs simultaneously.
- **Support for Recurring Tasks**: In addition to one-time tasks, a new `every` field is supported in the trigger configuration for simple, interval-based recurring tasks (e.g., `every = "1h30m"`).
- **Basic Logging System**: Integrated the `log` and `env_logger` crates to provide structured logging for daemon status, task loading, and execution results.

### Changed
- **Scheduler Logic Overhaul**: The main loop was re-architected to handle all due tasks in a single cycle, preventing execution delays when multiple jobs have similar trigger times.
- **Unified Timestamp Precision**: All internal time calculations are now consistently truncated to second-level precision, leading to cleaner logs and more predictable behavior.

### Fixed
- **Time Drift in Recurring Tasks**: Rescheduling logic for recurring tasks is now based on their previous *scheduled* run time, not the *actual* run time, which completely eliminates cumulative time drift.
- **Error Handling**: Replaced a `.expect()` call during directory creation with graceful error logging and a non-zero process exit code.
- **Code Quality and Lints**: Addressed all warnings from a strict `clippy -- -D warnings` run, adopting modern Rust idioms like captured identifiers in format strings.

## [0.1.0] - 2025-07-05

This was the initial proof-of-concept release, establishing the basic functionality of the Kronos timer.

### Added
- **Initial Prototype**: First runnable version of the `kronos` binary.
- **Single Task Parsing**: Ability to parse a single, hardcoded `task.toml` file.
- **One-Time Task Trigger**: Support for a single, one-time execution based on the `on_calendar` trigger.
- **Command Execution**: Basic command execution using `std::process::Command`.
