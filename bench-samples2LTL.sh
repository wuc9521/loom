#!/usr/bin/env bash
set -euo pipefail

HEADER=$'group\tpath\tstatus\ttotal_ms\tmethod\taccepting_traces\trejecting_traces\tformula_depth\tnum_variables\tmax_trace_length\tspecified_formula\tobtained_formula\tcomment\tdt_num_primitive_candidates\tdt_num_primitives_used\tdt_subset_size\ttree_path\traw_stats_csv'

usage() {
  cat <<'EOF'
Usage:
  bench-samples2LTL.sh [options]

Options:
  --repo PATH                 samples2LTL checkout. Default: /root/samples2LTL if present,
                              otherwise /Users/wuchentian/Codes/Python/samples2LTL
  --traces-folder PATH        Trace folder to benchmark. Default:
                              traces/generated/5to10Traces
  --name NAME                 Benchmark group name. Default: ltl-syslite-samples2ltl
  --results PATH              Explicit TSV output path.
  --results-dir PATH          Results root. Default: bench-results
  --python CMD                Python executable. Default: python3
  --timeout-secs N            Per-case timeout passed to samples2LTL. Default: 600
  --method MODE               sat | dt | both. Default: sat
  --test-name NAME            samples2LTL experiment name. Default: derived from --name
  --encoder NAME              dag | tree. Default: dag
  --iteration-start N         SAT iteration start. Default: 1
  --iteration-step N          SAT iteration step. Default: 1
  --dt-subset-size N          DT subset size. Default: 10
  --dt-decrease-rate X        DT decrease rate. Default: 0.5
  --dt-restarts N             DT restarts of sampling. Default: 3
  --dt-repetitions N          DT repetitions inside sampling. Default: 3
  --dt-strategy N             DT strategy. Default: 2
  --help                      Show this help.
EOF
}

die() {
  echo "bench-samples2LTL: $*" >&2
  exit 1
}

slugify() {
  python3 - "$1" <<'PY'
import re, sys
slug = re.sub(r'[^0-9A-Za-z]+', '-', sys.argv[1]).strip('-').lower()
print(slug or "results")
PY
}

abspath() {
  python3 - "$1" <<'PY'
import os, sys
print(os.path.abspath(sys.argv[1]))
PY
}

sanitize_field() {
  python3 - "$1" <<'PY'
import sys
print(sys.argv[1].replace('\t', ' ').replace('\n', ' '))
PY
}

REPO=""
TRACES_FOLDER="traces/generated/5to10Traces"
NAME="ltl-syslite-samples2ltl"
RESULTS_PATH=""
RESULTS_DIR=""
PYTHON_BIN="python3"
TIMEOUT_SECS=600
METHOD="sat"
TEST_NAME=""
ENCODER="dag"
ITERATION_START=1
ITERATION_STEP=1
DT_SUBSET_SIZE=10
DT_DECREASE_RATE=0.5
DT_RESTARTS=3
DT_REPETITIONS=3
DT_STRATEGY=2

