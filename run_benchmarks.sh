#!/bin/bash
set -e

NUM_DOCS=15
MODE="all"
SKIP_GEN=false
TIMEOUT=18000  # 300 minutes in seconds

while getopts "n:m:st:h" opt; do
  case $opt in
    n) NUM_DOCS=$OPTARG ;;
    m) MODE=$OPTARG ;;
    s) SKIP_GEN=true ;;
    t) TIMEOUT=$OPTARG ;;
    h)
      echo "Usage: $0 [-n NUM_DOCS] [-m MODE] [-s] [-t TIMEOUT]"
      echo "  -n  Number of documents (default: 15, must be 2^k-1 for tree)"
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

export BENCHMARK_NUM_DOCS=$NUM_DOCS

if [ "$SKIP_GEN" = false ]; then
    echo ""
    echo "Generating $NUM_DOCS benchmark documents"
    cargo test generate_benchmark_data -- --ignored --nocapture
else
    echo ""
    echo "Skipping document generation"
fi

run_benchmarks() {
    echo ""
    echo "Running benchmarks with $1 (timeout: ${TIMEOUT}s)"

    echo "[1/3] Composition"
    timeout $TIMEOUT cargo test bench_composition -- --ignored --nocapture || echo "TIMEOUT or FAILED: bench_composition"

    echo "[2/3] Proof aggregation"
    timeout $TIMEOUT cargo test bench_proofaggregation -- --ignored --nocapture || echo "TIMEOUT or FAILED: bench_proofaggregation"

    echo "[3/3] Tree aggregation"
    timeout $TIMEOUT cargo test bench_tree_aggregation -- --ignored --nocapture || echo "TIMEOUT or FAILED: bench_tree_aggregation"
}

# Order: baseline_precompiles, lazy, bls, baseline

if [[ "$MODE" == "baseline_precompiles" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=false
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=true
    run_benchmarks "baseline guest (RSA with precompiles)"
fi

if [[ "$MODE" == "lazy" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=false
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_benchmarks "lazy guest (deferred RSA verification)"
fi

if [[ "$MODE" == "bls" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=true
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_benchmarks "BLS guest (aggregate signatures with precompiles)"
fi

if [[ "$MODE" == "baseline" || "$MODE" == "all" ]]; then
    export USE_DUMMY_GUEST=false
    export USE_BLS_PRECOMPILES_GUEST=false
    export USE_BASELINE_GUEST=true
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_benchmarks "baseline guest (RSA without precompiles)"
fi

LATEST=$(ls -1 benchmarks/documents/ | grep "^benchmark_" | sort | tail -n 1)
echo ""
echo "Results in: benchmarks/documents/$LATEST"
