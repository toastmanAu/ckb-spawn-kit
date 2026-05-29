//! Timelock-Only Composed Lock — delegates to spawn-kit-timelock via spawn()
//!
//! This is the simplest possible spawn composition test:
//!   1. Parse `since_epoch` from lock args
//!   2. Spawn spawn-kit-timelock (CellDep #0)
//!   3. Module checks current_epoch >= since_epoch
//!   4. Forward the module's result
//!
//! Lock args: [since_epoch: u64 LE]

#![no_std]

extern crate alloc;

use core::convert::TryInto;
use spawn_kit_core::caller::{self, ModuleResult};

/// Run the composed lock verification.
pub fn verify() -> Result<(), i8> {
    // ── Parse lock args (since_epoch as u64 LE) ───────────────────────
    let mut script_buf = [0u8; 256];
    let script_len =
        ckb_std::syscalls::load_script(&mut script_buf, 0).unwrap_or(0);

    // Script is molecule-encoded: {code_hash(32), hash_type(1), args}
    // args start at offset 53
    let args_offset = 53;
    let args_len = 8; // just since_epoch (u64)
    if script_len < args_offset + args_len {
        return Err(-1);
    }
    let args = &script_buf[args_offset..args_offset + args_len];
    let since_epoch = u64::from_le_bytes(args.try_into().unwrap());

    // ── Prepare timelock params ───────────────────────────────────────
    let timelock_params = alloc::format!(
        r#"{{"since_epoch":{}}}"#,
        since_epoch
    );

    let ctx = spawn_kit_core::CallContext {
        tx_hash: None,
        cell_index: None,
    };

    // ── Spawn timelock module (CellDep #0) ───────────────────────────
    let result: ModuleResult = match caller::call_module(0, &timelock_params, &ctx) {
        Ok(r) => r,
        Err(_) => return Err(-2),
    };

    // ── Forward the module's result ──────────────────────────────────
    if result.response.ok {
        Ok(())
    } else {
        Err(-3)
    }
}
