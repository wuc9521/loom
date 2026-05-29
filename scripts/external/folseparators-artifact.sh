#!/usr/bin/env bash
set -euo pipefail

IMAGE=${FOLSEPARATORS_IMAGE:-folseparators-artifact:560}
TIMEOUT_SECS=${FOLSEPARATORS_TIMEOUT_SECS:-600}
CASE_NAME=${DELIS_BENCH_INPUT_STEM:?missing DELIS_BENCH_INPUT_STEM}
ARTIFACT_DIR=${DELIS_BENCH_ARTIFACT_DIR:?missing DELIS_BENCH_ARTIFACT_DIR}
RESULT_JSON=${DELIS_BENCH_RESULT_JSON:?missing DELIS_BENCH_RESULT_JSON}
SOLUTION_PATH=${DELIS_BENCH_SOLUTION_PATH:-}
RAW_JSON="$ARTIFACT_DIR/raw.json"
RAW_LOG="$ARTIFACT_DIR/raw.log"

mkdir -p "$ARTIFACT_DIR"
command -v docker >/dev/null 2>&1 || {
  echo "folseparators wrapper requires docker" >&2
  exit 1
}
docker image inspect "$IMAGE" >/dev/null 2>&1 || {
  echo "folseparators docker image not found: $IMAGE" >&2
  exit 2
}

docker run --rm \
  -e CASE_NAME="$CASE_NAME" \
  -e TIMEOUT_SECS="$TIMEOUT_SECS" \
  -v "$ARTIFACT_DIR:/data" \
  "$IMAGE" \
  /bin/bash -lc 'cd /root/folseparators && python3.7 experiments/run_experiment.py -o /data/raw.json --timeout "$TIMEOUT_SECS" --cpus 1 --count 1 --single "$CASE_NAME" > /data/raw.log 2>&1'

python3 - "$RAW_JSON" "$RAW_LOG" "$RESULT_JSON" "$SOLUTION_PATH" <<'PY'
import json
import os
import sys

raw_json, raw_log, result_json, solution_path = sys.argv[1:5]
rows = json.load(open(raw_json, "r", encoding="utf-8"))
if len(rows) != 1:
    raise SystemExit(f"expected exactly one folseparators row, got {len(rows)}")
row = rows[0]
stats = row.get("stats", {})
if row.get("killed"):
    status = "timeout"
elif row.get("success"):
    status = "solved"
else:
    status = "failed"
formula = stats.get("formula") or ""
if solution_path and formula:
    with open(solution_path, "w", encoding="utf-8") as fh:
        fh.write(formula)
        fh.write("\n")
error = (stats.get("error") or "").strip()
log_summary = ""
if os.path.exists(raw_log):
    with open(raw_log, "r", encoding="utf-8", errors="replace") as fh:
        lines = [line.strip() for line in fh if line.strip()]
    if lines:
        log_summary = " | ".join(lines)
        if len(log_summary) > 240:
            log_summary = log_summary[:240] + "..."
detail_parts = [part for part in [error, log_summary] if part]
metadata = {
    "status": status,
    "total_ms": int(1000 * float(stats.get("total_time", 0.0))),
    "solve_ms": int(1000 * float(stats.get("total_time", 0.0))),
    "detail": " | ".join(detail_parts),
    "solution_path": solution_path or None,
    "solution_bytes": os.path.getsize(solution_path) if solution_path and os.path.exists(solution_path) else 0,
    "solver_exit_code": 0,
}
with open(result_json, "w", encoding="utf-8") as fh:
    json.dump(metadata, fh, indent=2, sort_keys=True)
PY
