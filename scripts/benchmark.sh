#!/bin/bash

# Benchmark script for ls-parser + cvc5 pipeline
# Usage: ./scripts/benchmark.sh <input_path> [iterations]

set -e

INPUT_PATH="$1"
ITERATIONS="${2:-10}"

if [ -z "$INPUT_PATH" ]; then
    echo "Usage: $0 <input_path> [iterations]"
    echo "  input_path: file or directory containing .sl files"
    echo "  iterations: number of benchmark runs (default: 10)"
    exit 1
fi

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "hyperfine not found. Installing..."
    if command -v cargo &> /dev/null; then
        cargo install hyperfine
    elif command -v brew &> /dev/null; then
        brew install hyperfine
    else
        echo "Please install hyperfine manually: https://github.com/sharkdp/hyperfine"
        exit 1
    fi
fi

# Check if cvc5 is available
if ! command -v cvc5 &> /dev/null; then
    echo "cvc5 not found. Please install cvc5 first."
    exit 1
fi

# Function to benchmark a single file
benchmark_file() {
    local file="$1"
    local output_dir="$2"
    
    echo "Benchmarking: $file"
    
    # Create output directory for this file
    local file_output_dir="$output_dir/$(basename "$file" .sl)"
    mkdir -p "$file_output_dir"
    
    # Run benchmark
    hyperfine \
        --warmup 2 \
        --min-runs 3 \
        --max-runs "$ITERATIONS" \
        --export-json "$file_output_dir/results.json" \
        --export-markdown "$file_output_dir/results.md" \
        --command-name "$(basename "$file")" \
        "./target/release/ls-parser --file '$file' --to-bv --dump '$file_output_dir' | cvc5 --lang=sygus2"
    
    echo "Results saved to: $file_output_dir/"
}

# Function to benchmark all files in a directory
benchmark_directory() {
    local dir="$1"
    local output_dir="$2"
    
    echo "Benchmarking all .sl files in: $dir"
    
    # Find all .sl files
    local files=($(find "$dir" -name "*.sl" -type f))
    
    if [ ${#files[@]} -eq 0 ]; then
        echo "No .sl files found in $dir"
        exit 1
    fi
    
    echo "Found ${#files[@]} .sl files"
    
    for file in "${files[@]}"; do
        benchmark_file "$file" "$output_dir"
    done
    
    # Generate summary report
    generate_summary "$output_dir"
}

# Function to generate summary report
generate_summary() {
    local output_dir="$1"
    local summary_file="$output_dir/summary.md"
    
    echo "# Benchmark Summary" > "$summary_file"
    echo "Generated on: $(date)" >> "$summary_file"
    echo "" >> "$summary_file"
    
    # Find all result files
    local result_files=($(find "$output_dir" -name "results.md" -type f))
    
    if [ ${#result_files[@]} -eq 0 ]; then
        echo "No benchmark results found" >> "$summary_file"
        return
    fi
    
    echo "## Results by File" >> "$summary_file"
    echo "" >> "$summary_file"
    
    for result_file in "${result_files[@]}"; do
        local file_name=$(basename "$(dirname "$result_file")")
        echo "### $file_name" >> "$summary_file"
        echo "" >> "$summary_file"
        cat "$result_file" >> "$summary_file"
        echo "" >> "$summary_file"
    done
    
    echo "Summary report generated: $summary_file"
}

# Main execution
main() {
    local output_dir="benchmark/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$output_dir"
    
    echo "Starting benchmark..."
    echo "Input: $INPUT_PATH"
    echo "Iterations: $ITERATIONS"
    echo "Output directory: $output_dir"
    echo ""
    
    if [ -f "$INPUT_PATH" ]; then
        # Single file
        benchmark_file "$INPUT_PATH" "$output_dir"
    elif [ -d "$INPUT_PATH" ]; then
        # Directory
        benchmark_directory "$INPUT_PATH" "$output_dir"
    else
        echo "Error: $INPUT_PATH is not a file or directory"
        exit 1
    fi
    
    echo ""
    echo "Benchmark completed! Results saved to: $output_dir/"
}

main "$@" 