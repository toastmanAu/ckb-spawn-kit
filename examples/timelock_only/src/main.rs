// RISC-V binary entry point — not compiled on host targets.
#![cfg(target_arch = "riscv64")]
#![no_std]
#![no_main]

use ckb_std::default_alloc;
default_alloc!();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    match timelock_only_lock::verify() {
        Ok(()) => ckb_std::syscalls::exit(0),
        Err(code) => ckb_std::syscalls::exit(code),
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-99);
}
