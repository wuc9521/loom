#!/usr/bin/env bash
set -euo pipefail

HEADER=$'group\tpath\tstatus\ttotal_ms\tsygus_ms\tsolve_ms\tsygus_bytes\tsygus_lines\tsolution_bytes\tdelis_exit_code\tsolver_exit_code\tdelis_args\tsolver\tsolver_args\tsolution_path\tdetail'

usage() {
  cat <<'EOF'
Usage:
  bench-cvc5.sh --path PATH [--path PATH ...] [options]

Options:
  --path PATH                  SyGuS file or folder. Required. Repeatable.
  --name NAME                  Benchmark group name. Default: ad-hoc-cvc5
  --solver CMD                 SyGuS solver. Default: cvc5
  --solver-arg ARG             Extra solver arg. Repeatable.
  --recursive                  Recurse into benchmark folders.
  --solve-timeout-secs N       Timeout for solver. Default: 600
  --results-dir PATH           Results root. Default: bench-results
  --results PATH               Explicit TSV output path.
  --rerun                      Update existing results in place and resume safely.
  --restart                    Ignore prior rows and rerun everything.
  --shard-count N              Deterministically partition files into N shards. Default: 1
  --shard-index I              Zero-based shard index to run. Default: 0
  --all-existing               Run the built-in regex SyGuS benchmark groups.
  --help                       Show this help.
EOF
}

