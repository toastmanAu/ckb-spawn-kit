#!/usr/bin/env bash
# run-debugger.sh — Integration test runner for ckb-spawn-kit modules.
#
# Requires: ckb-debugger, riscv64 target, built modules in build/
#
# Usage:
#   ./tests/run-debugger.sh all           # run all tests
#   ./tests/run-debugger.sh multisig      # run multisig tests only
#   ./tests/run-debugger.sh timelock      # run timelock tests only

set -euo pipefail

BUILD_DIR="${BUILD_DIR:-build}"
EXAMPLES_DIR="examples"
PASS=0
FAIL=0

green() { echo -e "\033[32m$*\033[0m"; }
red()   { echo -e "\033[31m$*\033[0m"; }

run_test() {
    local module="$1"
    local tx_file="$2"
    local mode="${3:-full}"

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

    echo "  Testing $module with $(basename "$tx_file")..."
    if ckb-debugger --tx-file "$tx_file" --bin "$BUILD_DIR/$module" --mode "$mode" 2>&1 | tail -5; then
        green "  PASS: $module ($(basename "$tx_file"))"
        ((PASS++)) || true
    else
        red "  FAIL: $module ($(basename "$tx_file"))"
        ((FAIL++)) || true
    fi
}

echo "=== ckb-spawn-kit Integration Tests ==="
echo ""

# ─── spawn-kit-multisig ───────────────────────────────────────────────

test_multisig() {
    echo "--- spawn-kit-multisig ---"

    # Basic 2-of-3 multisig
    run_test "spawn-kit-multisig" "$EXAMPLES_DIR/basic_multisig/tx.json" "full"

    # Edge case: insufficient signatures
    run_test "spawn-kit-multisig" "$EXAMPLES_DIR/basic_multisig/insufficient_sigs.json" "full"

    # Edge case: key count mismatch
    run_test "spawn-kit-multisig" "$EXAMPLES_DIR/basic_multisig/key_count_mismatch.json" "full"
}

# ─── spawn-kit-timelock ───────────────────────────────────────────────

test_timelock() {
    echo "--- spawn-kit-timelock ---"

    run_test "spawn-kit-timelock" "$EXAMPLES_DIR/basic_multisig/timelock_pass.json" "full"
    run_test "spawn-kit-timelock" "$EXAMPLES_DIR/basic_multisig/timelock_fail.json" "full"
}

# ─── spawn-kit-ratelimit ──────────────────────────────────────────────

test_ratelimit() {
    echo "--- spawn-kit-ratelimit ---"

    run_test "spawn-kit-ratelimit" "$EXAMPLES_DIR/basic_multisig/ratelimit_pass.json" "full"
    run_test "spawn-kit-ratelimit" "$EXAMPLES_DIR/basic_multisig/ratelimit_blocked.json" "full"
}

# ─── Run what's requested ─────────────────────────────────────────────

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
