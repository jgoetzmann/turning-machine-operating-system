#!/usr/bin/env sh
set -eu

OUT="$(printf 'type nosuch.com\n' | ./build/turingos 2>&1)"

case "$OUT" in *"?"*) ;;
  *)
    echo "FAIL: expected error marker for missing file" >&2
    exit 1
    ;;
esac

echo "PASS: test_type"
