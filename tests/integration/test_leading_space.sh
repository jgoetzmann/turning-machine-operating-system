#!/usr/bin/env sh
set -eu

OUT="$(printf '   dir\n' | ./build/turingos 2>&1)"

case "$OUT" in *"(empty)"*) ;; *)
    echo "FAIL: expected dir listing with leading spaces before command" >&2
    exit 1
    ;;
esac
case "$OUT" in *"A> "*) ;; *)
    echo "FAIL: expected shell prompt" >&2
    exit 1
    ;;
esac

echo "PASS: test_leading_space"
