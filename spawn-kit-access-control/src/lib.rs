//! spawn-kit-access-control: Capability token verification.
//!
//! ## Params: `{"required_capability":[0,1,2,...],"presented_token":[0,1,2,...]}`

#![no_std]
#![no_main]

extern crate alloc;

use serde::Deserialize;
use spawn_kit_core::{read_request, write_response, CycleTracker, ErrorCode, Response, PROTOCOL_MAGIC};

const ERR_ACCESS_DENIED: u32 = 0x4001;

#[derive(Debug, Deserialize)]
struct AccessControlParams {
    required_capability: [u8; 32],
    presented_token: [u8; 32],
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
            let params: AccessControlParams = match serde_json_core::from_str::<AccessControlParams>(params_str) {
                Ok((p, _)) => p,
                Err(_) => {
                    write_response(&Response::error(ErrorCode::JsonParseError, b"bad params"));
                    ckb_std::syscalls::exit(-1);
                }
            };

            if params.required_capability == params.presented_token {
                write_response(&Response::ok(None, cycles));
            } else {
                write_response(&Response::error(ErrorCode::UnknownError, b"access denied"));
                ckb_std::syscalls::exit(-1);
            }
        }
        "metadata" => {
            let meta = b"{\"name\":\"spawn-kit-access-control\",\"version\":\"0.1.0\"}";
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
