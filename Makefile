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

# Run a module through ckb-debugger with the tx.json fixture
test-debugger-multisig: spawn-kit-multisig
	ckb-debugger --tx-file examples/basic_multisig/tx.json \
	             --bin $(BUILD_DIR)/spawn-kit-multisig \
	             --mode full

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
	@echo "  make test           — run unit tests (host target)"
	@echo "  make test-debugger-*  — run through ckb-debugger"
	@echo "  make lint           — clippy + fmt check"
	@echo "  make clean          — remove build artifacts"
