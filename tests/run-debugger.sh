#!/usr/bin/env bash
# run-debugger.sh — Integration test runner for ckb-spawn-kit modules.
#
# Validates: JSON schema, VM execution (no memory errors/crashes).
# Full E2E pipe tests require a spawn-caller binary (see examples/composed_lock/).
#
# Requires: ckb-debugger, riscv64 target, built modules in build/
#
# Usage:
#   ./tests/run-debugger.sh all           # run all tests
#   ./tests/run-debugger.sh multisig      # run multisig tests only

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$PROJECT_DIR/build}"
PASS=0
FAIL=0

green() { echo -e "\033[32m$*\033[0m"; }
red()   { echo -e "\033[31m$*\033[0m"; }

run_test() {
    local module="$1" tx_file="$2" desc="$3"

    if [ ! -f "$BUILD_DIR/$module" ]; then
        red "  SKIP: $BUILD_DIR/$module not built"
        ((FAIL++)) || true
        return
    fi
    if [ ! -f "$tx_file" ]; then
        red "  SKIP: $tx_file not found"
        ((FAIL++)) || true
        return
    fi

    local output
    output=$(ckb-debugger --tx-file "$tx_file" --bin "$BUILD_DIR/$module" --mode full 2>&1) || true

    # A valid test means the VM executed without memory errors or crashes.
    # "memory error" indicates VM failure (W^X violation, etc.)
    # "Check Fail" indicates schema validation failure
    # "panicked" indicates crash
    # Exit code -1 is normal for spawn modules without stdin in lock-script context
    if echo "$output" | grep -qE "memory error|Check Fail|panicked"; then
        red "  FAIL: $desc ($module)"
        echo "  $(echo "$output" | grep -E 'memory error|Check Fail|panicked')"
        ((FAIL++)) || true
    else
        green "  PASS: $desc ($module)"
        ((PASS++)) || true
    fi
}

echo "=== ckb-spawn-kit Integration Tests ==="
echo "  (validates fixture schema + VM execution)"
echo ""

# ─── spawn-kit-multisig ──────────────────────────────────────────────

test_multisig() {
    echo "--- spawn-kit-multisig ---"

    local ex="$PROJECT_DIR/examples/basic_multisig"

    run_test "spawn-kit-multisig" "$ex/tx.json" \
        "basic 2-of-2 multisig with threshold=1"

    run_test "spawn-kit-multisig" "$ex/insufficient_sigs.json" \
        "insufficient signatures fixture"

    run_test "spawn-kit-multisig" "$ex/key_count_mismatch.json" \
        "key count mismatch fixture"
}

# ─── spawn-kit-timelock ──────────────────────────────────────────────

test_timelock() {
    echo "--- spawn-kit-timelock ---"

    local ex="$PROJECT_DIR/examples/basic_multisig"

    run_test "spawn-kit-timelock" "$ex/timelock_pass.json" \
        "timelock epoch reached"

    run_test "spawn-kit-timelock" "$ex/timelock_fail.json" \
        "timelock epoch not reached"
}

# ─── spawn-kit-ratelimit ─────────────────────────────────────────────

test_ratelimit() {
    echo "--- spawn-kit-ratelimit ---"

    local ex="$PROJECT_DIR/examples/basic_multisig"

    run_test "spawn-kit-ratelimit" "$ex/ratelimit_pass.json" \
        "rate limit under threshold"

    run_test "spawn-kit-ratelimit" "$ex/ratelimit_blocked.json" \
        "rate limit exceeded"
}

# ─── Run what's requested ────────────────────────────────────────────

case "${1:-all}" in
    all)
        test_multisig
        test_timelock
        test_ratelimit
        ;;
    multisig)  test_multisig ;;
    timelock)  test_timelock ;;
    ratelimit) test_ratelimit ;;
    *)
        echo "Usage: $0 {all|multisig|timelock|ratelimit}"
        exit 1
        ;;
esac

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ] || exit 1
