//! spawn-kit-core: Standard ABI and protocol types for spawn-based script composition.
//!
//! Every module communicates over inherited file descriptors:
//!   fd 0 = stdin  (read JSON Request)
//!   fd 1 = stdout (write JSON Response)
//!
//! ## Protocol
//!
//! Request (stdin):  `{"magic":"SKT","version":1,"action":"verify","params":...}`
//! Response (stdout): `{"magic":"SKT","ok":true,"code":0,"data":...,"reason":"ok","cycles":123}`

#![no_std]

extern crate alloc;

pub mod caller;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

// ─── Protocol Constants ───────────────────────────────────────────────

pub const PROTOCOL_MAGIC: [u8; 4] = [0x53, 0x4b, 0x4b, 0x54]; // "SKT"

pub const PROTOCOL_VERSION: u16 = 1;

pub const MAX_FRAME_SIZE: usize = 4096;

// ─── Error Codes ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode {
    Success = 0,
    UnknownError = 0x0001,
    InvalidMagic = 0x0002,
    InvalidVersion = 0x0003,
    FrameTooLarge = 0x0004,
    JsonParseError = 0x0005,
    InvalidAction = 0x0006,
    MissingRequiredParam = 0x0007,
    ChildProcessError = 0x0008,
    PipeReadError = 0x0009,
    PipeWriteError = 0x000A,
}

impl ErrorCode {
    pub fn reason(self) -> &'static str {
        match self {
            Self::Success => "ok",
            Self::UnknownError => "unknown error",
            Self::InvalidMagic => "invalid protocol magic",
            Self::InvalidVersion => "unsupported protocol version",
            Self::FrameTooLarge => "frame too large",
            Self::JsonParseError => "failed to parse JSON",
            Self::InvalidAction => "unknown action",
            Self::MissingRequiredParam => "missing required parameter",
            Self::ChildProcessError => "child process error",
            Self::PipeReadError => "pipe read error",
            Self::PipeWriteError => "pipe write error",
        }
    }
}

// ─── Request / Response ───────────────────────────────────────────────

/// Context passed to every module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallContext {
    pub tx_hash: Option<[u8; 32]>,
    pub cell_index: Option<u32>,
}

/// Request that a caller sends to a module via stdin.
/// Owns all data so it can outlive the I/O buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub magic: [u8; 4],
    pub version: u16,
    pub action: Vec<u8>,
    pub params: Vec<u8>,
}

/// Response a module writes to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub magic: [u8; 4],
    pub ok: bool,
    pub code: u32,
    pub data: Option<Vec<u8>>,
    pub reason: Vec<u8>,
    pub cycles: u64,
}

// ─── Convenience constructors ─────────────────────────────────────────

impl Response {
    pub fn ok(data: Option<Vec<u8>>, cycles: u64) -> Self {
        Self {
            magic: PROTOCOL_MAGIC,
            ok: true,
            code: 0,
            data,
            reason: b"ok".to_vec(),
            cycles,
        }
    }

    pub fn error(code: ErrorCode, reason: &[u8]) -> Self {
        Self {
            magic: PROTOCOL_MAGIC,
            ok: false,
            code: code as u32,
            data: None,
            reason: reason.to_vec(),
            cycles: 0,
        }
    }
}

// ─── Cycle tracking ───────────────────────────────────────────────────

pub struct CycleTracker {
    start: u64,
}

impl CycleTracker {
    pub fn start() -> Self {
        Self {
            start: ckb_std::syscalls::current_cycles(),
        }
    }

    pub fn elapsed(&self) -> u64 {
        ckb_std::syscalls::current_cycles().saturating_sub(self.start)
    }
}

// ─── Module I/O helpers ───────────────────────────────────────────────

/// Read and parse a Request from stdin (fd 0).
pub fn read_request() -> Result<Request, ErrorCode> {
    let mut buf = [0u8; MAX_FRAME_SIZE];
    let len = ckb_std::syscalls::read(0, &mut buf).map_err(|_| ErrorCode::PipeReadError)?;
    let (req, _used) = serde_json_core::from_slice::<Request>(&buf[..len]).map_err(|_| ErrorCode::JsonParseError)?;
    Ok(req)
}

/// Write a Response to stdout (fd 1).
pub fn write_response(resp: &Response) {
    if let Ok(data) = serde_json_core::to_vec::<_, MAX_FRAME_SIZE>(resp) {
        ckb_std::syscalls::write(1, &data).ok();
    }
}

// Note: Each cdylib module provides its own global_allocator and panic_handler.
// Core is a library — do NOT define them here to avoid duplicate lang items.
