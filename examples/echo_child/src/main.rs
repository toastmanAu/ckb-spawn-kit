// RISC-V binary - just writes to stdout and exits. No stdin read.
#![cfg(target_arch = "riscv64")]
#![no_std]
#![no_main]

use ckb_std::default_alloc;
default_alloc!();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Write a simple message to stdout (fd 1)
    let msg = b"ECHO_OK";
    ckb_std::syscalls::write(1, msg).ok();
    ckb_std::syscalls::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-99);
}
