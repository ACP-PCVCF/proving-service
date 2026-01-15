# Reproducing Benchmark Results

## System Configuration

The benchmarks in this repository were generated on the following system:

### Hardware Specifications
- **CPU**: Intel Core i5-13600K (14 cores: 6 P-cores @ 3.5-5.1 GHz + 8 E-cores @ 2.6-3.9 GHz, 20 threads)
- **GPU**: NVIDIA GeForce RTX 3070 (8 GB VRAM)
- **RAM**: 32 GB DDR4 @ 3200 MHz
- **Storage**: 1 TB NVMe M.2 SSD
- **Operating System**: Ubuntu 24.04 LTS (kernel 6.8.0-90-generic)

### Software Versions
- **Rust toolchain**: 1.85.0 (includes rustc and cargo)
- **RISC Zero zkVM**: 3.0.4
- **Python**: 3.12.3

## Prerequisites

- Rust toolchain 1.85.0 (includes rustc and cargo)
- RISC Zero toolchain 3.0.4
- Python 3.12.3
- uv (Python dependency management)

## Build the Project

```bash
# Build the project in release mode
cargo build --release
```

## Quick Start: Running All Benchmarks

The easiest way to run all benchmarks is using the provided script:

```bash
# From the project root directory
./run_benchmarks.sh
```

This script will:
1. Generate benchmark data (20 base documents)
2. Run composition benchmark
3. Run aggregation benchmark
4. Run proof aggregation benchmark
5. Display where results are saved

**Output:**
All data is saved in a timestamped folder: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/`

## Manual Benchmark Execution

If you prefer to run benchmarks individually:

### Step 1: Generate Base Data

```bash
# From the project root directory
cargo test generate_benchmark_data -- --ignored --nocapture
```

**What this test does:**
- Generates 20 random base documents (ProofingDocuments)
- Saves them in `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/base_documents/`
- These documents are the input for all three benchmark tests

**Output:** Creates a new timestamped benchmark folder with base documents

### Step 2: Composition Benchmark

```bash
cargo test bench_composition -- --ignored --nocapture
```

**What this test does:**
- Reads the 20 base documents from the latest benchmark folder
- Generates 20 proofs using **proof composition** (recursive/nested proofs)
- Each proof includes and verifies the previous proof in a chain:
  - Proof 0: Processes document 0
  - Proof 1: Processes document 1 + verifies Proof 0
  - Proof 2: Processes document 2 + verifies Proof 1
  - ...
  - Proof 19: Processes document 19 + verifies Proof 18

**Output:**
- CSV: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/bench_composition.csv`
- Proofs: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/composition/comp_proof_X.json`

### Step 3: Aggregation Benchmark

```bash
cargo test bench_aggregation -- --ignored --nocapture
```

**What this test does:**
- Reads the 20 base documents from the latest benchmark folder
- Aggregates all TCE data from all 20 documents into a single document
- Generates 1 proof that processes all aggregated data at once

**Output:**
- CSV: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/bench_aggregation.csv`
- Aggregated document: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/aggregation/aggregated_document.json`
- Proof: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/aggregation/aggregation_proof.json`

### Step 4: Proof Aggregation Benchmark

```bash
cargo test bench_proofaggregation -- --ignored --nocapture
```

**What this test does:**
- Reads the 20 base documents from the latest benchmark folder
- Generates 20 independent proofs (one per document)
- Then generates 1 final proof that verifies all 20 proofs together
- CSV contains 21 rows:
  - Rows 0-19: Individual sequential proofs
  - Row 20: The aggregated proof that verifies all 20

**Output:**
- CSV: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/bench_proofaggregation.csv`
- Individual proofs: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/proof_aggregation/individual_proof_X.json`
- Aggregated document: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/proof_aggregation/proof_aggr_document.json`
- Final proof: `benchmarks/documents/benchmark_YYYYMMDD_HHMMSS/proof_aggregation/final_aggregated_proof.json`

## Generating Plots

### Install Python Dependencies

Before generating plots, install the required Python packages:

```bash
# Navigate to the plots directory
cd benchmarks/plots

# Install dependencies using uv
uv sync
```

This will install all required packages (pandas, matplotlib, seaborn, numpy) in an isolated environment.

### Option 1: Specify Benchmark Folder

```bash
# From benchmarks/plots directory
# Generate plots for a specific benchmark run
uv run python generate_plots.py ../documents/benchmark_20260115_143000
```

### Option 2: Auto-detect Latest Benchmark

```bash
# From benchmarks/plots directory
# Automatically use the most recent benchmark folder
uv run python generate_plots.py
```
