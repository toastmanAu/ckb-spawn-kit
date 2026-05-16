# ckb-spawn-kit build system
#
# Requires:
#   Rust + riscv64 target: rustup target add riscv64imac-unknown-none-elf
#   ckb-debugger: cargo install ckb-debugger
#   ckb-cli: cargo install ckb-cli

TARGET     := riscv64imac-unknown-none-elf
BUILD_DIR  := build
MODULES    := spawn-kit-multisig spawn-kit-timelock spawn-kit-ratelimit \
              spawn-kit-access-control spawn-kit-escrow

.PHONY: all build modules test clean deploy help

all: modules

# ── Modules (RISC-V binaries) ─────────────────────────────────────────

modules:
	@mkdir -p $(BUILD_DIR)
	@for m in $(MODULES); do \
		echo "Building $$m..."; \
		cd $$m && cargo build --target $(TARGET) --release 2>&1 | tail -3; cd ..; \
		cp $$m/target/$(TARGET)/release/$$m $(BUILD_DIR)/ 2>/dev/null || true; \
	done
	@echo "Done. Binaries in $(BUILD_DIR)/"

# Single module targets
spawn-kit-multisig:
	cd spawn-kit-multisig && cargo build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	cp spawn-kit-multisig/target/$(TARGET)/release/spawn-kit-multisig $(BUILD_DIR)/

spawn-kit-timelock:
	cd spawn-kit-timelock && cargo build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	cp spawn-kit-timelock/target/$(TARGET)/release/spawn-kit-timelock $(BUILD_DIR)/

spawn-kit-ratelimit:
	cd spawn-kit-ratelimit && cargo build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	cp spawn-kit-ratelimit/target/$(TARGET)/release/spawn-kit-ratelimit $(BUILD_DIR)/

spawn-kit-access-control:
	cd spawn-kit-access-control && cargo build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	cp spawn-kit-access-control/target/$(TARGET)/release/spawn-kit-access-control $(BUILD_DIR)/

spawn-kit-escrow:
	cd spawn-kit-escrow && cargo build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	cp spawn-kit-escrow/target/$(TARGET)/release/spawn-kit-escrow $(BUILD_DIR)/

# ── Testing ───────────────────────────────────────────────────────────

test:
	cargo test --workspace

# Run all ckb-debugger integration tests (validates fixtures + VM execution)
test-debugger: modules
	@bash tests/run-debugger.sh all

# Run a specific module's debugger tests
test-debugger-multisig: modules
	@bash tests/run-debugger.sh multisig

test-debugger-timelock: modules
	@bash tests/run-debugger.sh timelock

test-debugger-ratelimit: modules
	@bash tests/run-debugger.sh ratelimit

# Run all tests (unit + debugger integration)
test-all: test test-debugger

# ── Fuzzing ───────────────────────────────────────────────────────────

# Requires: nightly Rust (rustup toolchain install nightly)
fuzz-build:
	cd fuzz && cargo +nightly fuzz build

# Quick smoke test (100 iterations, <1s each)
fuzz-smoke:
	cd fuzz && for t in protocol multisig timelock ratelimit access_control escrow; do \
		cargo +nightly fuzz run $$t -- -runs=100 -max_total_time=2; \
	done

# Run a specific fuzz target (e.g. make fuzz-multisig RUNS=10000)
fuzz-%:
	cd fuzz && cargo +nightly fuzz run $* -- -runs=$(or $(RUNS),10000)

# ── Code quality ──────────────────────────────────────────────────────

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

# ── Clean ──────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -rf $(BUILD_DIR)

# ── Help ───────────────────────────────────────────────────────────────

help:
	@echo "ckb-spawn-kit — composable on-chain scripts via Spawn syscall"
	@echo ""
	@echo "Targets:"
	@echo "  make all            — build all RISC-V modules"
	@echo "  make <module-name>  — build a single module (e.g. make spawn-kit-multisig)"
	@echo "  make test            — run unit tests"
	@echo "  make test-debugger   — run all ckb-debugger integration tests"
	@echo "  make test-all        — run unit + debugger integration tests"
	@echo "  make fuzz-build      — build all fuzz targets (requires nightly)"
	@echo "  make fuzz-smoke      — quick fuzz smoke test (100 iterations each)"
	@echo "  make fuzz-multisig   — fuzz a specific module (set RUNS env var)"
	@echo "  make lint            — clippy + fmt check"
	@echo "  make clean           — remove build artifacts"
