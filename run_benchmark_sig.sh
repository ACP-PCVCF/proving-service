#!/bin/bash
set -e

MODE="all"
SKIP_GEN=false
TIMEOUT=18000  # 300 minutes in seconds

while getopts "m:st:h" opt; do
  case $opt in
    m) MODE=$OPTARG ;;
    s) SKIP_GEN=true ;;
    t) TIMEOUT=$OPTARG ;;
    h)
      echo "Usage: $0 [-m MODE] [-s] [-t TIMEOUT]"
      echo "  -m  Mode: baseline_precompiles, lazy, bls, baseline, or all (default: all)"
      echo "  -s  Skip document generation"
      echo "  -t  Timeout per variant in seconds (default: 18000 = 300 minutes)"
      exit 0 ;;
  esac
done

# Validate mode
if [[ "$MODE" != "all" && "$MODE" != "lazy" && "$MODE" != "baseline" && "$MODE" != "baseline_precompiles" && "$MODE" != "bls" ]]; then
  echo "Error: Mode must be 'all', 'lazy', 'baseline', 'baseline_precompiles', or 'bls'"
  exit 1
fi

[ ! -f "Cargo.toml" ] && echo "Run from project root" && exit 1

if [ "$SKIP_GEN" = false ]; then
    echo ""
    echo "Generating signature test documents (1, 4, 8, 12 signatures)"
    cargo test generate_signature_test_documents -- --ignored --nocapture
else
    echo ""
    echo "Skipping document generation"
fi

run_signature_test() {
    echo ""
    echo "Running signature tests with $1 (timeout: ${TIMEOUT}s)"
    timeout $TIMEOUT cargo test test_signatures -- --ignored --nocapture || echo "TIMEOUT or FAILED: test_signatures"
}

# Order: baseline_precompiles, lazy, bls, baseline

if [[ "$MODE" == "baseline_precompiles" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=false
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=true
    run_signature_test "baseline guest (RSA with precompiles)"
fi

if [[ "$MODE" == "lazy" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=false
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_signature_test "lazy guest (deferred RSA verification)"
fi

if [[ "$MODE" == "bls" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=true
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_signature_test "BLS guest (aggregate signatures with precompiles)"
fi

if [[ "$MODE" == "baseline" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=false
    export USE_BASELINE_GUEST=true
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_signature_test "baseline guest (RSA without precompiles)"
fi

LATEST=$(ls -1 benchmarks/documents/ | grep "^signature_test_" | sort | tail -n 1)
echo ""
echo "Results in: benchmarks/documents/$LATEST"
