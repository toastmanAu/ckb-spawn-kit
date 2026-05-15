# ckb-spawn-kit — Architecture

## Problem

CKB scripts are monolithic. A multisig lock script contains signature verification, threshold logic, and key management in one binary. An xUDT type script bundles issuance, transfer validation, and regulatory compliance. Every project rewrites these from scratch because there's no way to compose scripts like Unix pipes.

The Spawn syscall (Meepo hard fork, 2025) provides the mechanism — `spawn()` + `pipe()` lets one on-chain script execute another as a child process with IPC. But the ecosystem has zero reusable modules built on it.

## Solution

**ckb-spawn-kit** is a library of pre-built, audited script modules that compose via spawn+pipes. Each module does ONE thing, takes arguments over stdin, and writes results to stdout — like Unix coreutils but for CKB on-chain scripts.

## Composition Model

```
┌─────────────────────────────────────────┐
│  Your Lock Script (the "caller")        │
│                                         │
│  fn verify() {                          │
│    let sig_check = spawn(multisig,      │
│      args = [pubkeys, threshold]);       │
│    let time_ok  = spawn(timelock,       │
│      args = [since_epoch]);             │
│    let rate_ok  = spawn(ratelimit,      │
│      args = [max_per_hour]);            │
│                                         │
│    pipe_wait_all([sig_check, time_ok,   │
│                    rate_ok])            │
│  }                                      │
└──────────┬──────────┬──────────┬────────┘
           │ spawn    │ spawn    │ spawn
           ▼          ▼          ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │ multisig │ │ timelock │ │ ratelimit│
    │ module   │ │ module   │ │ module   │
    └──────────┘ └──────────┘ └──────────┘
```

## Spawn IPC Protocol

Every module follows this ABI:

```
stdin  → JSON-serialized arguments (one line)
stdout → JSON-serialized result  (one line)
stderr → (unused, reserved for debugging)
exit code → 0 = success, non-zero = error code
```

Argument format (stdin):
```json
{
  "action": "verify" | "install" | "metadata",
  "params": { ... module-specific ... },
  "context": {
    "tx_hash": "<hex>",
    "cell_index": <uint>,
    "script_hash": "<hex>"
  }
}
```

Result format (stdout):
```json
{
  "ok": true | false,
  "code": 0,
  "data": { ... module-specific return values ... },
  "cycles": 123456
}
```

Error format:
```json
{
  "ok": false,
  "code": <error_code>,
  "reason": "<human-readable>",
  "data": null
}
```

## Error Code Ranges

| Range | Module |
|---|---|
| 0x0000-0x0FFF | spawn-kit-core (protocol errors) |
| 0x1000-0x1FFF | spawn-kit-multisig |
| 0x2000-0x2FFF | spawn-kit-timelock |
| 0x3000-0x3FFF | spawn-kit-ratelimit |
| 0x4000-0x4FFF | spawn-kit-access-control |
| 0x5000-0x5FFF | spawn-kit-escrow |

## On-Chain Deployment

Each module compiles to a RISC-V binary, deployed as a Cell:

```
Cell {
  capacity: <binary size in bytes>,
  data: <RISC-V ELF binary>,
  type_script: SpawnModuleType,  // marks it as a spawn-kit module
  lock: <anyone-can-spend or null>
}
```

Callers reference modules via Cell Deps + Type Script lookup — they never embed module code. To upgrade a module, deploy a new Cell with a new version and update the Type Script reference.

## Security Model

1. **Modules are read-only to caller state** — spawn creates an isolated VM, modules can't access the caller's Cell data except through explicit pipe data.
2. **Modules validate, callers authorize** — a module says "signatures are valid" or "rate limit not exceeded", but the caller's Lock Script makes the final spend decision.
3. **No upgradeable modules** — deployed Cells are immutable. To fix a bug, deploy a new version. Old Cells remain spendable but users should migrate.
4. **Cycle accounting** — each spawn costs cycles. Callers set cycle budgets per child process.
