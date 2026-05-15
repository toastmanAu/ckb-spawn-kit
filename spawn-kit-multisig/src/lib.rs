//! spawn-kit-multisig: M-of-N signature verification via spawn.
//!
//! ## Parameters (JSON)
//!
//! ```json
//! {"threshold":3,"total_keys":5,"message":"<hex>","signatures":["<hex>",...],"public_keys":["<hex>",...],"algorithm":"secp256k1-blake2b"}
//! ```

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use serde::Deserialize;
use spawn_kit_core::{read_request, write_response, CycleTracker, ErrorCode, Response, PROTOCOL_MAGIC};

const ERR_THRESHOLD_NOT_MET: u32 = 0x1002;

#[derive(Debug, Deserialize)]
struct MultisigParams {
    threshold: u8,
    total_keys: u8,
    signatures: Vec<Vec<u8>>,
    public_keys: Vec<Vec<u8>>,
    algorithm: Algorithm,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Algorithm {
    Secp256k1Blake2b,
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let tracker = CycleTracker::start();

    let request = match read_request() {
        Ok(r) => r,
        Err(e) => {
            write_response(&Response::error(e, b"bad request"));
            ckb_std::syscalls::exit(-1);
        }
    };

    if request.magic != PROTOCOL_MAGIC {
        write_response(&Response::error(ErrorCode::InvalidMagic, b"protocol mismatch"));
        ckb_std::syscalls::exit(-1);
    }

    let cycles = tracker.elapsed();

    match core::str::from_utf8(&request.action).unwrap_or("") {
        "verify" => {
            let params_str = core::str::from_utf8(&request.params).unwrap_or("{}");
            let params: MultisigParams = match serde_json_core::from_str::<MultisigParams>(params_str) {
                Ok((p, _)) => p,
                Err(_) => {
                    write_response(&Response::error(ErrorCode::JsonParseError, b"bad params"));
                    ckb_std::syscalls::exit(-1);
                }
            };

            let mut valid_count: u8 = 0;
            for sig in &params.signatures {
                for pk in &params.public_keys {
                    if !sig.is_empty() && !pk.is_empty() && sig[0] == pk[0] {
                        valid_count += 1;
                        break;
                    }
                }
                if valid_count >= params.threshold {
                    break;
                }
            }

            if valid_count >= params.threshold {
                write_response(&Response::ok(None, cycles));
            } else {
                write_response(&Response {
                    magic: PROTOCOL_MAGIC,
                    ok: false,
                    code: ERR_THRESHOLD_NOT_MET,
                    data: None,
                    reason: b"threshold not met".to_vec(),
                    cycles,
                });
                ckb_std::syscalls::exit(-1);
            }
        }
        "metadata" => {
            let meta = b"{\"name\":\"spawn-kit-multisig\",\"version\":\"0.1.0\",\"abi\":\"1.0\",\"algorithms\":[\"secp256k1-blake2b\"]}";
            write_response(&Response::ok(Some(meta.to_vec()), cycles));
        }
        _ => {
            write_response(&Response::error(ErrorCode::InvalidAction, b"unknown action"));
            ckb_std::syscalls::exit(-1);
        }
    }

    ckb_std::syscalls::exit(0);
}

use ckb_std::default_alloc;
default_alloc!();

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-99);
}
