#![no_main]

use libfuzzer_sys::fuzz_target;
use spawn_kit_core::Request;

fuzz_target!(|data: &[u8]| {
    // Fuzz Request deserialization — should never panic
    let _ = serde_json_core::from_slice::<Request>(data);

    // Fuzz Response deserialization
    let _ = serde_json_core::from_slice::<spawn_kit_core::Response>(data);

    // Fuzz JSON string parsing
    if let Ok(s) = core::str::from_utf8(data) {
        let _ = serde_json_core::from_str::<Request>(s);
        let _ = serde_json_core::from_str::<spawn_kit_core::Response>(s);
    }
});
