#!/usr/bin/env bash
set -euo pipefail

# Build a self-contained dev image and run the smoke suite inside it.
# This avoids installing Rust/QEMU/NASM/Python deps globally on macOS.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="${TMOS_DOCKER_IMAGE:-tmos-dev}"
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
  "location":"scripts/docker_smoke.sh",
  "message":os.environ.get("TMOS_MSG",""),
  "data":json.loads(os.environ.get("TMOS_DATA","{}") or "{}"),
  "timestamp":int(time.time()*1000),
}
path.open("a", encoding="utf-8").write(json.dumps(payload)+"\n")
PY
}
# #endregion agent log

if ! command -v docker >/dev/null 2>&1; then
  _tmos_log "H1" "docker missing" "{\"which\":\"none\"}"
  echo "ERROR: docker not found. Install Docker Desktop first."
  exit 1
fi

echo "=== TMOS: docker smoke ==="
echo "Image: $IMAGE_NAME"

_tmos_log "H1" "docker present" "{\"which\":\"$(command -v docker)\"}"

# Check daemon connectivity early with a clearer hint.
if ! docker info >/dev/null 2>&1; then
  _tmos_log "H1" "docker daemon unreachable" "{}"
  echo "ERROR: cannot connect to Docker daemon."
  echo "If you're using Docker Desktop: start it and retry."
  echo "If you're using Colima: start it (e.g. 'colima start') and retry."
  exit 1
fi

_tmos_log "H1" "docker daemon reachable" "{}"

docker build -f "$REPO_ROOT/Dockerfile.dev" -t "$IMAGE_NAME" "$REPO_ROOT"

docker run --rm -t \
  -v "$REPO_ROOT":/work \
  -w /work \
  "$IMAGE_NAME" \
  bash -lc "scripts/setup_venv.sh && source .venv/bin/activate && scripts/smoke.sh"

