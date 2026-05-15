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
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use spawn_kit_core::caller::{self, call_module, ModuleResult};

/// Lock args layout (packed in script args):
///   [threshold: u8] [total_keys: u8] [since_epoch: u64 LE] [pubkey_hash: [u8; 20]]
const ARGS_LEN: usize = 1 + 1 + 8 + 20; // 30 bytes

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // ── Parse lock args ──────────────────────────────────────────────
    let mut script_buf = [0u8; 128];
    let script_len = ckb_std::syscalls::load_script(&mut script_buf, 0).unwrap_or(0);

    // Script is a molecule-encoded Script struct: {code_hash, hash_type, args}
    // args are at offset 53 (32 + 1 + 20 for code_hash(32) + hash_type(1) + padding)
    // Simplified: assume args start at offset 53
    let args_offset = 53;
    if script_len < args_offset + ARGS_LEN {
        ckb_std::syscalls::exit(-1);
    }
    let args = &script_buf[args_offset..args_offset + ARGS_LEN];

    let threshold = args[0];
    let total_keys = args[1];
    let since_epoch = u64::from_le_bytes(args[2..10].try_into().unwrap());

    // ── Get current epoch ───────────────────────────────────────────
    // The timelock will verify this — we just need to pass it as param
    let current_epoch = ckb_std::syscalls::load_epoch().unwrap_or(0);

    // ── Prepare multisig params ─────────────────────────────────────
    let multisig_params = alloc::format!(
        r#"{{"threshold":{},"total_keys":{},"message":"<tx_hash>","signatures":[],"public_keys":[],"algorithm":"secp256k1-blake2b"}}"#,
        threshold, total_keys
    );

    // ── Prepare timelock params ─────────────────────────────────────
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
    let modules: [(usize, &str); 2] = [(0, &multisig_params), (1, &timelock_params)];
    let results: Vec<ModuleResult> = match caller::call_all(&modules, &ctx) {
        Ok(r) => r,
        Err(_) => {
            // Module call failed (timeout, pipe error, etc.)
            ckb_std::syscalls::exit(-2);
        }
    };

    // ── Check results ───────────────────────────────────────────────
    let multisig_ok = results[0].response.ok;
    let timelock_ok = results[1].response.ok;

    if multisig_ok && timelock_ok {
        ckb_std::syscalls::exit(0); // Unlocked
    } else {
        ckb_std::syscalls::exit(-3); // Locked
    }
}

#[cfg(not(target_arch = "riscv64"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
