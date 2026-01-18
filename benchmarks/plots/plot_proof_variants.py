#!/usr/bin/env python3
"""
Plot script for comparing proof variants: Composition, Proof Aggregation, and Tree Aggregation.
Generates plots for proof time, file size, and cycle counts from baseline benchmark data.
"""

import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
import seaborn as sns
import sys
from pathlib import Path
from typing import Dict, Optional, Tuple

# Configure plotting style globally
plt.style.use('seaborn-v0_8-whitegrid')
plt.rcParams.update({
    'font.size': 14,
    'axes.titlesize': 18,
    'axes.labelsize': 16,
    'legend.fontsize': 12,
    'xtick.labelsize': 12,
    'ytick.labelsize': 12
})
sns.set_palette('Spectral')

# Global variables for paths
CSV_FILES = {}
OUTPUT_DIR = None


def find_csv_files(benchmark_path: str) -> bool:
    """
    Find the three baseline CSV files in the benchmark run directory.

    Args:
        benchmark_path: Path to benchmark run (e.g., '../documents/benchmark_20260118_032016')

    Returns:
        True if files were found, False otherwise
    """
    global CSV_FILES, OUTPUT_DIR

    run_dir = Path(benchmark_path).resolve()

    if not run_dir.exists():
        print(f"Error: Benchmark directory '{run_dir}' does not exist")
        return False

    if not run_dir.is_dir():
        print(f"Error: '{run_dir}' is not a directory")
        return False

    # Look for the three baseline CSV files
    csv_names = {
        'composition': 'bench_composition_baseline.csv',
        'proofaggregation': 'bench_proofaggregation_baseline.csv',
        'tree_aggregation': 'bench_tree_aggregation_baseline.csv',
    }

    found_files = {}
    for key, filename in csv_names.items():
        csv_path = run_dir / filename
        if csv_path.exists():
            found_files[key] = csv_path
            print(f"Found {key}: {filename}")
        else:
            print(f"Warning: {filename} not found")

    if not found_files:
        print(f"Error: No CSV files found in '{run_dir}'")
        print("Expected files: bench_composition_baseline.csv, bench_proofaggregation_baseline.csv, bench_tree_aggregation_baseline.csv")
        return False

    CSV_FILES.update(found_files)

    # Set output directory to plots subfolder in the benchmark run directory
    OUTPUT_DIR = run_dir / "variant_plots"
    OUTPUT_DIR.mkdir(exist_ok=True)
    print(f"\nOutput directory: {OUTPUT_DIR}")

    return True


def load_csv_data(file_path: Path) -> Optional[pd.DataFrame]:
    """Load CSV data with error handling."""
    try:
        df = pd.read_csv(file_path)
        print(f"Loaded {file_path.name}: {len(df)} rows")
        return df
    except Exception as e:
        print(f"Error: Could not load data from '{file_path}': {e}")
        return None


def prepare_dataframe(df: pd.DataFrame) -> pd.DataFrame:
    """Add computed columns for easier plotting."""
    df = df.copy()
    df['input_size_MB'] = df['input_size'] / 1_000_000
    df['output_size_MB'] = df['output_size'] / 1_000_000
    df['total_cycles_M'] = df['total_cycles'] / 1_000_000
    df['paging_cycles_K'] = df['paging_cycles'] / 1_000
    df['user_cycles_K'] = df['user_cycles'] / 1_000
    df['reserved_cycles_K'] = df['reserved_cycles'] / 1_000
    df['total_cycles_K'] = df['total_cycles'] / 1_000
    return df


def get_dynamic_ylim(data: pd.Series, margin: float = 0.1) -> Tuple[float, float]:
    """Calculate dynamic y-axis limits with margin, handling NaN/Inf."""
    clean_data = data.replace([np.inf, -np.inf], np.nan).dropna()

    if len(clean_data) == 0 or clean_data.max() == 0:
        return (0, 1)

    max_val = clean_data.max()
    return (0, max_val * (1 + margin))


def setup_grid_and_ticks(ax, xlim: Tuple[float, float]):
    """Configure grid and tick marks for an axis."""
    ax.set_xlim(xlim)
    ax.xaxis.set_major_locator(ticker.MultipleLocator(2))
    ax.xaxis.set_minor_locator(ticker.MultipleLocator(1))
    ax.grid(True, which='major', linestyle='-', linewidth=0.7, alpha=0.7)
    ax.grid(True, which='minor', linestyle=':', linewidth=0.5, alpha=0.5)
    ax.ticklabel_format(style='plain', axis='y')


