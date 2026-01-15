import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
import seaborn as sns
import sys
from typing import Dict, Tuple, Optional
from pathlib import Path

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

# Number of sequential proofs (row n is the aggregated proof over proofs 0 to n-1)
N_PROOFS = 20

# Will be set by find_csv_files()
CSV_FILES = {}
OUTPUT_DIR = None

# Helper Functions
def find_csv_files(benchmark_path: str) -> bool:
    """
    Find CSV files in the benchmark run directory.

    Args:
        benchmark_path: Path to benchmark run (e.g., '../documents/benchmark_1234567890')

    Returns:
        True if files were found, False otherwise
    """
    global CSV_FILES, OUTPUT_DIR

    # Convert to Path object and resolve
    run_dir = Path(benchmark_path).resolve()

    if not run_dir.exists():
        print(f"Error: Benchmark directory '{run_dir}' does not exist")
        return False

    if not run_dir.is_dir():
        print(f"Error: '{run_dir}' is not a directory")
        return False

    # Look for CSV files directly in the benchmark directory
    found_files = {}
    csv_names = {
        'composition': 'bench_composition.csv',
        'aggregation': 'bench_aggregation.csv',
        'proofaggregation': 'bench_proofaggregation.csv',
    }

    for key, filename in csv_names.items():
        csv_path = run_dir / filename
        if csv_path.exists():
            found_files[key] = csv_path
            print(f"Found {key}: {filename}")
        else:
            print(f"Warning: {filename} not found")

    if not found_files:
        print(f"Error: No CSV files found in '{run_dir}'")
        print("Expected files: bench_composition.csv, bench_aggregation.csv, bench_proofaggregation.csv")
        return False

    CSV_FILES.update(found_files)

    # Set output directory to plots subfolder in the benchmark run directory
    OUTPUT_DIR = run_dir / "plots"
    OUTPUT_DIR.mkdir(exist_ok=True)
    print(f"\nOutput directory: {OUTPUT_DIR}")

    return True


def get_latest_benchmark_folder(base_path: Path = None) -> Optional[Path]:
    """Find the most recent benchmark folder."""
    if base_path is None:
        base_path = Path(__file__).parent.parent / 'documents'

    if not base_path.exists():
        return None

    folders = [f for f in base_path.iterdir()
               if f.is_dir() and f.name.startswith('benchmark_')]

    if not folders:
        return None

    # Sort by name (timestamps sort chronologically)
    folders.sort()
    return folders[-1]


def load_csv_data(file_path: Path) -> Optional[pd.DataFrame]:
    """Load CSV data with error handling."""
    try:
        return pd.read_csv(file_path)
    except Exception as e:
        print(f"Error: Could not load data from '{file_path}': {e}")
        return None


def prepare_dataframe(df: pd.DataFrame) -> pd.DataFrame:
    """Add computed columns for easier plotting."""
    df = df.copy()
    df['input_size_M'] = df['input_size'] / 1_000_000
    df['output_size_M'] = df['output_size'] / 1_000_000
    df['total_cycles_M'] = df['total_cycles'] / 1_000_000
    df['paging_cycles_K'] = df['paging_cycles'] / 1_000
    df['user_cycles_K'] = df['user_cycles'] / 1_000
    df['reserved_cycles_K'] = df['reserved_cycles'] / 1_000
    df['total_cycles_K'] = df['total_cycles'] / 1_000
    return df


def get_dynamic_ylim(data: pd.Series, margin: float = 0.1) -> Tuple[float, float]:
    """Calculate dynamic y-axis limits with margin, handling NaN/Inf."""
    clean_data = data.replace([np.inf, -np.inf], np.nan).dropna()

    # Handle empty or all-zero data
    if len(clean_data) == 0 or clean_data.max() == 0:
        return (0, 1)  # Default range

    max_val = clean_data.max()
    return (0, max_val * (1 + margin))


def get_dynamic_xlim(df: pd.DataFrame, margin: float = 0.5) -> Tuple[float, float]:
    """Calculate dynamic x-axis limits."""
    min_id = df['run_id'].min()
    max_id = df['run_id'].max()
    return (min_id - margin, max_id + margin)


def setup_grid_and_ticks(ax, df: pd.DataFrame, xlim: Optional[Tuple[float, float]] = None):
    """Configure grid and tick marks for an axis."""
    if xlim is None:
        xlim = get_dynamic_xlim(df)
    ax.set_xlim(xlim)
    ax.xaxis.set_major_locator(ticker.MultipleLocator(1))
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



