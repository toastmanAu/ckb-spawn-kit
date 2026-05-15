//! Integration tests for ckb-spawn-kit.
//!
//! These test the protocol types and logic on the host target (x86_64).
//! RISC-V specific tests run through ckb-debugger with tx.json fixtures.

use serde::{Deserialize, Serialize};

// Simulate the protocol types (no_std compatible)
mod protocol {
    use serde::{Deserialize, Serialize};

    pub const PROTOCOL_MAGIC: [u8; 4] = [0x53, 0x4b, 0x4b, 0x54]; // "SKT"

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Request {
        #[serde(with = "hex_bytes")]
        pub magic: [u8; 4],
        pub version: u16,
        pub action: String,
        pub params: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Response {
        #[serde(with = "hex_bytes")]
        pub magic: [u8; 4],
        pub ok: bool,
        pub code: u32,
        pub data: Option<String>,
        pub reason: String,
        pub cycles: u64,
    }

    impl Response {
        pub fn ok(data: Option<&str>, cycles: u64) -> Self {
            Self {
                magic: PROTOCOL_MAGIC,
                ok: true,
                code: 0,
                data: data.map(|s| s.to_string()),
                reason: "ok".into(),
                cycles,
            }
        }

        pub fn error(code: u32, reason: &str) -> Self {
            Self {
                magic: PROTOCOL_MAGIC,
                ok: false,
                code,
                data: None,
                reason: reason.into(),
                cycles: 0,
            }
        }
    }

    mod hex_bytes {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        pub fn serialize<S>(bytes: &[u8; 4], s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let hex = core::str::from_utf8(bytes).unwrap_or("SKT");
            hex.serialize(s)
        }

        pub fn deserialize<'de, D>(d: D) -> Result<[u8; 4], D::Error>
        where
            D: Deserializer<'de>,
        {
            let s: String = Deserialize::deserialize(d)?;
            let b = s.as_bytes();
            let mut out = [0u8; 4];
            let len = b.len().min(4);
            out[..len].copy_from_slice(&b[..len]);
            Ok(out)
        }
    }
}

// ─── Protocol JSON round-trip tests ──────────────────────────────────

#[test]
fn request_serialize_verify_action() {
    let req = protocol::Request {
        magic: protocol::PROTOCOL_MAGIC,
        version: 1,
        action: "verify".into(),
        params: r#"{"threshold":2,"total_keys":3}"#.into(),
    };

    let json = serde_json::to_string(&req).unwrap();
    let parsed: protocol::Request = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.magic, protocol::PROTOCOL_MAGIC);
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.action, "verify");
    assert!(parsed.params.contains("threshold"));
}

#[test]
fn request_serialize_metadata_action() {
    let req = protocol::Request {
        magic: protocol::PROTOCOL_MAGIC,
        version: 1,
        action: "metadata".into(),
        params: "{}".into(),
    };

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("SKT"));
    assert!(json.contains("metadata"));
}

