#![no_main]
use libfuzzer_sys::fuzz_target;

// Imports the Task struct, which has `serde::Deserialize` derived.
use kronos::Task;

fuzz_target!(|data: &[u8]| {
    // Attempts to convert the random bytes into a valid UTF-8 string.
    if let Ok(s) = std::str::from_utf8(data) {
        // Feeds the random string directly to the TOML deserializer.
        // The goal is to find any edge case in toml or serde that causes a panic.
        let _ = toml::from_str::<Task>(s);
    }
});