def save_figure(filename: str):
    """Save figure in multiple formats to the output directory."""
    global OUTPUT_DIR
    output_path = OUTPUT_DIR if OUTPUT_DIR else Path('.')

    plt.tight_layout(rect=[0, 0.03, 1, 0.95])
    png_file = output_path / f'{filename}.png'
    pdf_file = output_path / f'{filename}.pdf'

    plt.savefig(png_file, dpi=300, bbox_inches='tight')
    plt.savefig(pdf_file, dpi=300, bbox_inches='tight')
    print(f"Saved {png_file.name} and {pdf_file.name}")


def plot_proof_time_comparison(dfs: Dict[str, pd.DataFrame]):
    """Generate comparison plot for proof time across all three variants."""
    print("\n[1/3] Generating: Proof Time Comparison...")

    fig, ax = plt.subplots(figsize=(14, 8))
    fig.suptitle('Proof Time Comparison: Baseline Variants',
                 fontsize=22, fontweight='bold')

    colors = {
        'composition': 'royalblue',
        'proofaggregation': 'forestgreen',
        'tree_aggregation': 'darkorange'
    }

    labels = {
        'composition': 'Sequential (Depth)',
        'proofaggregation': 'Proof Aggregation (Width)',
        'tree_aggregation': 'Tree Aggregation (Depth + Width)'
    }

    markers = {
        'composition': 'o',
        'proofaggregation': 's',
        'tree_aggregation': '^'
    }

    all_times = []
    max_run_id = 0

    for key, df in dfs.items():
        x = df['run_id']
        y = df['proof_time']
        ax.plot(x, y, label=labels[key], color=colors[key],
                linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_times.extend(y.tolist())
        max_run_id = max(max_run_id, x.max())

    ax.set_title('Proof Time per Run ID', fontsize=18, pad=15)
    ax.set_xlabel('Run ID', fontsize=16)
    ax.set_ylabel('Proof Time (Seconds)', fontsize=16)
    ax.set_ylim(get_dynamic_ylim(pd.Series(all_times)))
    ax.legend(loc='best', frameon=True, shadow=True)

    xlim = (0, max_run_id + 1)
    setup_grid_and_ticks(ax, xlim)

    save_figure('proof_time_variants')
    plt.close()


def plot_file_size_comparison(dfs: Dict[str, pd.DataFrame]):
    """Generate comparison plots for input and output file sizes."""
    print("\n[2/3] Generating: File Size Comparison...")

    fig, axes = plt.subplots(1, 2, figsize=(18, 7))
    fig.suptitle('File Size Comparison: Baseline Variants',
                 fontsize=22, fontweight='bold')

    colors = {
        'composition': 'royalblue',
        'proofaggregation': 'forestgreen',
        'tree_aggregation': 'darkorange'
    }

    labels = {
        'composition': 'Sequential (Depth)',
        'proofaggregation': 'Proof Aggregation (Width)',
        'tree_aggregation': 'Tree Aggregation (Depth + Width)'
    }

    markers = {
        'composition': 'o',
        'proofaggregation': 's',
        'tree_aggregation': '^'
    }

    all_input_sizes = []
    all_output_sizes = []
    max_run_id = 0

    # Plot input sizes
    for key, df in dfs.items():
        x = df['run_id']
        y = df['input_size_MB']
        axes[0].plot(x, y, label=labels[key], color=colors[key],
                     linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_input_sizes.extend(y.tolist())
        max_run_id = max(max_run_id, x.max())

    axes[0].set_title('Input (Host) Size per Run ID', fontsize=16, pad=12)
    axes[0].set_xlabel('Run ID', fontsize=14)
    axes[0].set_ylabel('Input Size (MB)', fontsize=14)
    axes[0].set_ylim(get_dynamic_ylim(pd.Series(all_input_sizes)))
    axes[0].legend(loc='best', frameon=True, shadow=True)
    xlim = (0, max_run_id + 1)
    setup_grid_and_ticks(axes[0], xlim)

    # Plot output sizes
    for key, df in dfs.items():
        x = df['run_id']
        y = df['output_size_MB']
        axes[1].plot(x, y, label=labels[key], color=colors[key],
                     linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_output_sizes.extend(y.tolist())

    axes[1].set_title('Proof (Output) Size per Run ID', fontsize=16, pad=12)
    axes[1].set_xlabel('Run ID', fontsize=14)
    axes[1].set_ylabel('Proof Size (MB)', fontsize=14)
    axes[1].set_ylim(get_dynamic_ylim(pd.Series(all_output_sizes)))
    axes[1].legend(loc='best', frameon=True, shadow=True)
    setup_grid_and_ticks(axes[1], xlim)

    save_figure('file_size_variants')
    plt.close()


def plot_stacked_bars(dfs: Dict[str, pd.DataFrame]):
    """Generate stacked bar charts comparing total proof time and proof size across variants."""
    print("\n[3/5] Generating: Stacked Bar Charts - Proving Time...")

    colors = {
        'composition': sns.color_palette('Blues', n_colors=35),
        'proofaggregation': sns.color_palette('Greens', n_colors=35),
        'tree_aggregation': sns.color_palette('Oranges', n_colors=35)
    }

    labels = {
        'composition': 'Sequential (Depth)',
        'proofaggregation': 'Proof Aggregation (Width)',
        'tree_aggregation': 'Tree Aggregation (Depth + Width)'
    }

    # Calculate max rows for padding
    max_rows = max(len(df) for df in dfs.values())

    # Prepare time data
    time_data = {}
    for key, df in dfs.items():
        time_data[labels[key]] = df['proof_time'].tolist()

    # Pad data
    padded_time = {}
    for name, data in time_data.items():
        padded_time[name] = data + [0] * (max_rows - len(data))
    df_time = pd.DataFrame(padded_time)

    # Plot 1: Proving Time Total
    fig1, ax1 = plt.subplots(figsize=(12, 8))
    bar_names = list(df_time.columns)
    x_pos = np.arange(len(bar_names))
    bar_width = 0.6

    bottom_time = np.zeros(len(bar_names))

    for i in range(max_rows):
        current_layer = df_time.iloc[i].values
        for j, category_name in enumerate(bar_names):
            if category_name == labels['composition']:
                segment_color = colors['composition'][i %
                                                      len(colors['composition'])]
            elif category_name == labels['proofaggregation']:
                segment_color = colors['proofaggregation'][i %
                                                           len(colors['proofaggregation'])]
            else:
                segment_color = colors['tree_aggregation'][i %
                                                           len(colors['tree_aggregation'])]

            if current_layer[j] > 0:
                ax1.bar(x_pos[j], current_layer[j], bottom=bottom_time[j],
                        color=segment_color, width=bar_width, linewidth=0.2,
                        edgecolor='black', label=f'Run ID {i+1}' if j == 0 else '')
        bottom_time += current_layer

    ax1.set_title('Total Proving Time Comparison',
                  fontsize=18, fontweight='bold', pad=20)
    ax1.set_xlabel('Proof Variant')
    ax1.set_ylabel('Time (Seconds)')
    ax1.set_ylim(get_dynamic_ylim(pd.Series(bottom_time)))
    ax1.set_xticks(x_pos)
    ax1.set_xticklabels(bar_names, rotation=15, ha='right')
    ax1.grid(axis='y', linestyle='-', linewidth=0.7, alpha=0.7)
    ax1.grid(False, axis='x')
    plt.tight_layout(rect=[0, 0.1, 1, 0.95])

    global OUTPUT_DIR
    output_path = OUTPUT_DIR if OUTPUT_DIR else Path('.')
    png_file = output_path / 'stacked_proving_time_variants.png'
    pdf_file = output_path / 'stacked_proving_time_variants.pdf'
    plt.savefig(png_file, dpi=300, bbox_inches='tight')
    plt.savefig(pdf_file, dpi=300, bbox_inches='tight')
    print(f"Saved {png_file.name} and {pdf_file.name}")
    plt.close()

    # Plot 2: Total Proof Size Across All Proofs (Stacked)
    print(
        "\n[4/6] Generating: Stacked Bar Charts - Total Proof Size Across All Proofs...")

    size_data = {}
    for key, df in dfs.items():
        size_data[labels[key]] = (df['output_size'] / 1_000_000).tolist()

    # Pad size data
    padded_size = {}
    for name, data in size_data.items():
        padded_size[name] = data + [0] * (max_rows - len(data))
    df_size = pd.DataFrame(padded_size)

    fig2, ax2 = plt.subplots(figsize=(12, 8))
    bottom_size = np.zeros(len(bar_names))

    for i in range(max_rows):
        current_layer = df_size.iloc[i].values
        for j, category_name in enumerate(bar_names):
            if category_name == labels['composition']:
                segment_color = colors['composition'][i %
                                                      len(colors['composition'])]
            elif category_name == labels['proofaggregation']:
                segment_color = colors['proofaggregation'][i %
                                                           len(colors['proofaggregation'])]
            else:
                segment_color = colors['tree_aggregation'][i %
                                                           len(colors['tree_aggregation'])]

            if current_layer[j] > 0:
                ax2.bar(x_pos[j], current_layer[j], bottom=bottom_size[j],
                        color=segment_color, width=bar_width, linewidth=0.2,
                        edgecolor='black')
        bottom_size += current_layer

    ax2.set_title('Total Proof Size Across All Proofs (Stacked)',
                  fontsize=18, fontweight='bold', pad=20)
    ax2.set_xlabel('Proof Variant')
    ax2.set_ylabel('Size (MB)')
    ax2.set_ylim(get_dynamic_ylim(pd.Series(bottom_size)))
    ax2.set_xticks(x_pos)
    ax2.set_xticklabels(bar_names, rotation=15, ha='right')
    ax2.grid(axis='y', linestyle='-', linewidth=0.7, alpha=0.7)
    ax2.grid(False, axis='x')
    plt.tight_layout(rect=[0, 0.1, 1, 0.95])

    png_file = output_path / 'total_proof_size_all_stacked.png'
    pdf_file = output_path / 'total_proof_size_all_stacked.pdf'
    plt.savefig(png_file, dpi=300, bbox_inches='tight')
    plt.savefig(pdf_file, dpi=300, bbox_inches='tight')
    print(f"Saved {png_file.name} and {pdf_file.name}")
    plt.close()

    # Plot 3: Last Proof File Size Only
    print("\n[5/6] Generating: Last Proof File Size Comparison...")

    # Get only the final (last) proof size for each variant
    final_size_data = {}
    for key, df in dfs.items():
        final_size_data[labels[key]] = (df['output_size'].iloc[-1] / 1_000_000)

    fig3, ax3 = plt.subplots(figsize=(12, 8))

    # Use the darkest color from each palette for final sizes
    final_colors = {
        'composition': colors['composition'][-5],
        'proofaggregation': colors['proofaggregation'][-5],
        'tree_aggregation': colors['tree_aggregation'][-5]
    }

    bottom_size = []
    for j, category_name in enumerate(bar_names):
        size_val = final_size_data[category_name]
        if category_name == labels['composition']:
            bar_color = final_colors['composition']
        elif category_name == labels['proofaggregation']:
            bar_color = final_colors['proofaggregation']
        else:
            bar_color = final_colors['tree_aggregation']

        ax3.bar(x_pos[j], size_val, bottom=0,
                color=bar_color, width=bar_width, linewidth=0.2,
                edgecolor='black')
        bottom_size.append(size_val)

    ax3.set_title('Last Proof File Size Comparison',
                  fontsize=18, fontweight='bold', pad=20)
    ax3.set_xlabel('Proof Variant')
    ax3.set_ylabel('Size (MB)')
    ax3.set_ylim(get_dynamic_ylim(pd.Series(bottom_size)))
    ax3.set_xticks(x_pos)
    ax3.set_xticklabels(bar_names, rotation=15, ha='right')
    ax3.grid(axis='y', linestyle='-', linewidth=0.7, alpha=0.7)
    ax3.grid(False, axis='x')
    plt.tight_layout(rect=[0, 0.1, 1, 0.95])

    png_file = output_path / 'last_proof_file_size_comparison.png'
    pdf_file = output_path / 'last_proof_file_size_comparison.pdf'
    plt.savefig(png_file, dpi=300, bbox_inches='tight')
    plt.savefig(pdf_file, dpi=300, bbox_inches='tight')
    print(f"Saved {png_file.name} and {pdf_file.name}")
    plt.close()


def plot_cycle_counts_comparison(dfs: Dict[str, pd.DataFrame]):
    """Generate comparison plots for cycle counts across all variants."""
    print("\n[6/6] Generating: Cycle Counts Comparison...")

    fig, axes = plt.subplots(2, 2, figsize=(18, 14))
    fig.suptitle('Cycle Counts Comparison: Baseline Variants',
                 fontsize=22, fontweight='bold')

    colors = {
        'composition': 'royalblue',
        'proofaggregation': 'forestgreen',
        'tree_aggregation': 'darkorange'
    }

    labels = {
        'composition': 'Sequential (Depth)',
        'proofaggregation': 'Proof Aggregation (Width)',
        'tree_aggregation': 'Tree Aggregation (Depth + Width)'
    }

    markers = {
        'composition': 'o',
        'proofaggregation': 's',
        'tree_aggregation': '^'
    }

    max_run_id = max(df['run_id'].max() for df in dfs.values())
    xlim = (0, max_run_id + 1)

    # Flatten axes for easier iteration
    axes_flat = axes.flatten()

    # Plot 1: Total Cycles (in Millions)
    all_total = []
    for key, df in dfs.items():
        x = df['run_id']
        y = df['total_cycles_M']
        axes_flat[0].plot(x, y, label=labels[key], color=colors[key],
                          linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_total.extend(y.tolist())

    axes_flat[0].set_title('Total Cycles per Run ID', fontsize=16, pad=12)
    axes_flat[0].set_xlabel('Run ID', fontsize=14)
    axes_flat[0].set_ylabel('Total Cycles (Millions)', fontsize=14)
    axes_flat[0].set_ylim(get_dynamic_ylim(pd.Series(all_total)))
    axes_flat[0].legend(loc='best', frameon=True, shadow=True)
    setup_grid_and_ticks(axes_flat[0], xlim)

    # Plot 2: User Cycles (in Thousands)
    all_user = []
    for key, df in dfs.items():
        x = df['run_id']
        y = df['user_cycles_K']
        axes_flat[1].plot(x, y, label=labels[key], color=colors[key],
                          linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_user.extend(y.tolist())

    axes_flat[1].set_title('User Cycles per Run ID', fontsize=16, pad=12)
    axes_flat[1].set_xlabel('Run ID', fontsize=14)
    axes_flat[1].set_ylabel('User Cycles (Thousands)', fontsize=14)
    axes_flat[1].set_ylim(get_dynamic_ylim(pd.Series(all_user)))
    axes_flat[1].legend(loc='best', frameon=True, shadow=True)
    setup_grid_and_ticks(axes_flat[1], xlim)

    # Plot 3: Paging Cycles (in Thousands)
    all_paging = []
    for key, df in dfs.items():
        x = df['run_id']
        y = df['paging_cycles_K']
        axes_flat[2].plot(x, y, label=labels[key], color=colors[key],
                          linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_paging.extend(y.tolist())

    axes_flat[2].set_title('Paging Cycles per Run ID', fontsize=16, pad=12)
    axes_flat[2].set_xlabel('Run ID', fontsize=14)
    axes_flat[2].set_ylabel('Paging Cycles (Thousands)', fontsize=14)
    axes_flat[2].set_ylim(get_dynamic_ylim(pd.Series(all_paging)))
    axes_flat[2].legend(loc='best', frameon=True, shadow=True)
    setup_grid_and_ticks(axes_flat[2], xlim)

    # Plot 4: Reserved Cycles (in Thousands)
    all_reserved = []
    for key, df in dfs.items():
        x = df['run_id']
        y = df['reserved_cycles_K']
        axes_flat[3].plot(x, y, label=labels[key], color=colors[key],
                          linestyle='-', marker=markers[key], markersize=6, linewidth=2)
        all_reserved.extend(y.tolist())

    axes_flat[3].set_title('Reserved Cycles per Run ID', fontsize=16, pad=12)
    axes_flat[3].set_xlabel('Run ID', fontsize=14)
    axes_flat[3].set_ylabel('Reserved Cycles (Thousands)', fontsize=14)
    axes_flat[3].set_ylim(get_dynamic_ylim(pd.Series(all_reserved)))
    axes_flat[3].legend(loc='best', frameon=True, shadow=True)
    setup_grid_and_ticks(axes_flat[3], xlim)

    save_figure('cycle_counts_variants')
    plt.close()


def main():
    """Generate all proof variant comparison plots."""
    print("=" * 70)
    print("Proof Variants Plotting Script")
    print("=" * 70)

    # Parse command line arguments
    if len(sys.argv) > 1:
        benchmark_path = sys.argv[1]
        print(f"\nUsing provided benchmark path: {benchmark_path}")
    else:
        print("\nUsage: python plot_proof_variants.py <benchmark_path>")
        print("\nExample:")
        print("  python plot_proof_variants.py ../documents/benchmark_20260118_032016")
        sys.exit(1)

    # Find CSV files
    print("\nSearching for CSV files...")
    if not find_csv_files(benchmark_path):
        sys.exit(1)

    # Load all data
    print("\nLoading CSV data...")
    dfs = {}
    for key, path in CSV_FILES.items():
        df = load_csv_data(path)
        if df is not None:
            dfs[key] = prepare_dataframe(df)

    if not dfs:
        print("\nError: No data could be loaded. Exiting.")
        sys.exit(1)

    print(f"\nSuccessfully loaded {len(dfs)} datasets")

    # Generate all plots
    print("\n" + "=" * 70)
    print("Generating Plots")
    print("=" * 70)

    plot_proof_time_comparison(dfs)
    plot_file_size_comparison(dfs)
    plot_stacked_bars(dfs)
    plot_cycle_counts_comparison(dfs)

    print("\n" + "-" * 70)
    print("All plots generated successfully!")
    print(f"Output location: {OUTPUT_DIR}")
    print("-" * 70)


if __name__ == '__main__':
    main()
