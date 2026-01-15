#!/bin/bash

set -e 

echo "Starting Benchmarks..."
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Please run this script from the project root directory"
    exit 1
fi

# Step 1: Generate benchmark data
echo "[1/4] Generating benchmark data..."
echo "------------------------------------------------------------------------"
cargo test generate_benchmark_data -- --ignored --nocapture
echo ""

# Step 2: Run composition benchmark
echo "[2/4] Running composition benchmark..."
echo "------------------------------------------------------------------------"
cargo test bench_composition -- --ignored --nocapture
echo ""

# Step 3: Run aggregation benchmark
echo "[3/4] Running aggregation benchmark..."
echo "------------------------------------------------------------------------"
cargo test bench_aggregation -- --ignored --nocapture
echo ""

# Step 4: Run proof aggregation benchmark
echo "[4/4] Running proof aggregation benchmark..."
echo "------------------------------------------------------------------------"
cargo test bench_proofaggregation -- --ignored --nocapture
echo ""

# Find the latest benchmark folder
LATEST_BENCHMARK=$(ls -1 benchmarks/documents/ | grep "^benchmark_" | sort | tail -n 1)

echo "Benchmarks Complete!"
echo ""
echo "Results saved to: benchmarks/documents/$LATEST_BENCHMARK"
echo ""
echo "To generate plots, run:"
echo "  cd benchmarks/plots"
echo "  python generate_plots.py ../documents/$LATEST_BENCHMARK"
echo ""
