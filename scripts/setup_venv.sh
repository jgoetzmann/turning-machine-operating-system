#!/usr/bin/env bash
set -euo pipefail

# Convenience wrapper around tools/setup_venv.sh
# Creates .venv/ and installs tools/requirements.txt

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== TMOS: setting up Python venv (.venv) ==="
bash "$REPO_ROOT/tools/setup_venv.sh"

echo ""
echo "Next:"
echo "  source .venv/bin/activate"
echo "  make test-list"
echo "  make qemu-test TEST=head_boot_basic"
