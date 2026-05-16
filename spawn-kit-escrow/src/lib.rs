#![no_std]

extern crate alloc;

use serde::Deserialize;
use spawn_kit_core::{read_request, write_response, CycleTracker, ErrorCode, Response, PROTOCOL_MAGIC};

const ERR_NOT_YET_RELEASABLE: u32 = 0x5002;
const ERR_MISSING_ATTESTATION: u32 = 0x5003;

#[derive(Debug, Deserialize)]
struct EscrowParams {
    release_epoch: u64,
    #[serde(default)]
    reviewer_attestation: Option<bool>,
}

pub fn verify(params_json: &[u8], current_epoch: u64) -> Result<bool, ErrorCode> {
    let params_str = core::str::from_utf8(params_json).map_err(|_| ErrorCode::JsonParseError)?;
    let params: EscrowParams = serde_json_core::from_str::<EscrowParams>(params_str)
        .map(|(p, _)| p)
        .map_err(|_| ErrorCode::JsonParseError)?;
    if current_epoch < params.release_epoch {
        return Ok(false);
    }
    if let Some(false) = params.reviewer_attestation {
        return Ok(false);
    }
    Ok(true)
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
            let current_epoch = ckb_std::high_level::load_header_epoch_number(0, ckb_std::ckb_constants::Source::HeaderDep).unwrap_or(0);
            match verify(&request.params, current_epoch) {
                Ok(true) => write_response(&Response::ok(None, cycles)),
                Ok(false) => {
                    write_response(&Response::error(ErrorCode::UnknownError, b"conditions not met"));
                    ckb_std::syscalls::exit(-1);
                }
                Err(e) => { write_response(&Response::error(e, b"bad params")); ckb_std::syscalls::exit(-1); }
            }
        }
        "metadata" => {
            let meta = b"{\"name\":\"spawn-kit-escrow\",\"version\":\"0.1.0\"}";
            write_response(&Response::ok(Some(meta.to_vec()), cycles));
        }
        _ => { write_response(&Response::error(ErrorCode::InvalidAction, b"unknown action")); ckb_std::syscalls::exit(-1); }
    }
    ckb_std::syscalls::exit(0);
}
