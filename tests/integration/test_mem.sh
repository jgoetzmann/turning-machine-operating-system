#!/usr/bin/env sh
set -eu

OUT="$(printf 'mem\n' | ./build/turingos 2>&1)"

case "$OUT" in *"0000-00FF BIOS"*) ;; *)
    echo "FAIL: expected mem map output" >&2
    exit 1
    ;;
esac
case "$OUT" in *"FF00-FFFF META"*) ;; *)
    echo "FAIL: expected mem map tail" >&2
    exit 1
    ;;
esac

echo "PASS: test_mem"