# Plot 1: Proof Composition vs Sequential Proofs
def plot_composition_vs_sequential():
    """Generate comparison plot between composition and sequential proofs."""
    print("\n[1/4] Generating: Proof Composition vs Sequential Proofs...")

    df_comp = load_csv_data(CSV_FILES['composition'])
    df_seq = load_csv_data(CSV_FILES['proofaggregation'])

    if df_comp is None or df_seq is None:
        print("  ✗ Failed to load required data files")
        return

    df_comp = prepare_dataframe(df_comp)
    df_seq = prepare_dataframe(df_seq)

    # Sequential proofs are rows 0 to N_PROOFS-1, row N_PROOFS is the aggregated proof
    df_seq = df_seq.iloc[0:N_PROOFS].reset_index(drop=True)

    # Create figure with 3 subplots
    fig, axes = plt.subplots(1, 3, figsize=(20, 7))
    fig.suptitle('Benchmark Performance Analysis for Nested Proofs vs. Sequential Proofs',
                 fontsize=22, fontweight='bold')

    x = df_comp['run_id']
    xlim = get_dynamic_xlim(df_comp)

    # Subplot 1: Proof Time
    axes[0].plot(x, df_comp['proof_time'], label='Proof Time - Composition',
                 color='royalblue', linestyle='-', marker='o', markersize=7, linewidth=1.5)
    axes[0].plot(x, df_seq['proof_time'], label='Proof Time',
                 color='green', linestyle='-', marker='D', markersize=7, linewidth=1.5)
    axes[0].set_title('Proof Time per Run', fontsize=16)
    axes[0].set_xlabel('Run ID')
    axes[0].set_ylabel('Time (Seconds)')
    axes[0].set_ylim(get_dynamic_ylim(pd.concat([df_comp['proof_time'], df_seq['proof_time']])))
    axes[0].legend(loc='upper left', frameon=True, shadow=True)
    setup_grid_and_ticks(axes[0], df_comp, xlim)

    # Subplot 2: Size Metrics
    axes[1].plot(x, df_comp['input_size_M'], label='Input (Host) Size - Composition',
                 color='cornflowerblue', linestyle='-', marker='v', markersize=7, linewidth=1.5)
    axes[1].plot(x, df_comp['output_size_M'], label='Proof Size - Composition',
                 color='royalblue', linestyle='-', marker='o', markersize=7, linewidth=1.5)
    axes[1].plot(x, df_seq['input_size_M'], label='Input (Host) Size',
                 color='mediumseagreen', linestyle='-', marker='s', markersize=7, linewidth=1.5)
    axes[1].plot(x, df_seq['output_size_M'], label='Proof Size',
                 color='green', linestyle='-', marker='D', markersize=7, linewidth=1.5)
    axes[1].set_title('Size Metrics per Run', fontsize=16)
    axes[1].set_xlabel('Run ID')
    axes[1].set_ylabel('Size (Million Bytes)')
    all_sizes = pd.concat([df_comp['input_size_M'], df_comp['output_size_M'],
                           df_seq['input_size_M'], df_seq['output_size_M']])
    axes[1].set_ylim(get_dynamic_ylim(all_sizes))
    axes[1].legend(loc='upper left', frameon=True, shadow=True)
    setup_grid_and_ticks(axes[1], df_comp, xlim)

    # Subplot 3: Cycle Metrics
    axes[2].plot(x, df_comp['total_cycles_M'], label='Total Cycles - Composition',
                 color='royalblue', linestyle='-', marker='o', markersize=7, linewidth=2.0)
    axes[2].plot(x, df_seq['total_cycles_M'], label='Total Cycles',
                 color='green', linestyle='-', marker='D', markersize=7, linewidth=2.0)
    axes[2].set_title('Cycle Metrics per Run', fontsize=16)
    axes[2].set_xlabel('Run ID')
    axes[2].set_ylabel('Cycles (Millions)')
    axes[2].set_ylim(get_dynamic_ylim(pd.concat([df_comp['total_cycles_M'], df_seq['total_cycles_M']])))
    axes[2].legend(loc='upper left', frameon=True, shadow=True, ncol=2)
    setup_grid_and_ticks(axes[2], df_comp, xlim)

    save_figure('benchmark_composition')
    plt.close()



