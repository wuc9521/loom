#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/setup-external-tools.sh [options]

Options:
  --root PATH                 Checkout root for external tool repos.
                              Default: $HOME/external-tools
  --folseparators-artifact PATH
                              Optional path to 560.tgz. If provided, load it into
                              Docker and tag it as folseparators-artifact:560.
  --help                      Show this help.
EOF
}

ROOT="$HOME/external-tools"
FOLSEP_ARTIFACT=""

while (($#)); do
  case "$1" in
    --root)
      ROOT=$2
      shift 2
      ;;
    --folseparators-artifact)
      FOLSEP_ARTIFACT=$2
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "setup-external-tools: unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$ROOT"

clone_or_update() {
  local url=$1
  local dir=$2
  if [[ -d "$dir/.git" ]]; then
    git -C "$dir" fetch --all --tags
    git -C "$dir" pull --ff-only
  else
    git clone "$url" "$dir"
  fi
}

clone_or_update https://github.com/jrkoenig/folseparators.git "$ROOT/folseparators"
clone_or_update https://github.com/wilcoxjay/mypyvy.git "$ROOT/mypyvy"
clone_or_update https://github.com/kupl/AlphaRegexPublic.git "$ROOT/AlphaRegexPublic"
clone_or_update https://github.com/gergia/samples2LTL.git "$ROOT/samples2LTL"
clone_or_update https://github.com/SySLite/SySLite.git "$ROOT/SySLite"
clone_or_update https://gitlab.lre.epita.fr/adrien/learnctl.git "$ROOT/learnctl"

if [[ -n "$FOLSEP_ARTIFACT" ]]; then
  command -v docker >/dev/null 2>&1 || {
    echo "setup-external-tools: docker is required to load the folseparators artifact" >&2
    exit 1
  }
  docker load -i "$FOLSEP_ARTIFACT" >/dev/null
  if docker image inspect 560:latest >/dev/null 2>&1; then
    docker tag 560:latest folseparators-artifact:560
  fi
fi

if command -v cvc5 >/dev/null 2>&1; then
  echo "cvc5: $(cvc5 --version | head -n 1)"
else
  echo "cvc5 is not installed; install it separately or make it available on PATH." >&2
fi

echo "external tools root: $ROOT"
