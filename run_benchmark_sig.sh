#!/bin/bash
set -e

MODE="all"
SKIP_GEN=false

while getopts "m:sh" opt; do
  case $opt in
    m) MODE=$OPTARG ;;
    s) SKIP_GEN=true ;;
    h)
      echo "Usage: $0 [-m MODE] [-s]"
      echo "  -m  Mode: lazy, baseline, baseline_precompiles, or all (default: all)"
      echo "       Examples: -m lazy, -m baseline, -m baseline_precompiles"
      echo "  -s  Skip document generation"
      exit 0 ;;
  esac
done

# Validate mode
if [[ "$MODE" != "all" && "$MODE" != "lazy" && "$MODE" != "baseline" && "$MODE" != "baseline_precompiles" ]]; then
  echo "Error: Mode must be 'all', 'lazy', 'baseline', or 'baseline_precompiles'"
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
    echo "Running signature tests with $1"
    cargo test test_signatures -- --ignored --nocapture
}

if [[ "$MODE" == "baseline_precompiles" || "$MODE" == "all" ]]; then
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=true
    run_signature_test "baseline guest (with precompiles)"
fi

if [[ "$MODE" == "lazy" || "$MODE" == "all" ]]; then
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_signature_test "lazy guest"
fi

if [[ "$MODE" == "baseline" || "$MODE" == "all" ]]; then
    export USE_BASELINE_GUEST=true
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_signature_test "baseline guest (no precompiles)"
fi

LATEST=$(ls -1 benchmarks/documents/ | grep "^signature_test_" | sort | tail -n 1)
echo ""
echo "Results in: benchmarks/documents/$LATEST"
