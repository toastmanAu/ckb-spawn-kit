// This binary entry point is only for RISC-V cross-compilation.
// On host targets, only the library is used (via `cargo test`).
#![cfg(target_arch = "riscv64")]
#![no_std]
#![no_main]

use ckb_std::default_alloc;
default_alloc!();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    spawn_kit_multisig::entry()
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-99);
}
