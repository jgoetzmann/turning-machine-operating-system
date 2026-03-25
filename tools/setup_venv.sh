#!/usr/bin/env bash
# tools/setup_venv.sh — Python venv setup for TM OS tooling
set -e
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="$REPO_ROOT/.venv"
echo "=== TM OS — Python Venv Setup ==="
python3 -m venv "$VENV"
source "$VENV/bin/activate"
pip install --quiet --upgrade pip
pip install --quiet -r "$REPO_ROOT/tools/requirements.txt"
echo "✅ Venv ready at $VENV"
echo "Activate: source .venv/bin/activate"
