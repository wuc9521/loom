#!/usr/bin/env bash
set -euo pipefail

HEADER=$'group\tcase\tstatus\ttotal_ms\tseparation_ms\tcounterexample_ms\tmatrix_ms\tmodel_count\tformula_quantifiers\ttimeout_s\tkilled\tsuccess\tformula\terror\tdetail'

usage() {
  cat <<'EOF'
Usage:
  bench-folseparators.sh --artifact PATH [options]

Options:
  --artifact PATH            Path to the uploaded 560.tgz Docker archive. Required.
  --name NAME                Benchmark group name. Default: fo-folseparators-artifact
  --results PATH             Explicit TSV output path. Default: bench-results/<slug>.tsv
  --results-dir PATH         Results root. Default: bench-results
  --image NAME               Docker image name to load/use. Default: 560
  --container NAME           Docker container name. Default: c560-bench
  --volume NAME              Docker volume name mounted at /data. Default: pldi20-560-bench
  --timeout-secs N           Per-case timeout passed to the artifact. Default: 600
  --cpus N                   Concurrent workers passed to the artifact. Default: 16
  --count N                  Repeat each case N times. Default: 1
  --single NAME              Run one case only, e.g. fast_paxos-c1
  --keep-container           Keep the container after the run.
  --restart                  Remove any existing container with the same name first.
  --help                     Show this help.
EOF
}

die() {
  echo "bench-folseparators: $*" >&2
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

ARTIFACT=""
NAME="fo-folseparators-artifact"
RESULTS_PATH=""
RESULTS_DIR=""
IMAGE="560"
CONTAINER="c560-bench"
VOLUME="pldi20-560-bench"
TIMEOUT_SECS=600
CPUS=16
COUNT=1
SINGLE=""
KEEP_CONTAINER=0
RESTART=0

while (($#)); do
  case "$1" in
    --artifact)
      (($# >= 2)) || die "--artifact requires a value"
      ARTIFACT=$2
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
    --image)
      (($# >= 2)) || die "--image requires a value"
      IMAGE=$2
      shift 2
      ;;
    --container)
      (($# >= 2)) || die "--container requires a value"
      CONTAINER=$2
      shift 2
      ;;
    --volume)
      (($# >= 2)) || die "--volume requires a value"
      VOLUME=$2
      shift 2
      ;;
    --timeout-secs)
      (($# >= 2)) || die "--timeout-secs requires a value"
      TIMEOUT_SECS=$2
      shift 2
      ;;
    --cpus)
      (($# >= 2)) || die "--cpus requires a value"
      CPUS=$2
      shift 2
      ;;
    --count)
      (($# >= 2)) || die "--count requires a value"
      COUNT=$2
      shift 2
      ;;
    --single)
      (($# >= 2)) || die "--single requires a value"
      SINGLE=$2
      shift 2
      ;;
    --keep-container)
      KEEP_CONTAINER=1
      shift
      ;;
    --restart)
      RESTART=1
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

[[ -n "$ARTIFACT" ]] || die "--artifact is required"
ARTIFACT=$(abspath "$ARTIFACT")
[[ -f "$ARTIFACT" ]] || die "artifact not found: $ARTIFACT"

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

JSON_OUT="$RESULTS_DIR/${SLUG}.json"
LOG_OUT="$RESULTS_DIR/${SLUG}.log"

command -v docker >/dev/null 2>&1 || die "docker is required"

if ((RESTART)); then
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  docker load -i "$ARTIFACT" >/dev/null
fi

if ! docker container inspect "$CONTAINER" >/dev/null 2>&1; then
  docker run --name "$CONTAINER" -dit -v "$VOLUME":/data "$IMAGE" /bin/bash >/dev/null
fi

run_cmd=(
  cd /root/folseparators
  "python3.7 experiments/run_experiment.py -o /data/artifact-learning.json --timeout $TIMEOUT_SECS --cpus $CPUS --count $COUNT"
)
if [[ -n "$SINGLE" ]]; then
  run_cmd[1]+=" --single $SINGLE"
fi
run_cmd[1]+=" > /data/artifact-learning.log 2>&1"

docker exec "$CONTAINER" /bin/bash -lc "${run_cmd[0]} && ${run_cmd[1]}"

docker cp "$CONTAINER":/data/artifact-learning.json "$JSON_OUT"
docker cp "$CONTAINER":/data/artifact-learning.log "$LOG_OUT"

python3 - "$JSON_OUT" "$RESULTS_PATH" "$HEADER" "$NAME" <<'PY'
import json
import sys

json_path, out_path, header, group = sys.argv[1:]
data = json.load(open(json_path))

def clean(value):
    text = "" if value is None else str(value)
    return text.replace("\t", " ").replace("\n", " ")

with open(out_path, "w", encoding="utf-8") as out:
    out.write(header + "\n")
    for row in data:
        stats = row.get("stats", {})
        case = f"{row.get('base', '')}-{row.get('conjecture', '')}"
        if row.get("killed"):
            status = "timeout"
        elif row.get("success"):
            status = "solved"
        else:
            status = "failed"
        total_ms = int(1000 * float(stats.get("total_time", 0.0)))
        separation_ms = int(1000 * float(stats.get("separation_time", 0.0)))
        counterexample_ms = int(1000 * float(stats.get("counterexample_time", 0.0)))
        matrix_ms = int(1000 * float(stats.get("matrix_time", 0.0)))
        fields = [
            clean(group),
            clean(case),
            clean(status),
            clean(total_ms),
            clean(separation_ms),
            clean(counterexample_ms),
            clean(matrix_ms),
            clean(stats.get("model_count", "")),
            clean(stats.get("formula_quantifiers", "")),
            clean(row.get("timeout", "")),
            clean(row.get("killed", False)).lower(),
            clean(row.get("success", False)).lower(),
            clean(stats.get("formula", "")),
            clean(stats.get("error", "")),
            clean(""),
        ]
        out.write("\t".join(fields) + "\n")
PY

printf '%s\n' "$HEADER"
tail -n +2 "$RESULTS_PATH"

if (( ! KEEP_CONTAINER )); then
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
fi