# Plot 2: Cycle Analysis
def plot_cycle_analysis():
    """Generate detailed cycle analysis for sequential and nested proofs."""
    print("\n[2/4] Generating: Cycle Analysis...")

    df_agg = load_csv_data(CSV_FILES['proofaggregation'])
    df_comp = load_csv_data(CSV_FILES['composition'])

    if df_agg is None or df_comp is None:
        print("  ✗ Failed to load required data files")
        return

    # Sequential proofs are rows 0 to N_PROOFS-1, row N_PROOFS is the aggregated proof
    df_agg = df_agg.iloc[0:N_PROOFS].reset_index(drop=True)

    df_agg = prepare_dataframe(df_agg)
    df_comp = prepare_dataframe(df_comp)

    # Create figure with 2 subplots
    fig, axes = plt.subplots(1, 2, figsize=(14, 7))
    fig.suptitle('Cycle Analysis for Sequential Proofs and Nested Proofs',
                 fontsize=22, fontweight='bold')

    # Use first N_PROOFS rows
    x = df_agg['run_id'][:N_PROOFS]
    xlim = (0.5, N_PROOFS + 0.5)
    palette_cycles = sns.color_palette('Dark2', n_colors=4)
    palette_size = sns.color_palette('Set1', n_colors=6)

    # Subplot 1: Sequential Proofs
    axes[0].plot(x, df_agg['paging_cycles_K'][:N_PROOFS], label='Paging Cycles',
                 color=palette_cycles[0], linestyle='-', marker='^', markersize=7, linewidth=1.5)
    axes[0].plot(x, df_agg['user_cycles_K'][:N_PROOFS], label='User Cycles',
                 color=palette_cycles[1], linestyle='-', marker='v', markersize=7, linewidth=1.5)
    axes[0].plot(x, df_agg['reserved_cycles_K'][:N_PROOFS], label='Reserved Cycles',
                 color='DarkGrey', linestyle='-', marker='P', markersize=4, linewidth=1)
    axes[0].plot(x, df_agg['total_cycles_K'][:N_PROOFS], label='Total Cycles',
                 color=palette_size[3], linestyle='-', marker='X', markersize=7, linewidth=2.0)
    axes[0].set_title('Cycle Metrics Sequential Proofs', fontsize=16)
    axes[0].set_xlabel('Run ID')
    axes[0].set_ylabel('Cycles (Thousands)')
    all_cycles_seq = pd.concat([df_agg['paging_cycles_K'][:N_PROOFS], df_agg['user_cycles_K'][:N_PROOFS],
                                 df_agg['reserved_cycles_K'][:N_PROOFS], df_agg['total_cycles_K'][:N_PROOFS]])
    axes[0].set_ylim(get_dynamic_ylim(all_cycles_seq, margin=0.2))
    axes[0].legend(loc='upper left', frameon=True, shadow=True, ncol=2)
    axes[0].set_xlim(xlim)
    axes[0].xaxis.set_major_locator(ticker.MultipleLocator(1))
    axes[0].xaxis.set_minor_locator(ticker.MultipleLocator(1))
    axes[0].grid(True, which='major', linestyle='-', linewidth=0.7, alpha=0.7)
    axes[0].grid(True, which='minor', linestyle=':', linewidth=0.5, alpha=0.5)
    axes[0].ticklabel_format(style='plain', axis='y')

    # Subplot 2: Nested Proofs
    x_comp = df_comp['run_id'][:N_PROOFS]
    axes[1].plot(x_comp, df_comp['paging_cycles_K'][:N_PROOFS], label='Paging Cycles',
                 color=palette_cycles[0], linestyle='-', marker='^', markersize=7, linewidth=1.5)
    axes[1].plot(x_comp, df_comp['user_cycles_K'][:N_PROOFS], label='User Cycles',
                 color=palette_cycles[1], linestyle='-', marker='v', markersize=7, linewidth=1.5)
    axes[1].plot(x_comp, df_comp['reserved_cycles_K'][:N_PROOFS], label='Reserved Cycles',
                 color='DarkGrey', linestyle='-', marker='P', markersize=4, linewidth=1)
    axes[1].plot(x_comp, df_comp['total_cycles_K'][:N_PROOFS], label='Total Cycles',
                 color=palette_size[3], linestyle='-', marker='X', markersize=7, linewidth=2.0)
    axes[1].set_title('Cycle Metrics Nested Proofs', fontsize=16)
    axes[1].set_xlabel('Run ID')
    axes[1].set_ylabel('Cycles (Thousands)')
    all_cycles_nested = pd.concat([df_comp['paging_cycles_K'][:N_PROOFS], df_comp['user_cycles_K'][:N_PROOFS],
                                    df_comp['reserved_cycles_K'][:N_PROOFS], df_comp['total_cycles_K'][:N_PROOFS]])
    axes[1].set_ylim(get_dynamic_ylim(all_cycles_nested, margin=0.2))
    axes[1].legend(loc='upper left', frameon=True, shadow=True, ncol=2)
    axes[1].set_xlim(xlim)
    axes[1].xaxis.set_major_locator(ticker.MultipleLocator(1))
    axes[1].xaxis.set_minor_locator(ticker.MultipleLocator(1))
    axes[1].grid(True, which='major', linestyle='-', linewidth=0.7, alpha=0.7)
    axes[1].grid(True, which='minor', linestyle=':', linewidth=0.5, alpha=0.5)
    axes[1].ticklabel_format(style='plain', axis='y')

    save_figure('benchmark_cycles')
    plt.close()



