#!/usr/bin/env bash
set -euo pipefail

HEADER=$'group\tpath\tstatus\ttotal_ms\tsygus_ms\tsolve_ms\tsygus_bytes\tsygus_lines\tsolution_bytes\tdelis_exit_code\tsolver_exit_code\tdelis_args\tsolver\tsolver_args\tsolution_path\tdetail'

usage() {
  cat <<'EOF'
Usage:
  delis-bench --path PATH [--path PATH ...] [options]
  bench.sh [--restart] [--rerun] [shared options]

Options:
  --path PATH                  Benchmark file or folder. Required. Repeatable.
  --name NAME                  Benchmark group name. Default: ad-hoc
  --delis PATH                 Delis binary. Default: $DELIS_BENCH_SOLVER or sibling delis
  --mode MODE                  native | external_sygus. Default: native
  --delis-arg ARG             Extra argument passed to delis. Repeatable.
  --recursive                  Recurse into benchmark folders.
  --sygus-solver CMD           External SyGuS solver. Default: cvc5
  --sygus-solver-arg ARG       Extra external solver arg. Repeatable.
  --sygus-timeout-secs N       Timeout for --dump-sygus generation. Default: 600
  --solve-timeout-secs N       Timeout for native solve or external solver. Default: 600
  --results-dir PATH           Results root. Default: bench-results
  --results PATH               Explicit TSV output path.
  --rerun                      Update existing results in place and resume safely.
  --restart                    Ignore prior rows and rerun everything.
  --shard-count N              Deterministically partition files into N shards. Default: 1
  --shard-index I              Zero-based shard index to run. Default: 0
  --all-existing               Run the built-in benchmark groups matching existing bench-results names.
  --help                       Show this help.
EOF
}

die() {
  echo "delis-bench: $*" >&2
  exit 1
}

slugify() {
  python3 - "$1" <<'PY'
import re, sys
slug = re.sub(r'[^0-9A-Za-z]+', '-', sys.argv[1]).strip('-').lower()
print(slug or "results")
PY
}

default_solver_args() {
  local solver_name
  solver_name=$(basename "$1")
  case "$solver_name" in
    dryadsynth-bv) printf '%s\n' "-in" ;;
    cvc5) printf '%s\n' "--lang=sygus2" ;;
  esac
}

resolve_delis_path() {
  if [[ -n "${DELIS_BIN}" ]]; then
    printf '%s\n' "$DELIS_BIN"
    return
  fi
  if [[ -n "${DELIS_BENCH_SOLVER:-}" ]]; then
    printf '%s\n' "$DELIS_BENCH_SOLVER"
    return
  fi
  if command -v delis >/dev/null 2>&1; then
    command -v delis
    return
  fi
  local script_dir
  script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
  printf '%s\n' "$(cd "$script_dir/.." && pwd)/target/debug/delis"
}

abspath() {
  python3 - "$1" <<'PY'
import os, sys
print(os.path.abspath(sys.argv[1]))
PY
}

relative_display() {
  python3 - "$1" "$2" <<'PY'
import os, sys
path = os.path.abspath(sys.argv[1])
base = os.path.abspath(sys.argv[2])
try:
    print(os.path.relpath(path, base))
except ValueError:
    print(path)
PY
}

common_parent() {
  python3 - "$@" <<'PY'
import os, sys
paths = [os.path.abspath(p) for p in sys.argv[1:]]
if not paths:
    print(".")
else:
    parents = [p if os.path.isdir(p) else os.path.dirname(p) for p in paths]
    print(os.path.commonpath(parents))
PY
}

sanitize_field() {
  python3 - "$1" <<'PY'
import sys
print(sys.argv[1].replace('\t', ' ').replace('\n', ' '))
PY
}

