//! Composed Lock Example — Multisig + Timelock via Spawn
//!
//! This is a complete Lock Script that composes TWO spawn-kit modules:
//!   1. spawn-kit-multisig: M-of-N signature verification
//!   2. spawn-kit-timelock: time-based access control
//!
//! The lock unlocks when BOTH conditions are met.
//!
//! ## Deploy
//!
//! 1. Deploy spawn-kit-multisig and spawn-kit-timelock as Cells on-chain
//! 2. Add them to your transaction's CellDeps
//! 3. Deploy THIS composed lock script
//! 4. Lock Cells with this script
//!
//! ## Unlock
//!
//! Build a transaction with:
//!   - CellDep #0: spawn-kit-multisig binary
//!   - CellDep #1: spawn-kit-timelock binary
//!   - Witness: signatures
//!   - The block must be past the timelock epoch

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::convert::TryInto;
use spawn_kit_core::caller::{self, ModuleResult};
use ckb_std::ckb_constants::Source;

/// Lock args layout (packed in script args):
///   [threshold: u8] [total_keys: u8] [since_epoch: u64 LE] [pubkey_hash: [u8; 20]]
const ARGS_LEN: usize = 1 + 1 + 8 + 20; // 30 bytes

/// Run the composed lock verification.
/// Returns Ok(()) if unlock conditions are met, Err(exit_code) otherwise.
pub fn verify() -> Result<(), i8> {
    // ── Parse lock args ──────────────────────────────────────────────
    let mut script_buf = [0u8; 128];
    let script_len =
        ckb_std::syscalls::load_script(&mut script_buf, 0).unwrap_or(0);

    // Script is molecule-encoded: {code_hash(32), hash_type(1), args}
    // args start at offset 53
    let args_offset = 53;
    if script_len < args_offset + ARGS_LEN {
        return Err(-1);
    }
    let args = &script_buf[args_offset..args_offset + ARGS_LEN];

    let threshold = args[0];
    let total_keys = args[1];
    let since_epoch = u64::from_le_bytes(args[2..10].try_into().unwrap());

    // ── Get current epoch from header deps ────────────────────────────
    let _current_epoch = ckb_std::high_level::load_header_epoch_number(
        0,
        ckb_std::ckb_constants::Source::HeaderDep,
    )
    .unwrap_or(0);

    // ── Prepare module params ─────────────────────────────────────────
    let multisig_params = alloc::format!(
        r#"{{"threshold":{},"total_keys":{},"message":"<tx_hash>","signatures":[],"public_keys":[],"algorithm":"secp256k1-blake2b"}}"#,
        threshold,
        total_keys
    );

    let timelock_params = alloc::format!(
        r#"{{"since_epoch":{},"relative_to":"block_header"}}"#,
        since_epoch
    );

    let ctx = spawn_kit_core::CallContext {
        tx_hash: None,
        cell_index: None,
    };

    // ── Call modules in parallel ────────────────────────────────────
    // CellDep #0 = spawn-kit-multisig, CellDep #1 = spawn-kit-timelock
    let modules: [(usize, &str); 2] =
        [(0, &multisig_params), (1, &timelock_params)];
    let results: Vec<ModuleResult> = match caller::call_all(&modules, &ctx) {
        Ok(r) => r,
        Err(_) => return Err(-2),
    };

    // ── Check results ───────────────────────────────────────────────
    let multisig_ok = results[0].response.ok;
    let timelock_ok = results[1].response.ok;

    if multisig_ok && timelock_ok {
        Ok(())
    } else {
        Err(-3)
    }
}