# Plot 3 & 4: Stacked Bar Charts
def load_stacked_data() -> Tuple[Dict, Dict, int]:
    """Load and prepare data for stacked bar charts."""
    time_data = {}
    size_data = {}
    max_rows = 0

    # Load all data files
    df_comp = load_csv_data(CSV_FILES['composition'])
    df_proofagg = load_csv_data(CSV_FILES['proofaggregation'])
    df_agg = load_csv_data(CSV_FILES['aggregation'])

    if df_proofagg is None:
        return {}, {}, 0

    # Nested Proof (from bench_composition.csv)
    if df_comp is not None:
        time_data['Nested Proof'] = df_comp['proof_time'].tolist()
        # Only N_PROOFS-1 data point for size
        size_data['Nested Proof'] = [(df_comp['output_size'].iloc[N_PROOFS-1] / 1_000_000)] if len(df_comp) >= N_PROOFS else []
    else:
        time_data['Nested Proof'] = []
        size_data['Nested Proof'] = []

    # Sequential Proof (from bench_proofaggregation.csv, rows 0 to N_PROOFS-1)
    time_data['Sequential Proof'] = df_proofagg['proof_time'].iloc[0:N_PROOFS].tolist()
    size_data['Sequential Proof'] = (df_proofagg['output_size'].iloc[0:N_PROOFS] / 1_000_000).tolist()

    # Aggregated Proof (from bench_proofaggregation.csv, rows 0 to N_PROOFS, includes aggregated at index N_PROOFS)
    time_data['Aggregated Proof'] = df_proofagg['proof_time'].iloc[0:N_PROOFS+1].tolist()
    # Only N_PROOFS data point for size (the aggregated proof)
    size_data['Aggregated Proof'] = (df_proofagg['output_size'].iloc[N_PROOFS:N_PROOFS+1] / 1_000_000).tolist()

    # Single Proof (from bench_aggregation.csv)
    if df_agg is not None:
        time_data['Single Proof'] = df_agg['proof_time'].tolist()
        size_data['Single Proof'] = (df_agg['output_size'] / 1_000_000).tolist()
    else:
        time_data['Single Proof'] = []
        size_data['Single Proof'] = []

    # Calculate max rows
    for data in time_data.values():
        max_rows = max(max_rows, len(data))

    return time_data, size_data, max_rows


def create_stacked_dataframes(time_data: Dict, size_data: Dict, max_rows: int) -> Tuple[pd.DataFrame, pd.DataFrame]:
    """Pad data and create DataFrames for stacked plotting."""
    # Pad time data
    padded_time = {}
    for name, data in time_data.items():
        padded_time[name] = data + [np.nan] * (max_rows - len(data))
    df_time = pd.DataFrame(padded_time).fillna(0)

    # Pad size data
    padded_size = {}
    for name, data in size_data.items():
        if name == 'Nested Proof' and len(data) == 1:
            padded_size[name] = data + [np.nan] * (max_rows - 1)
        else:
            padded_size[name] = data + [np.nan] * (max_rows - len(data))
    df_size = pd.DataFrame(padded_size).fillna(0)

    return df_time, df_size