die() {
  echo "bench-cvc5: $*" >&2
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
  meta_file=$(mktemp "${TMPDIR:-/tmp}/bench-cvc5-meta.XXXXXX")
  stdout_file=$(mktemp "${TMPDIR:-/tmp}/bench-cvc5-stdout.XXXXXX")
  stderr_file=$(mktemp "${TMPDIR:-/tmp}/bench-cvc5-stderr.XXXXXX")
  python3 - "$timeout_secs" "$stdin_path" "$stdout_file" "$stderr_file" "$meta_file" "$@" <<'PY'
import subprocess, sys, time
timeout_secs = float(sys.argv[1])
stdin_path, stdout_path, stderr_path, meta_path = sys.argv[2:6]
cmd = sys.argv[6:]
stdin = open(stdin_path, 'rb') if stdin_path else subprocess.DEVNULL
stdout = open(stdout_path, 'wb')
stderr = open(stderr_path, 'wb')
start = time.monotonic()
timed_out = 0
returncode = None
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
    "reg-alpharegex-cvc5|benchmark/Reg/alpharegex/sygus"
    "reg-deepregex-cvc5|benchmark/Reg/deepregex/sygus"
    "reg-so-cvc5|benchmark/Reg/so/sygus"
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

NAME="ad-hoc-cvc5"
SOLVER="cvc5"
SOLVE_TIMEOUT_SECS=600
RESULTS_DIR=""
RESULTS_PATH=""
RERUN=0
RESTART=0
RECURSIVE=0
RUN_ALL_EXISTING=0
SHARD_COUNT=1
SHARD_INDEX=0
declare -a PATHS=()
declare -a SOLVER_ARGS=()

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
    --solver)
      (($# >= 2)) || die "--solver requires a value"
      SOLVER=$2
      shift 2
      ;;
    --solver-arg)
      (($# >= 2)) || die "--solver-arg requires a value"
      SOLVER_ARGS+=("$2")
      shift 2
      ;;
    --recursive)
      RECURSIVE=1
      shift
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
  [[ -n "$SOLVER" ]] && shared_args+=(--solver "$SOLVER")
  ((RECURSIVE)) && shared_args+=(--recursive)
  shared_args+=(--solve-timeout-secs "$SOLVE_TIMEOUT_SECS")
  [[ -n "$RESULTS_DIR" ]] && shared_args+=(--results-dir "$RESULTS_DIR")
  ((RERUN)) && shared_args+=(--rerun)
  ((RESTART)) && shared_args+=(--restart)
  shared_args+=(--shard-count "$SHARD_COUNT" --shard-index "$SHARD_INDEX")
  if ((${#SOLVER_ARGS[@]} > 0)); then
    local_arg=
    for local_arg in "${SOLVER_ARGS[@]}"; do
      shared_args+=(--solver-arg "$local_arg")
    done
  fi
  run_existing_groups "${shared_args[@]}"
  exit 0
fi

((${#PATHS[@]} > 0)) || die "at least one --path is required"
[[ "$SHARD_COUNT" =~ ^[0-9]+$ ]] || die "--shard-count must be a positive integer"
[[ "$SHARD_INDEX" =~ ^[0-9]+$ ]] || die "--shard-index must be a non-negative integer"
((SHARD_COUNT >= 1)) || die "--shard-count must be at least 1"
((SHARD_INDEX < SHARD_COUNT)) || die "--shard-index must be less than --shard-count"

CWD=$(pwd)
if ! command -v "$SOLVER" >/dev/null 2>&1; then
  die "solver not found in PATH: $SOLVER"
fi
if ((${#SOLVER_ARGS[@]} == 0)) && [[ "$(basename "$SOLVER")" == "cvc5" ]]; then
  SOLVER_ARGS+=(--lang=sygus2)
fi

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

printf '%s\n' "$HEADER"
append_log "$LOG_PATH" "start name=$NAME solver=$SOLVER root=$ROOT results=$(relative_display "$RESULTS_PATH" "$CWD") rerun=$( ((RERUN)) && echo true || echo false ) restart=$( ((RESTART)) && echo true || echo false ) shard_index=$SHARD_INDEX shard_count=$SHARD_COUNT files=${#FILES[@]}"

folder_paths_file=$(mktemp "${TMPDIR:-/tmp}/bench-cvc5-folder-paths.XXXXXX")
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

action="running folder"
if ((pending == 0)); then
  action="skipping folder"
fi
echo "$action $ROOT -> $(relative_display "$RESULTS_PATH" "$CWD") (${#FILES[@]} total, $pending pending, rerun=$( ((RERUN)) && echo true || echo false), restart=$( ((RESTART)) && echo true || echo false), solver=$SOLVER, shard=$SHARD_INDEX/$SHARD_COUNT)" >&2
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
  sygus_bytes=$(wc -c <"$file" | tr -d ' ')
  sygus_lines=$(count_lines "$file")
  solver_args_joined=$(sanitize_field "${SOLVER_ARGS[*]:-}")
  solve_cmd=("$SOLVER")
  if ((${#SOLVER_ARGS[@]} > 0)); then
    solve_cmd+=("${SOLVER_ARGS[@]}")
  fi
  solve_cmd+=("$file")
  command_outputs=()
  while IFS= read -r line; do
    command_outputs+=("$line")
  done < <(run_command "$SOLVE_TIMEOUT_SECS" "" "${solve_cmd[@]}")
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
  solver_exit_code="$returncode"
  if [[ "$timed_out" == "1" ]]; then
    status="solve_timeout"
    if [[ -z "$stderr_summary" ]]; then
      detail="solver timed out after $(python3 - "$duration_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
)"
    else
      detail="solver timed out after $(python3 - "$duration_ms" <<'PY'
import sys
print(f"{int(sys.argv[1])/1000:.3f}s")
PY
); $stderr_summary"
    fi
    solver_exit_code=""
  elif [[ "$returncode" == "0" ]]; then
    if [[ -z "$stdout_text" || "$stdout_text" == "unknown" ]]; then
      status="unknown"
      detail="${stderr_summary:-solver returned unknown}"
    else
      status="solved"
      detail="$stderr_summary"
      artifact=$(artifact_path "$ARTIFACTS_DIR" "$ROOT" "$file")
      mkdir -p "$(dirname "$artifact")"
      cp "$stdout_file" "$artifact"
      solution_path=$(sanitize_field "$(relative_display "$artifact" "$CWD")")
    fi
  else
    if [[ -n "$stdout_text" && "$stdout_text" != "unknown" ]]; then
      status="solved"
      detail="$stderr_summary"
      artifact=$(artifact_path "$ARTIFACTS_DIR" "$ROOT" "$file")
      mkdir -p "$(dirname "$artifact")"
      cp "$stdout_file" "$artifact"
      solution_path=$(sanitize_field "$(relative_display "$artifact" "$CWD")")
    else
      status="error"
      if [[ -z "$stderr_summary" ]]; then
        detail="solver exited with status $returncode"
      else
        detail="$stderr_summary"
      fi
    fi
  fi

  row=$(printf '%s\t%s\t%s\t%s\t0\t%s\t%s\t%s\t%s\t\t%s\t\t%s\t%s\t%s\t%s' \
    "$group_field" "$path_field" "$status" "$duration_ms" "$duration_ms" "$sygus_bytes" "$sygus_lines" "$stdout_size" \
    "${solver_exit_code:-}" "$(sanitize_field "$SOLVER")" "$solver_args_joined" "$solution_path" "$(sanitize_field "$detail")")
  rm -f "$meta_file" "$stdout_file" "$stderr_file"

  if ((manage_results_table)); then
    with_lock "$RESULTS_LOCK_PATH" update_results_row "$RESULTS_PATH" "$row"
  else
    with_lock "$RESULTS_LOCK_PATH" append_results_row "$RESULTS_PATH" "$row"
  fi
  printf '%s\n' "$row"
  completed_now=$(( ${completed_now:-0} + 1 ))
  append_log "$LOG_PATH" "done path=$relative status=$(printf '%s' "$row" | cut -f3) completed=$completed_now"
  clear_progress_marker "$PROGRESS_PATH"
done

clear_progress_marker "$PROGRESS_PATH"
append_log "$LOG_PATH" "done completed=${completed_now:-0}"