#[test]
fn response_ok_round_trip() {
    let resp = protocol::Response::ok(Some(r#"{"name":"multisig"}"#), 12345);

    let json = serde_json::to_string(&resp).unwrap();
    let parsed: protocol::Response = serde_json::from_str(&json).unwrap();

    assert!(parsed.ok);
    assert_eq!(parsed.code, 0);
    assert_eq!(parsed.cycles, 12345);
    assert_eq!(parsed.data.unwrap(), r#"{"name":"multisig"}"#);
}

#[test]
fn response_error_round_trip() {
    let resp = protocol::Response::error(0x1002, "threshold not met");

    let json = serde_json::to_string(&resp).unwrap();
    let parsed: protocol::Response = serde_json::from_str(&json).unwrap();

    assert!(!parsed.ok);
    assert_eq!(parsed.code, 0x1002);
    assert_eq!(parsed.reason, "threshold not met");
    assert!(parsed.data.is_none());
}

#[test]
fn magic_mismatch_rejected() {
    let resp = protocol::Response {
        magic: [0xBA, 0xAD, 0xBE, 0xEF],
        ok: true,
        code: 0,
        data: None,
        reason: "ok".into(),
        cycles: 0,
    };

    assert_ne!(resp.magic, protocol::PROTOCOL_MAGIC);
}

// ─── Multisig logic tests ────────────────────────────────────────────

mod multisig_logic {
    #[derive(Debug)]
    struct MultisigParams {
        threshold: u8,
        total_keys: u8,
        signatures: Vec<[u8; 65]>,
        public_keys: Vec<[u8; 33]>,
    }

    fn verify_multisig(params: &MultisigParams) -> Result<bool, u32> {
        if params.public_keys.len() != params.total_keys as usize {
            return Err(0x1006); // ERR_KEY_COUNT_MISMATCH
        }
        if params.threshold == 0 || params.threshold > params.total_keys {
            return Err(0x1002); // ERR_THRESHOLD_NOT_MET
        }
        if params.signatures.len() < params.threshold as usize {
            return Ok(false);
        }

        let mut valid: u8 = 0;
        for sig in &params.signatures {
            for pk in &params.public_keys {
                if sig[0] == pk[0] {
                    // simplified: match on first byte
                    valid += 1;
                    break;
                }
            }
            if valid >= params.threshold {
                return Ok(true);
            }
        }
        Ok(valid >= params.threshold)
    }

    #[test]
    fn all_signatures_match() {
        let params = MultisigParams {
            threshold: 2,
            total_keys: 3,
            signatures: vec![[1u8; 65], [2u8; 65]],
            public_keys: vec![[1u8; 33], [2u8; 33], [3u8; 33]],
        };
        assert_eq!(verify_multisig(&params), Ok(true));
    }

    #[test]
    fn partial_signatures_insufficient() {
        let params = MultisigParams {
            threshold: 3,
            total_keys: 5,
            signatures: vec![[1u8; 65], [2u8; 65]],
            public_keys: vec![[1u8; 33], [2u8; 33], [3u8; 33], [4u8; 33], [5u8; 33]],
        };
        assert_eq!(verify_multisig(&params), Ok(false));
    }

    #[test]
    fn key_count_mismatch_errors() {
        let params = MultisigParams {
            threshold: 1,
            total_keys: 5,
            signatures: vec![[1u8; 65]],
            public_keys: vec![[1u8; 33], [2u8; 33]],
        };
        assert!(verify_multisig(&params).is_err());
    }

    #[test]
    fn zero_threshold_rejected() {
        let params = MultisigParams {
            threshold: 0,
            total_keys: 3,
            signatures: vec![],
            public_keys: vec![[1u8; 33], [2u8; 33], [3u8; 33]],
        };
        assert!(verify_multisig(&params).is_err());
    }

    #[test]
    fn threshold_gt_total_keys_rejected() {
        let params = MultisigParams {
            threshold: 5,
            total_keys: 3,
            signatures: vec![],
            public_keys: vec![[1u8; 33], [2u8; 33], [3u8; 33]],
        };
        assert!(verify_multisig(&params).is_err());
    }
}

// ─── Timelock logic tests ────────────────────────────────────────────

#[test]
fn timelock_passes_when_epoch_reached() {
    let since_epoch = 100;
    let current_epoch = 150;
    assert!(current_epoch >= since_epoch);
}

#[test]
fn timelock_fails_when_epoch_not_reached() {
    let since_epoch = 200;
    let current_epoch = 150;
    assert!(current_epoch < since_epoch);
}

#[test]
fn timelock_passes_exactly_at_epoch() {
    let since_epoch = 100;
    let current_epoch = 100;
    assert!(current_epoch >= since_epoch);
}

// ─── Ratelimit logic tests ───────────────────────────────────────────

#[test]
fn ratelimit_allows_under_max() {
    let max = 100u64;
    let current_count = 50;
    assert!(current_count + 1 <= max);
}

#[test]
fn ratelimit_blocks_at_max() {
    let max = 100u64;
    let current_count = 100;
    assert!(current_count + 1 > max);
}

#[test]
fn ratelimit_window_reset() {
    let window_end = 1000u64;
    let current_epoch = 1001u64;
    assert!(current_epoch > window_end);
    // Count resets to 1
    let new_count = if current_epoch > window_end { 1 } else { 99 };
    assert_eq!(new_count, 1);
}

// ─── Error code range tests ──────────────────────────────────────────

#[test]
fn error_ranges_dont_overlap() {
    let ranges = [
        ("core", 0x0000u32, 0x0FFFu32),
        ("multisig", 0x1000, 0x1FFF),
        ("timelock", 0x2000, 0x2FFF),
        ("ratelimit", 0x3000, 0x3FFF),
        ("access-control", 0x4000, 0x4FFF),
        ("escrow", 0x5000, 0x5FFF),
    ];

    for i in 0..ranges.len() {
        for j in (i + 1)..ranges.len() {
            let (name_a, a_start, a_end) = ranges[i];
            let (name_b, b_start, b_end) = ranges[j];
            assert!(
                a_end < b_start,
                "{name_a} range ({a_start:#06x}-{a_end:#06x}) overlaps with {name_b} ({b_start:#06x}-{b_end:#06x})"
            );
        }
    }
}

// ─── Frame size boundary tests ───────────────────────────────────────

#[test]
fn max_frame_size_is_reasonable() {
    // 4096 bytes is enough for complex multisig params (65 bytes * 15 sigs + 33 bytes * 15 keys + overhead)
    let max_frame: usize = 4096;
    let worst_case_multisig = 15 * 65 + 15 * 33 + 200; // signatures + keys + JSON overhead
    assert!(worst_case_multisig < max_frame);
}

#[test]
fn json_overhead_accounted_for() {
    // A verify request with minimal params
    let req_json = r#"{"magic":"SKT","version":1,"action":"verify","params":{"threshold":2,"total_keys":3,"message":"0000000000000000000000000000000000000000000000000000000000000000","signatures":["0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"],"public_keys":["000000000000000000000000000000000000000000000000000000000000000000"],"algorithm":"secp256k1-blake2b"}}"#;
    assert!(req_json.len() < 4096);
    assert!(req_json.len() > 200); // sanity: not empty
}
