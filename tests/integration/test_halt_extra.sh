#!/usr/bin/env sh
set -eu

OUT="$(printf 'halt extra\n' | ./build/turingos 2>&1)"

case "$OUT" in *"?"*) ;; *)
    echo "FAIL: expected ? when halt has extra arguments" >&2
    exit 1
    ;;
esac
case "$OUT" in *"HALT"*)
    echo "FAIL: halt with args should not print HALT" >&2
    exit 1
    ;;
esac

echo "PASS: test_halt_extra"