def plot_stacked_bars(df_time: pd.DataFrame, df_size: pd.DataFrame, max_rows: int,
                     color_scheme: str = 'default', plot_number: int = 3):
    """Create stacked bar charts for time and size."""
    scheme_name = 'Green Palette' if color_scheme == 'green' else 'Default'
    print(f"\n[{plot_number}/4] Generating: Stacked Bar Charts ({scheme_name})...")

    bar_names = list(df_time.columns)
    x_pos = np.arange(len(bar_names))
    bar_width = 0.6

    # Define color palettes
    if color_scheme == 'green':
        category_palettes = {
            'Nested Proof': sns.color_palette('Blues', n_colors=max_rows),
            'Sequential Proof': sns.color_palette('Greens', n_colors=max_rows),
            'Aggregated Proof': sns.color_palette('Greens', n_colors=max_rows),
            'Single Proof': sns.color_palette('BrBG', n_colors=max_rows),
        }
    else:
        category_palettes = {
            'Nested Proof': sns.color_palette('Blues', n_colors=max_rows),
            'Sequential Proof': sns.color_palette('Reds', n_colors=max_rows),
            'Aggregated Proof': sns.color_palette('Greens', n_colors=max_rows),
            'Single Proof': sns.color_palette('BrBG', n_colors=max_rows),
        }

    color_21st = None
    color_20th = None

    # Plot 1: Proving Time Total
    fig1, ax1 = plt.subplots(figsize=(12, 7))
    bottom_time = np.zeros(len(bar_names))

    for i in range(max_rows):
        current_layer = df_time.iloc[i].values
        for j, category_name in enumerate(bar_names):
            segment_color = category_palettes[category_name][i % len(category_palettes[category_name])]

            if category_name == 'Aggregated Proof' and i == 20:
                color_21st = '#004545' if color_scheme == 'green' else segment_color
                segment_color = color_21st
            if category_name == 'Nested Proof' and i == 19:
                color_20th = segment_color

            ax1.bar(x_pos[j], current_layer[j], bottom=bottom_time[j],
                   color=segment_color, width=bar_width, linewidth=0.2,
                   edgecolor='black', label=f'Run ID {i+1}' if j == 0 else '')
        bottom_time += current_layer

    ax1.set_title('Proving Time Total', fontsize=18, fontweight='bold', pad=20)
    ax1.set_xlabel('Benchmark Approach')
    ax1.set_ylabel('Time (Seconds)')
    ax1.set_ylim(get_dynamic_ylim(pd.Series(bottom_time)))
    ax1.set_xticks(x_pos)
    ax1.set_xticklabels(bar_names, rotation=0, ha='center')
    ax1.grid(axis='y', linestyle='-', linewidth=0.7, alpha=0.7)
    ax1.grid(False, axis='x')
    plt.tight_layout(rect=[0, 0.1, 1, 0.95])

    global OUTPUT_DIR
    output_path = OUTPUT_DIR if OUTPUT_DIR else Path('.')
    suffix = '_green' if color_scheme == 'green' else ''

    png_file = output_path / f'proving_time_total{suffix}.png'
    pdf_file = output_path / f'proving_time_total{suffix}.pdf'
    plt.savefig(png_file, dpi=300, bbox_inches='tight')
    plt.savefig(pdf_file, dpi=600, bbox_inches='tight')
    print(f"Saved {png_file.name} and {pdf_file.name}")
    plt.close()

    # Plot 2: Proof Size Total
    fig2, ax2 = plt.subplots(figsize=(12, 7))
    bottom_size = np.zeros(len(bar_names))

    # Plot Nested Proof (single bar)
    comp_idx = bar_names.index('Nested Proof')
    if color_20th and df_size['Nested Proof'].iloc[0] > 0:
        ax2.bar(x_pos[comp_idx], df_size['Nested Proof'].iloc[0], bottom=0,
               color=color_20th, width=bar_width, linewidth=0.2, edgecolor='black')
        bottom_size[comp_idx] = df_size['Nested Proof'].iloc[0]

    # Plot Sequential Proof (stacked)
    loose_idx = bar_names.index('Sequential Proof')
    for i in range(max_rows):
        if df_size['Sequential Proof'].iloc[i] > 0:
            segment_color = category_palettes['Sequential Proof'][i % len(category_palettes['Sequential Proof'])]
            ax2.bar(x_pos[loose_idx], df_size['Sequential Proof'].iloc[i],
                   bottom=bottom_size[loose_idx], color=segment_color,
                   width=bar_width, linewidth=0.2, edgecolor='black')
            bottom_size[loose_idx] += df_size['Sequential Proof'].iloc[i]

    # Plot Aggregated Proof (single bar)
    proof_agg_idx = bar_names.index('Aggregated Proof')
    if color_21st and df_size['Aggregated Proof'].iloc[0] > 0:
        ax2.bar(x_pos[proof_agg_idx], df_size['Aggregated Proof'].iloc[0], bottom=0,
               color=color_21st, width=bar_width, linewidth=0.2, edgecolor='black')
        bottom_size[proof_agg_idx] = df_size['Aggregated Proof'].iloc[0]

    # Plot Single Proof (stacked)
    if 'Single Proof' in bar_names:
        agg_idx = bar_names.index('Single Proof')
        for i in range(max_rows):
            if df_size['Single Proof'].iloc[i] > 0:
                segment_color = category_palettes['Single Proof'][i % len(category_palettes['Single Proof'])]
                ax2.bar(x_pos[agg_idx], df_size['Single Proof'].iloc[i],
                       bottom=bottom_size[agg_idx], color=segment_color,
                       width=bar_width, linewidth=0.2, edgecolor='black')
                bottom_size[agg_idx] += df_size['Single Proof'].iloc[i]

    ax2.set_title('Proof Size Total', fontsize=18, fontweight='bold', pad=20)
    ax2.set_xlabel('Benchmark Approach')
    ax2.set_ylabel('Size (Million Bytes)')
    ax2.set_ylim(get_dynamic_ylim(pd.Series(bottom_size)))
    ax2.set_xticks(x_pos)
    ax2.set_xticklabels(bar_names, rotation=0, ha='center')
    ax2.grid(axis='y', linestyle='-', linewidth=0.7, alpha=0.7)
    ax2.grid(False, axis='x')
    plt.tight_layout(rect=[0, 0.1, 1, 0.95])

    png_file = output_path / f'proof_size_total{suffix}.png'
    pdf_file = output_path / f'proof_size_total{suffix}.pdf'
    plt.savefig(png_file, dpi=300, bbox_inches='tight')
    plt.savefig(pdf_file, dpi=600, bbox_inches='tight')
    print(f"Saved {png_file.name} and {pdf_file.name}")
    plt.close()

