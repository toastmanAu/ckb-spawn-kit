# 3. Spawn Composition Library

## Viability Assessment

### Summary
Pre-built, audited script building blocks using the Spawn syscall and pipes. Composable modules for: multisig, time-locks, rate limiting, access control, escrow, oracle reading, and token gating. The Spawn syscall shipped with Meepo hard fork (2025) but has almost no ecosystem libraries.

### What Exists (from graph analysis)
- **ckb-debugger:example:spawn** (community #104) — the canonical example, labeled "spawn syscall (ckb2023)", suggesting the syscall has been available since CKB2023 VM edition.
- **Edge from spawn → ckb-debugger:dsl** — the debugger has a DSL concept that spawn examples use, hinting at a proto-framework.
- **ckb-script-fuzzing-toolkit** (community #8, xxuejie) includes `spawn()` in `protobuf-ckb-syscalls` — spawn syscall is modeled in the fuzzing/simulation layer.
- **ckb-production-scripts** (communities #1, #49) contain xUDT RCE validators that exercise spawn-like patterns but aren't reusable modules.
- The graph shows spawn in testing/example contexts only — zero production library nodes.

### Gap Analysis
| Dimension | Status |
|---|---|
| Spawn syscall (protocol layer) | Shipped (Meepo hard fork, 2025) |
| Debugging/example code | Exists (ckb-debugger examples) |
| Fuzzing support for spawn | Exists (fuzzing toolkit) |
| Reusable multisig module | **Does not exist** |
| Reusable time-lock module | **Does not exist** |
| Rate limiting / access control | **Does not exist** |
| Escrow / oracle reading | **Does not exist** |
| Composition patterns & standards | **Does not exist** |
| Documentation beyond syscall spec | **Does not exist** |

### Viability Score: 9/10

**Strengths:**
- **Highest potential impact** of the three ideas. Spawn enables Unix-pipe-style script composition — it's the killer feature of CKB's RISC-V VM. Without libraries, the syscall is a ghost town.
- Clear, bounded initial scope: ship 3-4 modules (multisig, time-lock, rate-limit, access-control) and a composition guide.
- First-mover advantage is massive — whoever ships the first spawn library becomes the standard.
- The Meepo hard fork timing is ideal (2025) — the syscall is fresh, and there's a window before the core team or another dev fills this gap.
- Lower trust risk than Idea #2: spawn modules are composable building blocks, not a registry vouching for third-party code.
- The graph confirms the vacuum: spawn exists at the syscall layer, examples exist in the debugger, but nothing connects them to production use.

**Risks:**
- Spawn's IPC/pipe model is genuinely novel — there's no prior art to copy from. Design mistakes will compound across the ecosystem.
- RISC-V binary ABI stability across compiler versions is unproven. Modules compiled with different toolchains may not compose.
- Gas/cycle accounting for spawn chains is poorly understood. Deeply nested spawn calls might hit unexpected cycle limits.
- Small developer audience: maybe 20-50 devs actively writing CKB scripts, and only a subset will use spawn immediately.

**Key Dependencies:**
- Spawn syscall stability in `ckb2023` / `ckb2024` VM editions
- RISC-V toolchain maturity for CKB targets (Capsule, ckb-xc)
- ABI standardization across spawn modules (this library would define the standard)

**Recommended First Step:** Ship ONE module (multisig via spawn) with exhaustive documentation and a worked example that a developer can clone and deploy in 5 minutes. Prove the composition model works end-to-end before building the full library. The graph's `ckb-debugger:dsl` node suggests there may be DSL patterns worth studying.
