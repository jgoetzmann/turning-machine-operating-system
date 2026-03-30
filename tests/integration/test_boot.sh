#!/usr/bin/env sh
set -eu

OUT="$(./build/turingos)"

case "$OUT" in
  *"TuringOS stub boot complete (state=5)"*) ;;
  *)
    echo "FAIL: boot output mismatch" >&2
    exit 1
    ;;
esac

echo "PASS: test_boot"