count_lines() {
  python3 - "$1" <<'PY'
import sys
path = sys.argv[1]
with open(path, 'rb') as fh:
    data = fh.read()
if not data:
    print(0)
else:
    print(len(data.decode('utf-8', errors='replace').splitlines()))
PY
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

ensure_results_header() {
  local path=$1
  mkdir -p "$(dirname "$path")"
  if [[ ! -f "$path" ]]; then
    printf '%s\n' "$HEADER" >"$path"
  fi
}

table_has_path() {
  local results_path=$1
  local relative=$2
  python3 - "$results_path" "$relative" <<'PY'
import sys
results_path, wanted = sys.argv[1:]
try:
    with open(results_path, 'r', encoding='utf-8', errors='replace') as fh:
        for line in fh:
            if not line or line.startswith("group\tpath\t"):
                continue
            cols = line.rstrip('\n').split('\t')
            if len(cols) > 1 and cols[1] == wanted:
                raise SystemExit(0)
except FileNotFoundError:
    pass
raise SystemExit(1)
PY
}

update_results_row() {
  local results_path=$1
  local row=$2
  python3 - "$results_path" "$HEADER" "$row" <<'PY'
import os, sys
results_path, header, row = sys.argv[1:]
target = row.split('\t', 2)[1]
rows = []
seen = False
if os.path.exists(results_path):
    with open(results_path, 'r', encoding='utf-8', errors='replace') as fh:
        for line in fh:
            line = line.rstrip('\n')
            if not line or line.startswith("group\tpath\t"):
                continue
            cols = line.split('\t', 2)
            if len(cols) > 1 and cols[1] == target:
                rows.append(row)
                seen = True
            else:
                rows.append(line)
if not seen:
    rows.append(row)
tmp = results_path + ".tmp"
with open(tmp, 'w', encoding='utf-8') as fh:
    fh.write(header + "\n")
    for item in rows:
        fh.write(item + "\n")
os.replace(tmp, results_path)
PY
}

append_results_row() {
  local results_path=$1
  local row=$2
  printf '%s\n' "$row" >>"$results_path"
}

write_progress_marker() {
  local progress_path=$1
  local relative=$2
  mkdir -p "$(dirname "$progress_path")"
  printf '%s\n' "$relative" >"$progress_path"
}

load_progress_marker() {
  local progress_path=$1
  if [[ -f "$progress_path" ]]; then
    head -n 1 "$progress_path" | tr -d '\r\n'
  fi
}

clear_progress_marker() {
  local progress_path=$1
  rm -f "$progress_path"
}

with_lock() {
  local lock_path=$1
  shift
  mkdir -p "$(dirname "$lock_path")"
  while ! mkdir "$lock_path" 2>/dev/null; do
    sleep 0.1
  done
  "$@"
  rmdir "$lock_path"
}

timestamp_utc() {
  python3 - <<'PY'
from datetime import datetime, timezone
print(datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
PY
}

timestamp_slug_utc() {
  python3 - <<'PY'
from datetime import datetime, timezone
print(datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"))
PY
}

append_log() {
  local log_path=$1
  shift
  mkdir -p "$(dirname "$log_path")"
  printf '[%s] %s\n' "$(timestamp_utc)" "$*" >>"$log_path"
}

artifact_path() {
  python3 - "$1" "$2" "$3" <<'PY'
import os, sys
artifacts_dir, folder_root, file_path = [os.path.abspath(x) for x in sys.argv[1:]]
try:
    rel = os.path.relpath(file_path, folder_root)
except ValueError:
    rel = os.path.basename(file_path)
print(os.path.join(artifacts_dir, rel))
PY
}

run_command() {
  local timeout_secs=$1
  local stdin_path=$2
  shift 2
  local meta_file stdout_file stderr_file
  meta_file=$(mktemp "${TMPDIR:-/tmp}/delis-bench-meta.XXXXXX")
  stdout_file=$(mktemp "${TMPDIR:-/tmp}/delis-bench-stdout.XXXXXX")
  stderr_file=$(mktemp "${TMPDIR:-/tmp}/delis-bench-stderr.XXXXXX")
  python3 - "$timeout_secs" "$stdin_path" "$stdout_file" "$stderr_file" "$meta_file" "$@" <<'PY'
import os, signal, subprocess, sys, time
timeout_secs = float(sys.argv[1])
stdin_path, stdout_path, stderr_path, meta_path = sys.argv[2:6]
cmd = sys.argv[6:]
stdin = open(stdin_path, 'rb') if stdin_path else subprocess.DEVNULL
stdout = open(stdout_path, 'wb')
stderr = open(stderr_path, 'wb')
start = time.monotonic()
timed_out = 0
returncode = None
proc = None
try:
    proc = subprocess.Popen(cmd, stdin=stdin, stdout=stdout, stderr=stderr)
    try:
        returncode = proc.wait(timeout=timeout_secs)
    except subprocess.TimeoutExpired:
        timed_out = 1
        proc.kill()
        returncode = proc.wait()
finally:
    if stdin not in (None, subprocess.DEVNULL):
        stdin.close()
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

run_existing_groups() {
  local shared_args=("$@")
  local groups=(
    "fo-finvar-popl22|benchmark/FO/finvar-popl22"
    "reg-alpharegex|benchmark/Reg/alpharegex"
    "fo-folseparators-fo2|benchmark/FO/folseparators/FO2"
    "fo-folseparators-fo3|benchmark/FO/folseparators/FO3"
    "ltl-learnltl-sat-abscence|benchmark/LTL/learnltl/sat-comparison/abscence"
    "ltl-learnltl-sat-universality|benchmark/LTL/learnltl/sat-comparison/universality"
    "ltl-learnltl-syslite|benchmark/LTL/learnltl/syslite-comparison"
    "ltl-sample2ltl|benchmark/LTL/sample2LTL"
    "reg-deepregex|benchmark/Reg/deepregex"
    "reg-so|benchmark/Reg/so"
    "ctl-learnctl-generated|benchmark/CTL/learnctl/generated"
    "fo-folseparators-fo1|benchmark/FO/folseparators/FO1"
    "fo-folseparators-fo0|benchmark/FO/folseparators/FO0"
    "fo-lfp-folseparators-fo1-lfp|benchmark/FO-LFP/folseparators/FO1-LFP"
    "fo-lfp-folseparators-fo2-lfp|benchmark/FO-LFP/folseparators/FO2-LFP"
  )
  local script_path
  script_path=$(abspath "${BASH_SOURCE[0]}")
  local entry name path
  for entry in "${groups[@]}"; do
    name=${entry%%|*}
    path=${entry#*|}
    bash "$script_path" --path "$path" --name "$name" "${shared_args[@]}"
  done
}

MODE="native"
NAME="ad-hoc"
DELIS_BIN=""
RECURSIVE=0
SYGUS_SOLVER=""
SYGUS_TIMEOUT_SECS=600
SOLVE_TIMEOUT_SECS=600
RESULTS_DIR=""
RESULTS_PATH=""
RERUN=0
RESTART=0
RUN_ALL_EXISTING=0
SHARD_COUNT=1
SHARD_INDEX=0
declare -a PATHS=()
declare -a DELIS_ARGS=()
declare -a SYGUS_SOLVER_ARGS=()

while (($#)); do
  case "$1" in
    --path)
      (($# >= 2)) || die "--path requires a value"
      PATHS+=("$2")
      shift 2
      ;;
    --name)
      (($# >= 2)) || die "--name requires a value"
      NAME=$2
      shift 2
      ;;
    --delis)
      (($# >= 2)) || die "--delis requires a value"
      DELIS_BIN=$2
      shift 2
      ;;
    --mode)
      (($# >= 2)) || die "--mode requires a value"
      MODE=$2
      shift 2
      ;;
    --delis-arg)
      (($# >= 2)) || die "--delis-arg requires a value"
      DELIS_ARGS+=("$2")
      shift 2
      ;;
    --recursive)
      RECURSIVE=1
      shift
      ;;
    --sygus-solver)
      (($# >= 2)) || die "--sygus-solver requires a value"
      SYGUS_SOLVER=$2
      shift 2
      ;;
    --sygus-solver-arg)
      (($# >= 2)) || die "--sygus-solver-arg requires a value"
      SYGUS_SOLVER_ARGS+=("$2")
      shift 2
      ;;
    --sygus-timeout-secs)
      (($# >= 2)) || die "--sygus-timeout-secs requires a value"
      SYGUS_TIMEOUT_SECS=$2
      shift 2
      ;;
    --solve-timeout-secs)
      (($# >= 2)) || die "--solve-timeout-secs requires a value"
      SOLVE_TIMEOUT_SECS=$2
      shift 2
      ;;
    --results-dir)
      (($# >= 2)) || die "--results-dir requires a value"
      RESULTS_DIR=$2
      shift 2
      ;;
    --results)
      (($# >= 2)) || die "--results requires a value"
      RESULTS_PATH=$2
      shift 2
      ;;
    --rerun)
      RERUN=1
      shift
      ;;
    --restart)
      RESTART=1
      shift
      ;;
    --shard-count)
      (($# >= 2)) || die "--shard-count requires a value"
      SHARD_COUNT=$2
      shift 2
      ;;
    --shard-index)
      (($# >= 2)) || die "--shard-index requires a value"
      SHARD_INDEX=$2
      shift 2
      ;;
    --all-existing)
      RUN_ALL_EXISTING=1
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

if ((RUN_ALL_EXISTING)) || ((${#PATHS[@]} == 0)); then
  shared_args=()
  [[ -n "$DELIS_BIN" ]] && shared_args+=(--delis "$DELIS_BIN")
  shared_args+=(--mode "$MODE")
  ((RECURSIVE)) && shared_args+=(--recursive)
  [[ -n "$SYGUS_SOLVER" ]] && shared_args+=(--sygus-solver "$SYGUS_SOLVER")
  shared_args+=(--sygus-timeout-secs "$SYGUS_TIMEOUT_SECS" --solve-timeout-secs "$SOLVE_TIMEOUT_SECS")
  [[ -n "$RESULTS_DIR" ]] && shared_args+=(--results-dir "$RESULTS_DIR")
  ((RERUN)) && shared_args+=(--rerun)
  ((RESTART)) && shared_args+=(--restart)
  shared_args+=(--shard-count "$SHARD_COUNT" --shard-index "$SHARD_INDEX")
  if ((${#DELIS_ARGS[@]} > 0)); then
    local_arg=
    for local_arg in "${DELIS_ARGS[@]}"; do
      shared_args+=(--delis-arg "$local_arg")
    done
  fi
  if ((${#SYGUS_SOLVER_ARGS[@]} > 0)); then
    local_arg=
    for local_arg in "${SYGUS_SOLVER_ARGS[@]}"; do
      shared_args+=(--sygus-solver-arg "$local_arg")
    done
  fi
  run_existing_groups "${shared_args[@]}"
  exit 0
fi

((${#PATHS[@]} > 0)) || die "at least one --path is required"
[[ "$MODE" == "native" || "$MODE" == "external_sygus" ]] || die "--mode must be native or external_sygus"
[[ "$SHARD_COUNT" =~ ^[0-9]+$ ]] || die "--shard-count must be a positive integer"
[[ "$SHARD_INDEX" =~ ^[0-9]+$ ]] || die "--shard-index must be a non-negative integer"
((SHARD_COUNT >= 1)) || die "--shard-count must be at least 1"
((SHARD_INDEX < SHARD_COUNT)) || die "--shard-index must be less than --shard-count"

CWD=$(pwd)
DELIS_BIN=$(resolve_delis_path)
[[ -x "$DELIS_BIN" ]] || die "delis binary not found or not executable: $DELIS_BIN"

if [[ -z "$RESULTS_DIR" ]]; then
  RESULTS_DIR="$CWD/bench-results"
else
  RESULTS_DIR=$(abspath "$RESULTS_DIR")
fi

SLUG=$(slugify "$NAME")
if [[ -z "$RESULTS_PATH" ]]; then
  RESULTS_PATH="$RESULTS_DIR/$SLUG.tsv"
else
  RESULTS_PATH=$(abspath "$RESULTS_PATH")
fi
ARTIFACTS_DIR="$RESULTS_DIR/artifacts/$SLUG"
SHARD_SUFFIX=""
if ((SHARD_COUNT > 1)); then
  SHARD_SUFFIX="-shard-${SHARD_INDEX}-of-${SHARD_COUNT}"
fi
PROGRESS_PATH="${RESULTS_PATH%.*}${SHARD_SUFFIX}.progress"
LOG_PATH="${RESULTS_PATH%.*}${SHARD_SUFFIX}-$(timestamp_slug_utc).log"
RESULTS_LOCK_PATH="${RESULTS_PATH}.lock"

declare -a FILES=()
for raw_path in "${PATHS[@]}"; do
  path=$(abspath "$raw_path")
  if [[ -d "$path" ]]; then
    if ((RECURSIVE)); then
      while IFS= read -r file; do FILES+=("$file"); done < <(find "$path" -type f -name '*.sl' | sort)
    else
      while IFS= read -r file; do FILES+=("$file"); done < <(find "$path" -maxdepth 1 -type f -name '*.sl' | sort)
    fi
  elif [[ -f "$path" && "$path" == *.sl ]]; then
    FILES+=("$path")
  else
    die "path is neither an .sl file nor a benchmark folder: $path"
  fi
done

((${#FILES[@]} > 0)) || die "no benchmark files found from --path"
deduped_files=()
while IFS= read -r file; do
  deduped_files+=("$file")
done < <(printf '%s\n' "${FILES[@]}" | sort -u)
FILES=("${deduped_files[@]}")
if ((SHARD_COUNT > 1)); then
  sharded_files=()
  file_idx=0
  for file in "${FILES[@]}"; do
    if (((file_idx % SHARD_COUNT) == SHARD_INDEX)); then
      sharded_files+=("$file")
    fi
    file_idx=$((file_idx + 1))
  done
  FILES=("${sharded_files[@]}")
fi
ROOT=$(common_parent "${FILES[@]}")

if [[ "$MODE" == "external_sygus" ]]; then
  if [[ -z "$SYGUS_SOLVER" ]]; then
    SYGUS_SOLVER=${DELIS_BENCH_SYGUS_SOLVER:-cvc5}
  fi
  if ((${#SYGUS_SOLVER_ARGS[@]} == 0)); then
    while IFS= read -r arg; do
      [[ -n "$arg" ]] && SYGUS_SOLVER_ARGS+=("$arg")
    done < <(default_solver_args "$SYGUS_SOLVER")
  fi
fi

printf '%s\n' "$HEADER"
append_log "$LOG_PATH" "start name=$NAME mode=$MODE root=$ROOT results=$(relative_display "$RESULTS_PATH" "$CWD") rerun=$( ((RERUN)) && echo true || echo false ) restart=$( ((RESTART)) && echo true || echo false ) shard_index=$SHARD_INDEX shard_count=$SHARD_COUNT files=${#FILES[@]}"

folder_paths_file=$(mktemp "${TMPDIR:-/tmp}/delis-bench-folder-paths.XXXXXX")
trap 'rm -f "$folder_paths_file"' EXIT
for file in "${FILES[@]}"; do
  relative=$(relative_display "$file" "$CWD")
  printf '%s\n' "$relative" >>"$folder_paths_file"
done

manage_results_table=0
if ((RERUN || RESTART)); then
  manage_results_table=1
fi

is_completed() {
  local relative=$1
  if ((RESTART)); then
    return 1
  fi
  table_has_path "$RESULTS_PATH" "$relative"
}

resume_path=$(load_progress_marker "$PROGRESS_PATH" || true)
if [[ -n "${resume_path:-}" ]]; then
  if grep -Fxq "$resume_path" "$folder_paths_file" && ! is_completed "$resume_path"; then
    echo "resuming folder $ROOT after interruption; last in-progress file was $resume_path" >&2
    append_log "$LOG_PATH" "resume last_in_progress=$resume_path"
  else
    clear_progress_marker "$PROGRESS_PATH"
  fi
fi

pending=0
for file in "${FILES[@]}"; do
  relative=$(relative_display "$file" "$CWD")
  if ! is_completed "$relative"; then
    pending=$((pending + 1))
  fi
done
completed_now=0

action="running folder"
if ((pending == 0)); then
  action="skipping folder"
fi
echo "$action $ROOT -> $(relative_display "$RESULTS_PATH" "$CWD") (${#FILES[@]} total, $pending pending, rerun=$( ((RERUN)) && echo true || echo false), restart=$( ((RESTART)) && echo true || echo false), mode=$MODE, shard=$SHARD_INDEX/$SHARD_COUNT)" >&2
append_log "$LOG_PATH" "$action total=${#FILES[@]} pending=$pending results=$(relative_display "$RESULTS_PATH" "$CWD") shard_index=$SHARD_INDEX shard_count=$SHARD_COUNT"
if ((pending == 0)); then
  clear_progress_marker "$PROGRESS_PATH"
  append_log "$LOG_PATH" "done pending=0"
  exit 0
fi

if (( ! manage_results_table )); then
  with_lock "$RESULTS_LOCK_PATH" ensure_results_header "$RESULTS_PATH"
fi

for file in "${FILES[@]}"; do
  relative=$(relative_display "$file" "$CWD")
  if is_completed "$relative"; then
    continue
  fi

  echo "  running $relative" >&2
  write_progress_marker "$PROGRESS_PATH" "$relative"
  append_log "$LOG_PATH" "running path=$relative"

  group_field=$(sanitize_field "$NAME")
  path_field=$(sanitize_field "$relative")
  delis_args_joined=$(sanitize_field "${DELIS_ARGS[*]:-}")

  if [[ "$MODE" == "native" ]]; then
    native_cmd=("$DELIS_BIN")
    if ((${#DELIS_ARGS[@]} > 0)); then
      native_cmd+=("${DELIS_ARGS[@]}")
    fi
    native_cmd+=("$file")
    command_outputs=()
    while IFS= read -r line; do
      command_outputs+=("$line")
    done < <(run_command "$SOLVE_TIMEOUT_SECS" "" "${native_cmd[@]}")
    meta_file=${command_outputs[0]}
    stdout_file=${command_outputs[1]}
    stderr_file=${command_outputs[2]}
    timed_out=$(read_meta_value "$meta_file" timed_out)
    returncode=$(read_meta_value "$meta_file" returncode)
    duration_ms=$(read_meta_value "$meta_file" duration_ms)
    stdout_size=$(wc -c <"$stdout_file" | tr -d ' ')
    stderr_summary=$(summarize_text_file "$stderr_file")
    stdout_text=$(python3 - "$stdout_file" <<'PY'
import sys
print(open(sys.argv[1], 'rb').read().decode('utf-8', errors='replace').strip())
PY
)
    status="error"
    detail=""
    solution_path=""
    delis_exit_code="$returncode"
    if [[ "$timed_out" == "1" ]]; then
      status="solve_timeout"
      if [[ -z "$stderr_summary" ]]; then
        detail="delis timed out after $(python3 - "$duration_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
)"
      else
        detail="delis timed out after $(python3 - "$duration_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
); $stderr_summary"
      fi
      delis_exit_code=""
    elif [[ "$returncode" == "0" ]]; then
      if [[ "$stdout_text" == "infeasible" ]]; then
        status="infeasible"
      else
        status="solved"
      fi
      detail="$stderr_summary"
      artifact=$(artifact_path "$ARTIFACTS_DIR" "$ROOT" "$file")
      mkdir -p "$(dirname "$artifact")"
      cp "$stdout_file" "$artifact"
      solution_path=$(sanitize_field "$(relative_display "$artifact" "$CWD")")
    else
      status="error"
      if [[ -z "$stderr_summary" ]]; then
        detail="delis exited with status $returncode"
      else
        detail="$stderr_summary"
      fi
      solution_path=""
    fi
    row=$(printf '%s\t%s\t%s\t%s\t0\t%s\t0\t0\t%s\t%s\t\t%s\t\t\t%s\t%s' \
      "$group_field" "$path_field" "$status" "$duration_ms" "$duration_ms" "$stdout_size" \
      "${delis_exit_code:-}" "$delis_args_joined" "$solution_path" "$(sanitize_field "$detail")")
    rm -f "$meta_file" "$stdout_file" "$stderr_file"
  else
    generation_cmd=("$DELIS_BIN")
    if ((${#DELIS_ARGS[@]} > 0)); then
      generation_cmd+=("${DELIS_ARGS[@]}")
    fi
    generation_cmd+=(--dump-sygus "$file")
    generation_outputs=()
    while IFS= read -r line; do
      generation_outputs+=("$line")
    done < <(run_command "$SYGUS_TIMEOUT_SECS" "" "${generation_cmd[@]}")
    gen_meta=${generation_outputs[0]}
    gen_stdout=${generation_outputs[1]}
    gen_stderr=${generation_outputs[2]}
    gen_timed_out=$(read_meta_value "$gen_meta" timed_out)
    gen_code=$(read_meta_value "$gen_meta" returncode)
    gen_ms=$(read_meta_value "$gen_meta" duration_ms)
    sygus_bytes=$(wc -c <"$gen_stdout" | tr -d ' ')
    sygus_lines=$(count_lines "$gen_stdout")
    gen_stderr_summary=$(summarize_text_file "$gen_stderr")
    status="error"
    detail=""
    solution_path=""
    solution_bytes=0
    solver_exit_code=""
    delis_exit_code="$gen_code"
    solve_ms=0
    if [[ "$gen_timed_out" == "1" ]]; then
      status="sygus_timeout"
      delis_exit_code=""
      if [[ -z "$gen_stderr_summary" ]]; then
        detail="SyGuS generation timed out after $(python3 - "$gen_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
)"
      else
        detail="SyGuS generation timed out after $(python3 - "$gen_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
); $gen_stderr_summary"
      fi
    elif [[ "$gen_code" != "0" ]]; then
      status="error"
      if [[ -z "$gen_stderr_summary" ]]; then
        detail="delis exited with status $gen_code"
      else
        detail="$gen_stderr_summary"
      fi
    else
      solve_cmd=("$SYGUS_SOLVER")
      if ((${#SYGUS_SOLVER_ARGS[@]} > 0)); then
        solve_cmd+=("${SYGUS_SOLVER_ARGS[@]}")
      fi
      solve_outputs=()
      while IFS= read -r line; do
        solve_outputs+=("$line")
      done < <(run_command "$SOLVE_TIMEOUT_SECS" "$gen_stdout" "${solve_cmd[@]}")
      solve_meta=${solve_outputs[0]}
      solve_stdout=${solve_outputs[1]}
      solve_stderr=${solve_outputs[2]}
      solve_timed_out=$(read_meta_value "$solve_meta" timed_out)
      solver_exit_code=$(read_meta_value "$solve_meta" returncode)
      solve_ms=$(read_meta_value "$solve_meta" duration_ms)
      solve_stderr_summary=$(summarize_text_file "$solve_stderr")
      solution_bytes=$(wc -c <"$solve_stdout" | tr -d ' ')
      solve_stdout_text=$(python3 - "$solve_stdout" <<'PY'
import sys
print(open(sys.argv[1], 'rb').read().decode('utf-8', errors='replace').strip())
PY
)
      if [[ "$solve_timed_out" == "1" ]]; then
        status="solve_timeout"
        solver_exit_code=""
        if [[ -z "$solve_stderr_summary" ]]; then
          detail="solver timed out after $(python3 - "$solve_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
)"
        else
          detail="solver timed out after $(python3 - "$solve_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
); $solve_stderr_summary"
        fi
      elif [[ "$solver_exit_code" == "0" ]]; then
        if [[ "$solve_stdout_text" == "infeasible" ]]; then
          status="infeasible"
        else
          status="solved"
        fi
        detail="$solve_stderr_summary"
        if ((solution_bytes > 0)); then
          artifact=$(artifact_path "$ARTIFACTS_DIR" "$ROOT" "$file")
          mkdir -p "$(dirname "$artifact")"
          cp "$solve_stdout" "$artifact"
          solution_path=$(sanitize_field "$(relative_display "$artifact" "$CWD")")
        fi
      else
        status="error"
        if [[ -z "$solve_stderr_summary" ]]; then
          detail="solver exited with status $solver_exit_code"
        else
          detail="$solve_stderr_summary"
        fi
      fi
    fi
    solver_field=$(sanitize_field "$SYGUS_SOLVER")
    solver_args_field=$(sanitize_field "${SYGUS_SOLVER_ARGS[*]:-}")
    row=$(printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s' \
      "$group_field" "$path_field" "$status" "$((gen_ms + solve_ms))" "$gen_ms" "$solve_ms" \
      "$sygus_bytes" "$sygus_lines" "$solution_bytes" "${delis_exit_code:-}" "${solver_exit_code:-}" \
      "$delis_args_joined" "$solver_field" "$solver_args_field" "$solution_path" "$(sanitize_field "$detail")")
    rm -f "$gen_meta" "$gen_stdout" "$gen_stderr"
    if [[ -n "${solve_meta:-}" ]]; then
      rm -f "$solve_meta" "$solve_stdout" "$solve_stderr"
    fi
  fi

  if ((manage_results_table)); then
    with_lock "$RESULTS_LOCK_PATH" update_results_row "$RESULTS_PATH" "$row"
  else
    with_lock "$RESULTS_LOCK_PATH" append_results_row "$RESULTS_PATH" "$row"
  fi
  printf '%s\n' "$row"
  append_log "$LOG_PATH" "row $(sanitize_field "$row")"
  completed_now=$((completed_now + 1))
  clear_progress_marker "$PROGRESS_PATH"
done

clear_progress_marker "$PROGRESS_PATH"
append_log "$LOG_PATH" "done completed_now=$completed_now total=${#FILES[@]}"
