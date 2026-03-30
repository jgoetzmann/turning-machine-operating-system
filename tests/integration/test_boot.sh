#!/usr/bin/env sh
set -eu

OUT="$(./build/turingos </dev/null)"

case "$OUT" in
  *"A> "*)
    ;;
  *)
    echo "FAIL: expected shell prompt in boot output" >&2
    exit 1
    ;;
esac
case "$OUT" in
  *"TuringOS stub boot complete (state=5)"*) ;;
  *)
    echo "FAIL: expected halt state line in boot output" >&2
    exit 1
    ;;
esac

echo "PASS: test_boot"
