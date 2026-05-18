#!/usr/bin/env bash
set -euo pipefail

HEADER=$'group\tpath\tstatus\ttotal_ms\talgo\tsolver\tformula_size\tnum_formulas\tresult_lines\tresult_formula\tstderr_summary\tresult_file'

usage() {
  cat <<'EOF'
Usage:
  bench-syslite.sh [options]

Options:
  --repo PATH                 SySLite checkout. Default: /root/SySLite if present,
                              otherwise /Users/wuchentian/Codes/Python/SySLite
  --path PATH                 Trace file or folder. Default: eas-example
  --recursive                 Recurse into subdirectories when PATH is a folder.
  --name NAME                 Benchmark group name. Default: ltl-syslite
  --results PATH              Explicit TSV output path.
  --results-dir PATH          Results root. Default: bench-results
  --python CMD                Python executable. Default: python3
  --timeout-secs N            Per-case timeout. Default: 600
  --algo NAME                 sat | sat_enum | sat_bound | dt | scikit_dt |
                              guided_sat | guided_sat_enum | fin_adt | adt_sygus | bv_sygus
                              Default: bv_sygus
  --solver NAME               z3 | msat | cvc4 | yices | boolector. Default: z3
  --formula-size N            `-s` argument. Default: 5
  --num-formulas N            `-n` argument. Default: 1
  --dict-support              Pass `-dict`.
  --help                      Show this help.
EOF
}

