#!/bin/bash
set -e

NUM_DOCS=15
MODE="all"
SKIP_GEN=false

while getopts "n:m:sh" opt; do
  case $opt in
    n) NUM_DOCS=$OPTARG ;;
    m) MODE=$OPTARG ;;
    s) SKIP_GEN=true ;;
    h)
      echo "Usage: $0 [-n NUM_DOCS] [-m MODE] [-s]"
      echo "  -n  Number of documents (default: 15, must be 2^k-1 for tree)"
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
    echo "Running benchmarks with $1"

    echo "[1/3] Composition"
    cargo test bench_composition -- --ignored --nocapture

    echo "[2/3] Proof aggregation"
    cargo test bench_proofaggregation -- --ignored --nocapture

    echo "[3/3] Tree aggregation"
    cargo test bench_tree_aggregation -- --ignored --nocapture
}

if [[ "$MODE" == "baseline_precompiles" || "$MODE" == "all" ]]; then
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=true
    run_benchmarks "baseline guest (with precompiles)"
fi

if [[ "$MODE" == "lazy" || "$MODE" == "all" ]]; then
    export USE_BASELINE_GUEST=false
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_benchmarks "lazy guest (no precompiles)"
fi

if [[ "$MODE" == "baseline" || "$MODE" == "all" ]]; then
    export USE_BASELINE_GUEST=true
    export USE_BASELINE_PRECOMPILES_GUEST=false
    run_benchmarks "baseline guest (no precompiles)"
fi

LATEST=$(ls -1 benchmarks/documents/ | grep "^benchmark_" | sort | tail -n 1)
echo ""
echo "Results in: benchmarks/documents/$LATEST"
