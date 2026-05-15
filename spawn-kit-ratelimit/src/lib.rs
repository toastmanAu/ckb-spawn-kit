#![no_std]

extern crate alloc;

use serde::Deserialize;
use spawn_kit_core::{read_request, write_response, CycleTracker, ErrorCode, Response, PROTOCOL_MAGIC};

#[derive(Debug, Deserialize)]
struct RatelimitParams {
    max_operations: u64,
    window_epochs: u64,
    current_count: u64,
    first_epoch_in_window: u64,
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
        "verify" => {
            let params_str = core::str::from_utf8(&request.params).unwrap_or("{}");
            let params: RatelimitParams = match serde_json_core::from_str::<RatelimitParams>(params_str) {
                Ok((p, _)) => p,
                Err(_) => { write_response(&Response::error(ErrorCode::JsonParseError, b"bad params")); ckb_std::syscalls::exit(-1); }
            };
            let current_epoch = ckb_std::high_level::load_header_epoch_number(0, ckb_std::ckb_constants::Source::HeaderDep).unwrap_or(0);
            let window_end = params.first_epoch_in_window.saturating_add(params.window_epochs);
            let count = if current_epoch > window_end { 1 } else { params.current_count + 1 };
            if count <= params.max_operations {
                write_response(&Response::ok(None, cycles));
            } else {
                write_response(&Response::error(ErrorCode::UnknownError, b"rate limit exceeded"));
                ckb_std::syscalls::exit(-1);
            }
        }
        "metadata" => {
            let meta = b"{\"name\":\"spawn-kit-ratelimit\",\"version\":\"0.1.0\"}";
            write_response(&Response::ok(Some(meta.to_vec()), cycles));
        }
        _ => { write_response(&Response::error(ErrorCode::InvalidAction, b"unknown action")); ckb_std::syscalls::exit(-1); }
    }
    ckb_std::syscalls::exit(0);
}