die() {
  echo "bench-syslite: $*" >&2
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

run_command() {
  local timeout_secs=$1
  local cwd=$2
  shift
  shift
  local meta_file stdout_file stderr_file
  meta_file=$(mktemp "${TMPDIR:-/tmp}/bench-syslite-meta.XXXXXX")
  stdout_file=$(mktemp "${TMPDIR:-/tmp}/bench-syslite-stdout.XXXXXX")
  stderr_file=$(mktemp "${TMPDIR:-/tmp}/bench-syslite-stderr.XXXXXX")
  python3 - "$timeout_secs" "$cwd" "$stdout_file" "$stderr_file" "$meta_file" "$@" <<'PY'
import subprocess, sys, time
timeout_secs = float(sys.argv[1])
cwd, stdout_path, stderr_path, meta_path = sys.argv[2:6]
cmd = sys.argv[6:]
stdout = open(stdout_path, 'wb')
stderr = open(stderr_path, 'wb')
start = time.monotonic()
timed_out = 0
returncode = None
try:
    proc = subprocess.Popen(cmd, stdout=stdout, stderr=stderr, cwd=cwd)
    try:
        returncode = proc.wait(timeout=timeout_secs)
    except subprocess.TimeoutExpired:
        timed_out = 1
        proc.kill()
        returncode = proc.wait()
finally:
    stdout.close()
    stderr.close()
duration_ms = int((time.monotonic() - start) * 1000)
with open(meta_path, 'w', encoding='utf-8') as fh:
    fh.write(f"timed_out={timed_out}\n")
    fh.write(f"returncode={'' if returncode is None else returncode}\n")
    fh.write(f"duration_ms={duration_ms}\n")
PY
  printf '%s\n%s\n%s\n' "$meta_file" "$stdout_file" "$stderr_file"
}

read_meta_value() {
  local meta_file=$1
  local key=$2
  awk -F= -v k="$key" '$1 == k {print substr($0, length($1) + 2)}' "$meta_file"
}

summarize_text_file() {
  python3 - "$1" <<'PY'
import sys
data = open(sys.argv[1], 'rb').read().decode('utf-8', errors='replace')
parts = [line.strip() for line in data.splitlines() if line.strip()]
text = " | ".join(parts).replace('\t', ' ')
print(text[:240] + "..." if len(text) > 240 else text)
PY
}

count_nonempty_lines() {
  python3 - "$1" <<'PY'
import sys
data = open(sys.argv[1], 'rb').read().decode('utf-8', errors='replace')
print(sum(1 for line in data.splitlines() if line.strip()))
PY
}

REPO=""
INPUT_PATH="eas-example"
RECURSIVE=0
NAME="ltl-syslite"
RESULTS_PATH=""
RESULTS_DIR=""
PYTHON_BIN="python3"
TIMEOUT_SECS=600
ALGO="bv_sygus"
SOLVER="z3"
FORMULA_SIZE=5
NUM_FORMULAS=1
DICT_SUPPORT=0

while (($#)); do
  case "$1" in
    --repo)
      (($# >= 2)) || die "--repo requires a value"
      REPO=$2
      shift 2
      ;;
    --path)
      (($# >= 2)) || die "--path requires a value"
      INPUT_PATH=$2
      shift 2
      ;;
    --recursive)
      RECURSIVE=1
      shift
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
    --algo)
      (($# >= 2)) || die "--algo requires a value"
      ALGO=$2
      shift 2
      ;;
    --solver)
      (($# >= 2)) || die "--solver requires a value"
      SOLVER=$2
      shift 2
      ;;
    --formula-size)
      (($# >= 2)) || die "--formula-size requires a value"
      FORMULA_SIZE=$2
      shift 2
      ;;
    --num-formulas)
      (($# >= 2)) || die "--num-formulas requires a value"
      NUM_FORMULAS=$2
      shift 2
      ;;
    --dict-support)
      DICT_SUPPORT=1
      shift
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
  if [[ -d /root/SySLite ]]; then
    REPO=/root/SySLite
  else
    REPO=/Users/wuchentian/Codes/Python/SySLite
  fi
fi
REPO=$(abspath "$REPO")
[[ -d "$REPO" ]] || die "repo not found: $REPO"
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

BASE_INPUT="$REPO/$INPUT_PATH"
if [[ ! -e "$BASE_INPUT" ]]; then
  BASE_INPUT=$(abspath "$INPUT_PATH")
fi
[[ -e "$BASE_INPUT" ]] || die "input path not found: $INPUT_PATH"

declare -a FILES=()
if [[ -d "$BASE_INPUT" ]]; then
  if ((RECURSIVE)); then
    while IFS= read -r file; do FILES+=("$file"); done < <(find "$BASE_INPUT" -type f -name '*.trace' | sort)
  else
    while IFS= read -r file; do FILES+=("$file"); done < <(find "$BASE_INPUT" -maxdepth 1 -type f -name '*.trace' | sort)
  fi
else
  FILES+=("$(abspath "$BASE_INPUT")")
fi

((${#FILES[@]} > 0)) || die "no .trace files found"

printf '%s\n' "$HEADER" >"$RESULTS_PATH"

for file in "${FILES[@]}"; do
  relative=$(python3 - "$file" "$REPO" <<'PY'
import os, sys
path = os.path.abspath(sys.argv[1])
base = os.path.abspath(sys.argv[2])
try:
    print(os.path.relpath(path, base))
except ValueError:
    print(path)
PY
)
  result_file=$(mktemp "${TMPDIR:-/tmp}/syslite-result.XXXXXX")
  cmd=( "$PYTHON_BIN" "$REPO/src/Driver.py" -s "$FORMULA_SIZE" -n "$NUM_FORMULAS" -a "$ALGO" -solver "$SOLVER" -t "$file" -r "$result_file" )
  ((DICT_SUPPORT)) && cmd+=( -dict )
  outputs=()
  while IFS= read -r line; do
    outputs+=("$line")
  done < <(run_command "$TIMEOUT_SECS" "$REPO" "${cmd[@]}")
  meta_file=${outputs[0]}
  stdout_file=${outputs[1]}
  stderr_file=${outputs[2]}
  timed_out=$(read_meta_value "$meta_file" timed_out)
  returncode=$(read_meta_value "$meta_file" returncode)
  duration_ms=$(read_meta_value "$meta_file" duration_ms)
  stderr_summary=$(summarize_text_file "$stderr_file")
  result_lines=0
  result_formula=""
  if [[ -f "$result_file" ]]; then
    result_lines=$(count_nonempty_lines "$result_file")
    result_formula=$(python3 - "$result_file" <<'PY'
import sys
lines = [line.strip() for line in open(sys.argv[1], 'r', encoding='utf-8', errors='replace').read().splitlines() if line.strip()]
print(lines[-1] if lines else "")
PY
)
  fi
  status="error"
  if [[ "$timed_out" == "1" ]]; then
    status="timeout"
  elif [[ "$returncode" == "0" ]]; then
    if [[ "$result_lines" != "0" ]]; then
      status="solved"
    else
      status="empty"
    fi
  fi
  row=$(printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s' \
    "$(sanitize_field "$NAME")" \
    "$(sanitize_field "$relative")" \
    "$status" \
    "$duration_ms" \
    "$(sanitize_field "$ALGO")" \
    "$(sanitize_field "$SOLVER")" \
    "$FORMULA_SIZE" \
    "$NUM_FORMULAS" \
    "$result_lines" \
    "$(sanitize_field "$result_formula")" \
    "$(sanitize_field "$stderr_summary")" \
    "$(sanitize_field "$result_file")")
  printf '%s\n' "$row" >>"$RESULTS_PATH"
  rm -f "$meta_file" "$stdout_file" "$stderr_file"
done

printf '%s\n' "$HEADER"
tail -n +2 "$RESULTS_PATH"
