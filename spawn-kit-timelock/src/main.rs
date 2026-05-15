#![no_std]
#![no_main]

use ckb_std::default_alloc;
default_alloc!();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    spawn_kit_timelock::entry()
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    ckb_std::syscalls::exit(-99);
}
