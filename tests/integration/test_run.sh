#!/usr/bin/env sh
set -eu

OUT="$(printf 'run nosuch.com\n' | ./build/turingos 2>&1)"

case "$OUT" in *"?"*) ;;
  *)
    echo "FAIL: expected error marker for missing file (run)" >&2
    exit 1
    ;;
esac

echo "PASS: test_run"
