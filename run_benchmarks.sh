#!/bin/bash
set -e

NUM_DOCS=15
MODE="lazy"

while getopts "n:m:h" opt; do
  case $opt in
    n) NUM_DOCS=$OPTARG ;;
    m) MODE=$OPTARG ;;
    h)
      echo "Usage: $0 [-n NUM_DOCS] [-m MODE]"
      echo "  -n  Number of documents (default: 15, must be 2^k-1 for tree)"
      echo "  -m  Mode: both, lazy, or baseline (default: lazy)"
      exit 0 ;;
  esac
done

# Validate mode
if [[ "$MODE" != "both" && "$MODE" != "lazy" && "$MODE" != "baseline" ]]; then
  echo "Error: Mode must be 'both', 'lazy', or 'baseline'"
  exit 1
fi

[ ! -f "Cargo.toml" ] && echo "Run from project root" && exit 1

export BENCHMARK_NUM_DOCS=$NUM_DOCS

echo ""
echo "Generating $NUM_DOCS benchmark documents"
cargo test generate_benchmark_data -- --ignored --nocapture

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

if [[ "$MODE" == "lazy" || "$MODE" == "both" ]]; then
    export USE_BASELINE_GUEST=false
    run_benchmarks "lazy guest"
fi

if [[ "$MODE" == "baseline" || "$MODE" == "both" ]]; then
    export USE_BASELINE_GUEST=true
    run_benchmarks "baseline guest"
fi

LATEST=$(ls -1 benchmarks/documents/ | grep "^benchmark_" | sort | tail -n 1)
echo ""
echo "Results in: benchmarks/documents/$LATEST"
