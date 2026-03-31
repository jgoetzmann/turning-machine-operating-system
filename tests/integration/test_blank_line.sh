#!/usr/bin/env sh
set -eu

OUT="$(printf '\n' | ./build/turingos 2>&1)"

case "$OUT" in *"?"*)
    echo "FAIL: empty line should not print ?" >&2
    exit 1
    ;;
esac
case "$OUT" in *"A> "*) ;; *)
    echo "FAIL: expected shell prompt in blank-line test" >&2
    exit 1
    ;;
esac

echo "PASS: test_blank_line"
