//! Caller-side library for scripts that compose spawn-kit modules.

extern crate alloc;
use alloc::vec::Vec;
use core::ffi::CStr;

use crate::{CallContext, ErrorCode, Request, Response, PROTOCOL_MAGIC};

pub use ckb_std::ckb_constants::Source;
pub use ckb_std::syscalls;

pub struct ModuleResult {
    pub response: Response,
    pub pid: u64,
}

/// Spawn a module from CellDep, send a verify request, read response.
pub fn call_module(
    cell_dep_index: usize,
    params_json: &str,
    _ctx: &CallContext,
) -> Result<ModuleResult, ErrorCode> {
    let (r0, w0) = syscalls::pipe().map_err(|_| ErrorCode::PipeReadError)?;
    let (r1, w1) = syscalls::pipe().map_err(|_| ErrorCode::PipeReadError)?;

    let mut fds_vec: alloc::vec::Vec<u64> = alloc::vec::Vec::new();
    fds_vec.extend_from_slice(&[r1, w0]);
    fds_vec.push(0);

    let request_json = alloc::format!(
        r#"{{"magic":[83,75,75,84],"version":1,"action":"verify","params":{}}}"#,
        params_json
    );

    let argc: u64 = 1;
    let argv = [
        CStr::from_bytes_with_nul(b"spawn-kit-module\0").unwrap().as_ptr(),
    ];

    let mut child_pid: u64 = 0;
    let mut spgs = syscalls::SpawnArgs {
        argc,
        argv: argv.as_ptr() as *const *const i8,
        process_id: &mut child_pid as *mut u64,
        inherited_fds: fds_vec.as_ptr(),
    };

    syscalls::spawn(cell_dep_index, Source::CellDep, 0, 0, &mut spgs)
        .map_err(|_| ErrorCode::ChildProcessError)?;

    let pid = child_pid;
    syscalls::write(w1, request_json.as_bytes()).map_err(|_| ErrorCode::PipeWriteError)?;
    syscalls::close(w1).ok();

    let mut buf = [0u8; crate::MAX_FRAME_SIZE];
    let len = syscalls::read(r0, &mut buf).map_err(|_| ErrorCode::PipeReadError)?;
    syscalls::close(r0).ok();
    syscalls::close(r1).ok();
    syscalls::close(w0).ok();

    let exit_code = syscalls::wait(pid).map_err(|_| ErrorCode::ChildProcessError)?;

    let (response, _): (Response, _) =
        serde_json_core::from_slice(&buf[..len]).map_err(|_| ErrorCode::JsonParseError)?;

    if response.magic != PROTOCOL_MAGIC {
        return Err(ErrorCode::InvalidMagic);
    }
    if exit_code != 0 && response.ok {
        return Err(ErrorCode::ChildProcessError);
    }

    Ok(ModuleResult { response, pid })
}

/// Chain two modules: A's data field becomes B's params.
pub fn pipe_modules(
    cell_dep_a: usize, params_a: &str,
    cell_dep_b: usize, ctx: &CallContext,
) -> Result<ModuleResult, ErrorCode> {
    let result_a = call_module(cell_dep_a, params_a, ctx)?;
    let intermediate = result_a.response.data.as_ref()
        .map(|v| core::str::from_utf8(v).unwrap_or("{}")).unwrap_or("{}");
    call_module(cell_dep_b, intermediate, ctx)
}

/// Run multiple modules in parallel.
pub fn call_all(modules: &[(usize, &str)], ctx: &CallContext) -> Result<Vec<ModuleResult>, ErrorCode> {
    let mut results = Vec::with_capacity(modules.len());
    for (idx, params) in modules { results.push(call_module(*idx, params, ctx)?); }
    Ok(results)
}
