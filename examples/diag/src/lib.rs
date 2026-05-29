#![no_std]

extern crate alloc;

/// Self-spawn test: parent spawns itself as child, child writes to stdout.
/// Exit codes:
///   0  = parent: got child data — pipes work!
///   -1 = pipe() failed
///   -2 = spawn() failed
///   -3 = write() failed
///   -4 = read() failed
///   -5 = wait() failed
pub fn verify() -> i8 {
    use ckb_std::ckb_constants::Source;
    use core::ffi::CStr;
    use ckb_std::syscalls;

    let pid = syscalls::process_id();

    if pid == 0 {
        // PARENT: spawn child, read from it
        let (r0, w0) = match syscalls::pipe() { Ok(p) => p, Err(_) => return -1 };
        let (_r1, _w1) = match syscalls::pipe() { Ok(p) => p, Err(_) => return -1 };

        let spawn_argv = [CStr::from_bytes_with_nul(b"self-spawn\0").unwrap()];
        let mut child_pid: u64 = 0;
        let argv: alloc::vec::Vec<*const i8> = spawn_argv.iter()
            .map(|e| e.as_ptr() as *const i8).collect();

        let mut fds_vec: alloc::vec::Vec<u64> = alloc::vec::Vec::new();
        fds_vec.extend_from_slice(&[_r1, w0]);
        fds_vec.push(0);

        let mut spgs = syscalls::SpawnArgs {
            argc: 1, argv: argv.as_ptr(),
            process_id: &mut child_pid as *mut u64,
            inherited_fds: fds_vec.as_ptr(),
        };

        // Spawn self from CellDep #0 (which is this diag binary)
        match syscalls::spawn(0, Source::CellDep, 0, 0, &mut spgs) {
            Ok(_) => {},
            Err(_) => return -2,
        }

        // Read from child's stdout
        let mut buf = [0u8; 64];
        let n = match syscalls::read(r0, &mut buf) {
            Ok(n) => n,
            Err(_) => return -4,
        };
        syscalls::close(r0).ok();
        syscalls::close(_r1).ok();
        syscalls::close(w0).ok();

        match syscalls::wait(child_pid) {
            Ok(_) => {},
            Err(_) => return -5,
        }

        if &buf[..n] == b"CHILD_OK" { 0 } else { -10 }
    } else {
        // CHILD: write to stdout (fd 1) and exit
        let msg = b"CHILD_OK";
        match syscalls::write(1, msg) {
            Ok(_) => {},
            Err(_) => syscalls::exit(-3),
        };
        syscalls::exit(0);
    }
}
