# ckb-spawn-kit — Composable On-Chain Script Modules

Pre-built, audited CKB script building blocks that compose via the Spawn syscall
and pipes. Each module does ONE thing, communicates over stdin/stdout JSON, and
deploys as an independent RISC-V Cell.

## Status

**Phase 1: Core protocol + 5 modules scaffolded. API verified against ckb-std
native.rs (Sep 2025) and ckb-core spawn.rs / pipe.rs (Mar 2025).**

Ready for: Capsule build integration, ckb-debugger testing, audit preparation.

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

## Real ckb-std API (verified)

```rust
pub fn spawn(index: usize, source: Source, place: usize, bounds: usize,
             spgs: &mut SpawnArgs) -> Result<(), SysError>;
pub fn pipe() -> Result<(u64, u64), SysError>;   // → (read_fd, write_fd)
pub fn read(fd: u64, buffer: &mut [u8]) -> Result<usize, SysError>;
pub fn write(fd: u64, buffer: &[u8]) -> Result<usize, SysError>;
pub fn wait(pid: u64) -> Result<i8, SysError>;
pub fn close(fd: u64) -> Result<(), SysError>;
pub fn process_id() -> u64;
pub fn inherited_fds(fds: &mut [u64]) -> u64;
pub fn exit(code: i8) -> !;
```

## Prerequisites

```bash
rustup target add riscv64imac-unknown-none-elf
cargo install ckb-debugger    # for testing
cargo install ckb-capsule      # for deployment
cargo install ckb-cli          # for on-chain tx
```

## Build

```bash
make modules        # Build all RISC-V modules → build/
make test           # Run host-target unit tests
```

## Test with ckb-debugger

```bash
# Test multisig module in isolation
make test-debugger-multisig

# Or manually:
ckb-debugger --tx-file examples/basic_multisig/tx.json \
             --bin build/spawn-kit-multisig \
             --mode full
```

## Deploy

```bash
# 1. Build the module
make spawn-kit-multisig

# 2. Deploy Cell with the binary
ckb-cli tx build-and-send \
  --from-account <account> \
  --to-address <deploy-addr> \
  --capacity $(stat -c%s build/spawn-kit-multisig) \
  --data-binary build/spawn-kit-multisig

# 3. Record CellDep reference → use in your Lock Script
```

## Key Design Decisions

- **Stateless modules**: state lives in the caller's Cell data; modules are pure validators
- **No upgradeable Cells**: deploy a new version, users migrate voluntarily
- **Error code ranges**: each module owns 0x1000 codes for unique error attribution
- **JSON over pipes**: human-debuggable, no custom serialization format
- **Algorithm agnostic**: multisig delegates to per-algorithm check modules via spawn

## Next Steps (not yet implemented)

1. Capsule build configuration for each module
2. ckb-debugger integration tests with real tx.json fixtures
3. Cycle cost benchmarks for spawn + pipe + JSON round-trip
4. Audit preparation: fuzz harness using ckb-script-fuzzing-toolkit
5. First on-chain deployment to CKB testnet