while (($#)); do
  case "$1" in
    --repo)
      (($# >= 2)) || die "--repo requires a value"
      REPO=$2
      shift 2
      ;;
    --traces-folder)
      (($# >= 2)) || die "--traces-folder requires a value"
      TRACES_FOLDER=$2
      shift 2
      ;;
    --name)
      (($# >= 2)) || die "--name requires a value"
      NAME=$2
      shift 2
      ;;
    --results)
      (($# >= 2)) || die "--results requires a value"
      RESULTS_PATH=$2
      shift 2
      ;;
    --results-dir)
      (($# >= 2)) || die "--results-dir requires a value"
      RESULTS_DIR=$2
      shift 2
      ;;
    --python)
      (($# >= 2)) || die "--python requires a value"
      PYTHON_BIN=$2
      shift 2
      ;;
    --timeout-secs)
      (($# >= 2)) || die "--timeout-secs requires a value"
      TIMEOUT_SECS=$2
      shift 2
      ;;
    --method)
      (($# >= 2)) || die "--method requires a value"
      METHOD=$2
      shift 2
      ;;
    --test-name)
      (($# >= 2)) || die "--test-name requires a value"
      TEST_NAME=$2
      shift 2
      ;;
    --encoder)
      (($# >= 2)) || die "--encoder requires a value"
      ENCODER=$2
      shift 2
      ;;
    --iteration-start)
      (($# >= 2)) || die "--iteration-start requires a value"
      ITERATION_START=$2
      shift 2
      ;;
    --iteration-step)
      (($# >= 2)) || die "--iteration-step requires a value"
      ITERATION_STEP=$2
      shift 2
      ;;
    --dt-subset-size)
      (($# >= 2)) || die "--dt-subset-size requires a value"
      DT_SUBSET_SIZE=$2
      shift 2
      ;;
    --dt-decrease-rate)
      (($# >= 2)) || die "--dt-decrease-rate requires a value"
      DT_DECREASE_RATE=$2
      shift 2
      ;;
    --dt-restarts)
      (($# >= 2)) || die "--dt-restarts requires a value"
      DT_RESTARTS=$2
      shift 2
      ;;
    --dt-repetitions)
      (($# >= 2)) || die "--dt-repetitions requires a value"
      DT_REPETITIONS=$2
      shift 2
      ;;
    --dt-strategy)
      (($# >= 2)) || die "--dt-strategy requires a value"
      DT_STRATEGY=$2
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$REPO" ]]; then
  if [[ -d /root/samples2LTL ]]; then
    REPO=/root/samples2LTL
  else
    REPO=/Users/wuchentian/Codes/Python/samples2LTL
  fi
fi
REPO=$(abspath "$REPO")
[[ -d "$REPO" ]] || die "repo not found: $REPO"

case "$METHOD" in
  sat|dt|both) ;;
  *) die "--method must be sat, dt, or both" ;;
esac
case "$ENCODER" in
  dag|tree) ;;
  *) die "--encoder must be dag or tree" ;;
esac

command -v "$PYTHON_BIN" >/dev/null 2>&1 || die "python executable not found: $PYTHON_BIN"

if [[ -z "$RESULTS_DIR" ]]; then
  RESULTS_DIR="$(pwd)/bench-results"
else
  RESULTS_DIR=$(abspath "$RESULTS_DIR")
fi
mkdir -p "$RESULTS_DIR"

SLUG=$(slugify "$NAME")
if [[ -z "$RESULTS_PATH" ]]; then
  RESULTS_PATH="$RESULTS_DIR/$SLUG.tsv"
else
  RESULTS_PATH=$(abspath "$RESULTS_PATH")
fi

if [[ -z "$TEST_NAME" ]]; then
  TEST_NAME="$SLUG"
fi

RAW_EXPERIMENT_DIR="$REPO/experiments/$TEST_NAME"
RAW_STATS_CSV="$RAW_EXPERIMENT_DIR/stats.csv"
RAW_INFO_LOG="$RAW_EXPERIMENT_DIR/info.log"
mkdir -p "$RAW_EXPERIMENT_DIR"

cd "$REPO"

cmd=(
  "$PYTHON_BIN" measureSolvingTime.py
  --test_traces_folder "$TRACES_FOLDER"
  --timeout "$TIMEOUT_SECS"
  --test_name "$TEST_NAME"
  --encoder "$ENCODER"
  --iteration_start_value "$ITERATION_START"
  --iteration_step "$ITERATION_STEP"
  --dt_subset_size "$DT_SUBSET_SIZE"
  --dt_decrease_rate "$DT_DECREASE_RATE"
  --dt_restarts_of_sampling "$DT_RESTARTS"
  --dt_repetitions_inside_sampling "$DT_REPETITIONS"
  --dt_strategy "$DT_STRATEGY"
)
case "$METHOD" in
  sat) cmd+=(--test_sat_method) ;;
  dt) cmd+=(--test_dt_method) ;;
  both) cmd+=(--test_sat_method --test_dt_method) ;;
esac

"${cmd[@]}"

[[ -f "$RAW_STATS_CSV" ]] || die "samples2LTL did not produce stats.csv at $RAW_STATS_CSV"

python3 - "$RAW_STATS_CSV" "$RESULTS_PATH" "$HEADER" "$NAME" "$METHOD" "$RAW_EXPERIMENT_DIR" <<'PY'
import csv
import os
import sys

raw_csv, out_tsv, header, group, method, raw_dir = sys.argv[1:]

def clean(value):
    if value is None:
        return ""
    return str(value).replace("\t", " ").replace("\n", " ")

def parse_ms(value):
    value = (value or "").strip()
    if not value or value == "/":
        return ""
    try:
        return str(int(float(value) * 1000))
    except ValueError:
        return ""

def row_status(row):
    sat_time = (row.get("solving time sat") or "").strip()
    sat_formula = (row.get("obtained formula sat") or "").strip()
    sat_comment = (row.get("comment sat") or "").strip()
    dt_time = (row.get("dt solving time") or "").strip()
    dt_prims = (row.get("dt num primitives used") or "").strip()
    if method == "sat":
        if sat_time and sat_time != "/":
            return "solved"
        if "timeout" in sat_comment.lower():
            return "timeout"
        return "failed"
    if method == "dt":
        if dt_time and dt_time != "/":
            return "solved"
        return "failed"
    sat_ok = bool(sat_time and sat_time != "/")
    dt_ok = bool(dt_time and dt_time != "/")
    if sat_ok or dt_ok:
        return "solved"
    if "timeout" in sat_comment.lower():
        return "partial_timeout"
    return "failed"

with open(raw_csv, newline="", encoding="utf-8", errors="replace") as f, open(
    out_tsv, "w", encoding="utf-8"
) as out:
    reader = csv.DictReader(f)
    out.write(header + "\n")
    for row in reader:
        path = row.get("name of the test file", "")
        sat_ms = parse_ms(row.get("solving time sat", ""))
        dt_ms = parse_ms(row.get("dt solving time", ""))
        total_ms = sat_ms or dt_ms or ""
        obtained_formula = row.get("obtained formula sat", "")
        comment = row.get("comment sat", "")
        if method == "dt":
            obtained_formula = row.get("tree txt representation", "")
            comment = ""
        elif method == "both":
            comment = clean(comment)
            if dt_ms:
                comment = clean((comment + " | " if comment else "") + f"dt_ms={dt_ms}")
        fields = [
            clean(group),
            clean(path),
            clean(row_status(row)),
            clean(total_ms),
            clean(method),
            clean(row.get("number of accepting traces", "")),
            clean(row.get("number or rejecting traces", "")),
            clean(row.get("depth of formula", "")),
            clean(row.get("number of variables", "")),
            clean(row.get("max length of traces", "")),
            clean(row.get("specified formula", "")),
            clean(obtained_formula),
            clean(comment),
            clean(row.get("dt num primitive candidates", "")),
            clean(row.get("dt num primitives used", "")),
            clean(row.get("dt subset sizes", "")),
            clean(row.get("tree txt representation", "")),
            clean(raw_csv),
        ]
        out.write("\t".join(fields) + "\n")
PY

printf '%s\n' "$HEADER"
tail -n +2 "$RESULTS_PATH"
