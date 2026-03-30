#!/usr/bin/env sh
set -eu

OUT="$(printf 'cc nosuch.c\n' | ./build/turingos 2>&1)"

case "$OUT" in *"?"*) ;;
  *)
    echo "FAIL: expected error marker for missing source (cc)" >&2
    exit 1
    ;;
esac

echo "PASS: test_cc"
