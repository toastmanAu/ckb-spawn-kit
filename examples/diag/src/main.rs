// RISC-V binary entry point
#![cfg(target_arch = "riscv64")]
#![no_std]
#![no_main]

use ckb_std::default_alloc;
default_alloc!();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let code = spawn_diag::verify();
    ckb_std::syscalls::exit(code);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-99);
}
