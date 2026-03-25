#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_PATH="$REPO_ROOT/.cursor/debug-c5b577.log"

# #region agent log
_tmos_log() {
  local hypothesisId="$1"; shift
  local message="$1"; shift
  local data="${1:-{}}"
  TMOS_LOG_PATH="$LOG_PATH" TMOS_HYP="$hypothesisId" TMOS_MSG="$message" TMOS_DATA="$data" \
    python3 - <<'PY' 2>/dev/null || true
import json, os, time, pathlib
path = pathlib.Path(os.environ["TMOS_LOG_PATH"])
path.parent.mkdir(parents=True, exist_ok=True)
payload = {
  "sessionId":"c5b577",
  "runId":"pre-fix",
  "hypothesisId":os.environ.get("TMOS_HYP",""),
  "location":"scripts/smoke.sh",
  "message":os.environ.get("TMOS_MSG",""),
  "data":json.loads(os.environ.get("TMOS_DATA","{}") or "{}"),
  "timestamp":int(time.time()*1000),
}
path.open("a", encoding="utf-8").write(json.dumps(payload)+"\n")
PY
}
# #endregion agent log

echo "=== TMOS smoke ==="

if [[ ! -x "$REPO_ROOT/.venv/bin/python3" ]]; then
  _tmos_log "H2" ".venv missing" "{\"expected\":\"$REPO_ROOT/.venv/bin/python3\"}"
  echo "ERROR: .venv not found. Run: bash scripts/setup_venv.sh"
  exit 1
fi

source "$REPO_ROOT/.venv/bin/activate"

_tmos_log "H2" ".venv present and activated" "{\"venv\":\"$VIRTUAL_ENV\"}"

need_cmd() {
  local c="$1"
  if ! command -v "$c" >/dev/null 2>&1; then
    _tmos_log "H2" "missing command" "{\"cmd\":\"$c\"}"
    echo "ERROR: missing required command: $c"
    return 1
  fi
  _tmos_log "H2" "found command" "{\"cmd\":\"$c\",\"path\":\"$(command -v "$c")\"}"
  return 0
}

# Local smoke requires these on the host (Docker smoke avoids this).
need_cmd cargo || { echo "Hint: install Rust (rustup) so 'cargo' is available."; exit 1; }
need_cmd qemu-system-i386 || { echo "Hint: install QEMU i386 (e.g. 'brew install qemu')."; exit 1; }
need_cmd nasm || { echo "Hint: install NASM (e.g. 'brew install nasm')."; exit 1; }
need_cmd file || true

echo ""
echo "[1/4] Build kernel"
make -C "$REPO_ROOT" build

echo ""
echo "[2/4] Verify 32-bit"
make -C "$REPO_ROOT" verify-32bit

echo ""
echo "[3/4] Run a couple QEMU integration tests"
make -C "$REPO_ROOT" qemu-test TEST=head_boot_basic TIMEOUT=30
make -C "$REPO_ROOT" qemu-test TEST=display_write TIMEOUT=30

echo ""
echo "[4/4] Compile freestanding C smoke programs (host-side)"
"$REPO_ROOT/.venv/bin/python3" "$REPO_ROOT/tools/c_smoke_tests.py"

echo ""
echo "✅ Smoke complete"

