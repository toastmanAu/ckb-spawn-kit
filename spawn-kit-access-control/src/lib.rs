#![no_std]

extern crate alloc;

use serde::Deserialize;
use spawn_kit_core::{read_request, write_response, CycleTracker, ErrorCode, Response, PROTOCOL_MAGIC};

#[derive(Debug, Deserialize)]
struct AccessControlParams {
    required_capability: [u8; 32],
    presented_token: [u8; 32],
}

pub fn verify(params_json: &[u8]) -> Result<bool, ErrorCode> {
    let params_str = core::str::from_utf8(params_json).map_err(|_| ErrorCode::JsonParseError)?;
    let params: AccessControlParams = serde_json_core::from_str::<AccessControlParams>(params_str)
        .map(|(p, _)| p)
        .map_err(|_| ErrorCode::JsonParseError)?;
    let mut ok = true;
    for i in 0..32 {
        if params.required_capability[i] != params.presented_token[i] {
            ok = false;
            break;
        }
    }
    Ok(ok)
}

pub fn entry() -> ! {
    let tracker = CycleTracker::start();
    let request = match read_request() {
        Ok(r) => r,
        Err(e) => { write_response(&Response::error(e, b"bad request")); ckb_std::syscalls::exit(-1); }
    };
    if request.magic != PROTOCOL_MAGIC {
        write_response(&Response::error(ErrorCode::InvalidMagic, b"protocol mismatch"));
        ckb_std::syscalls::exit(-1);
    }
    let cycles = tracker.elapsed();
    match core::str::from_utf8(&request.action).unwrap_or("") {
        "verify" => match verify(&request.params) {
            Ok(true) => write_response(&Response::ok(None, cycles)),
            Ok(false) => {
                write_response(&Response::error(ErrorCode::UnknownError, b"access denied"));
                ckb_std::syscalls::exit(-1);
            }
            Err(e) => { write_response(&Response::error(e, b"bad params")); ckb_std::syscalls::exit(-1); }
        },
        "metadata" => {
            let meta = b"{\"name\":\"spawn-kit-access-control\",\"version\":\"0.1.0\"}";
            write_response(&Response::ok(Some(meta.to_vec()), cycles));
        }
        _ => { write_response(&Response::error(ErrorCode::InvalidAction, b"unknown action")); ckb_std::syscalls::exit(-1); }
    }
    ckb_std::syscalls::exit(0);
}
