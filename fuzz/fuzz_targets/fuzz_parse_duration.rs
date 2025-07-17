#![no_main]
use libfuzzer_sys::fuzz_target;

// Imports the function to be fuzzed from the kronos library crate.
use kronos::parse_duration;

fuzz_target!(|data: &[u8]| {
    // Fuzzer generates random bytes.
    // Attempts to convert the bytes into a valid UTF-8 string.
    if let Ok(s) = std::str::from_utf8(data) {
        // If successful, pass the random string to the function under test.
        // The fuzzer will report a crash if this function call panics.
        let _ = parse_duration(s);
    }
});
