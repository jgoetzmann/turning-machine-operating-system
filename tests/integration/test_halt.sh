#!/usr/bin/env sh
set -eu

OUT="$(./build/turingos </dev/null)"

case "$OUT" in
  *"state=5"*) ;;
  *)
    echo "FAIL: halt state not observed" >&2
    exit 1
    ;;
esac

echo "PASS: test_halt"
