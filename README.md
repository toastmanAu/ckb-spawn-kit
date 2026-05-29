# ckb-spawn-kit — Composable On-Chain Script Modules

Pre-built, audited CKB script building blocks that compose via the Spawn syscall
and pipes. Each module does ONE thing, communicates over stdin/stdout JSON, and
deploys as an independent RISC-V Cell.

## Status

**Phase 1: Core protocol + 5 modules complete. On-chain diagnostics in progress.**

Off-chain validation: all tests pass. On-chain: `spawn()` and `pipe()` confirmed
working. `inherited_fds` (parent-to-child pipe connection) under investigation —
does not function on current testnet VM.

## Modules

| Module | Description | Error Range |
|---|---|---|
| **spawn-kit-core** | Protocol ABI, Request/Response types, caller library | 0x0000-0x0FFF |
| **spawn-kit-multisig** | M-of-N signature verification (algorithm-agnostic) | 0x1000-0x1FFF |
| **spawn-kit-timelock** | Epoch-based time-locked access control | 0x2000-0x2FFF |
| **spawn-kit-ratelimit** | Rate limiting with configurable windows | 0x3000-0x3FFF |
| **spawn-kit-access-control** | Capability-based access tokens | 0x4000-0x4FFF |
| **spawn-kit-escrow** | Conditional escrow with timelock + attestation | 0x5000-0x5FFF |

## Architecture

```
Your Lock Script
    │
    ├── spawn(multisig) ─── verify signatures
    ├── spawn(timelock) ─── check epoch
    ├── spawn(ratelimit) ── check rate
    └── spawn(escrow) ───── check release conditions
```

Each module is an independent Cell referenced via CellDep. The caller's Lock
Script spawns modules as child processes with cycle budgets. Modules receive
a JSON Request on stdin and write a JSON Response on stdout.

## Build & Test

```bash
# Prerequisites
rustup target add riscv64imac-unknown-none-elf
cargo install ckb-debugger

# Build all RISC-V modules
make modules

# Run host-target unit tests (19 tests)
make test

# Run ckb-debugger integration tests (7 tests)
make test-debugger

# Fuzzing (requires nightly Rust)
make fuzz-build
make fuzz-smoke
```

## Examples

| Example | Description |
|---|---|
| `examples/basic_multisig/` | Transaction fixtures for ckb-debugger testing |
| `examples/composed_lock/` | Full multisig + timelock compose via spawn |
| `examples/timelock_only/` | Minimal single-module spawn composition |
| `examples/diag/` | On-chain spawn/pipe diagnostic harness (v1-v13) |
| `examples/echo_child/` | Minimal child process (stdout-only, no stdin) |

## On-Chain Validation Results

Tests run against CKB testnet (local node: 192.168.68.134:8114).

### Confirmed Working

| Capability | Status | Method |
|---|---|---|
| RISC-V module builds | 5/5 modules | `riscv64imac-unknown-none-elf` |
| Host unit tests | 19/19 pass | `cargo test --workspace` |
| ckb-debugger tests | 7/7 pass | `ckb-debugger --tx-file` |
| Module deployment | Deployed | `ckb-cli deploy` |
| `spawn()` syscall | Working (v8) | Isolated spawn+wait test |
| `pipe()` syscall | Working (v7) | Pipe creation succeeds |
| `wait()` syscall | Working (v8) | Wait for child exit |
| Code hash resolution | Working | Via CellDep overlay (type_hash match) |
| Custom lock script execution | Working | Lock script loads and runs on-chain |
| Transaction pipeline | Working | Build → sign → send via curl RPC |

### Under Investigation

| Capability | Status | Detail |
|---|---|---|
| `inherited_fds` (stdin pipe) | **Not working** | `write()` to child stdin fails: OtherEndClosed (v7, v9, v11) |
| `inherited_fds` (stdout pipe) | **Not working** | `read()` from child stdout fails: no data (v13 self-spawn) |
| Write-before-spawn | **Deadlock** | Pipe data lost across spawn boundary (v10) |
| Full spawn composition | **Blocked** | Depends on inherited_fds fix |

### Root Cause Analysis

The CKB VM's `spawn()` syscall correctly creates child processes (confirmed
with spawn+wait). The `pipe()` syscall creates valid fd pairs. However,
`inherited_fds` — the mechanism that maps parent pipe file descriptors to
the child's stdin/stdout/stderr — does not connect the fds on the current
testnet VM. The child process receives default fds (0, 1, 2) instead of
the parent's pipe fds passed via `SpawnArgs.inherited_fds`.

This prevents the parent from writing request data to the child's stdin or
reading response data from the child's stdout. The spawn composition model
is architecturally correct but requires a VM update that properly supports
the full spawn+inherited_fds IPC model.

## Key Design Decisions

- **Stateless modules**: state lives in the caller's Cell data; modules are pure validators
- **No upgradeable Cells**: deploy a new version, users migrate voluntarily
- **Error code ranges**: each module owns 0x1000 codes for unique error attribution
- **JSON over pipes**: human-debuggable, no custom serialization format
- **Algorithm agnostic**: multisig delegates to per-algorithm check modules via spawn

## Roadmap

### Completed
- [x] Core protocol ABI (Request/Response types, error codes)
- [x] 5 modules (multisig, timelock, ratelimit, access-control, escrow)
- [x] Caller library (`call_module`, `pipe_modules`, `call_all`)
- [x] Host-target test suite (19 tests)
- [x] ckb-debugger integration tests (7 tests)
- [x] Fuzz harness (6 targets + corpus)
- [x] RISC-V build pipeline with W^X-compliant binaries
- [x] On-chain deployment pipeline (public testnet + local node)
- [x] Code hash resolution via CellDep overlay
- [x] On-chain spawn/pipe/wait syscall validation

### Blocked (VM dependency)
- [ ] Full spawn+pipes composition (requires inherited_fds fix)
- [ ] Cycle cost benchmarks for spawn + pipe + JSON round-trip
- [ ] First end-to-end composed lock on testnet

### Next
- [ ] Real secp256k1-blake2b verification in multisig module
- [ ] Witness access for spawned modules
- [ ] Capsule build integration
- [ ] Production audit
- [ ] Mainnet deployment

## Deployed Cells (CKB Testnet)

| Artifact | tx_hash | Size |
|----------|---------|------|
| spawn-kit-timelock (zero-locked) | `0xd927...9237:0` | 42K |
| spawn-kit-multisig | `0xf91b...0c8c:0` | 42K |
| spawn-diag-v13 (self-spawn test) | `0xd463...d5d3:0` | 9K |
| echo-child-v2 | `0xf580...abb8:0` | 9K |
| timelock-only-lock-fixed | `0x183f...72cd:0` | 22K |