def main():
    """Generate all benchmark plots."""
    print("Starting Benchmark Plot Generation...")


    # Parse command line arguments
    if len(sys.argv) > 1:
        benchmark_path = sys.argv[1]
        print(f"\nUsing provided benchmark path: {benchmark_path}")
    else:
        # Try to find the latest benchmark folder
        print("\nNo path provided, looking for latest benchmark folder...")
        latest_folder = get_latest_benchmark_folder()
        if latest_folder:
            benchmark_path = str(latest_folder)
            print(f"Found latest benchmark: {benchmark_path}")
        else:
            print("\nUsage: python generate_plots.py <benchmark_path>")
            print("\nExample:")
            print("  python generate_plots.py ../documents/benchmark_1737000000")
            print("\nOr run without arguments to use the latest benchmark folder.")
            sys.exit(1)

    # Find CSV files
    print("\nSearching for CSV files...")
    if not find_csv_files(benchmark_path):
        sys.exit(1)

    # Check if data files exist
    missing_files = []
    for name, path in CSV_FILES.items():
        if not path.exists():
            missing_files.append(f"  - {path}")

    if missing_files:
        print("\n⚠ Warning: Some data files are missing:")
        for f in missing_files:
            print(f)
        print("\nContinuing with available files...\n")

    # Generate all plots
    print("\n" + "=" * 70)
    print("Generating Plots")
    print("=" * 70)

    plot_composition_vs_sequential()
    plot_cycle_analysis()

    # Generate stacked bar charts
    time_data, size_data, max_rows = load_stacked_data()
    if max_rows > 0:
        df_time, df_size = create_stacked_dataframes(time_data, size_data, max_rows)
        plot_stacked_bars(df_time, df_size, max_rows, color_scheme='default', plot_number=3)
        plot_stacked_bars(df_time, df_size, max_rows, color_scheme='green', plot_number=4)
    else:
        print("\n  ✗ Failed to load data for stacked bar charts")

    print("\n" + "-" * 70)
    print("All plots generated successfully!")
    print(f"Output location: {OUTPUT_DIR}")


if __name__ == '__main__':
    main()
