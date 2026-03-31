#!/usr/bin/env sh
set -eu

CLS_SEQ="$(printf '\033[2J\033[H')"
OUT="$(printf 'cls\n' | ./build/turingos 2>&1)"

case "$OUT" in *"$CLS_SEQ"*) ;; *)
    echo "FAIL: expected ANSI clear (ESC [2J ESC [H) in cls output" >&2
    exit 1
    ;;
esac
case "$OUT" in *"A> "*) ;; *)
    echo "FAIL: expected shell prompt in cls test output" >&2
    exit 1
    ;;
esac

echo "PASS: test_cls"
