#!/usr/bin/env sh
set -eu

OUT="$(printf 'help\n' | ./build/turingos 2>&1)"

case "$OUT" in *"dir type halt help mem cls run del cc"*) ;; *)
    echo "FAIL: expected help text" >&2
    exit 1
    ;;
esac

echo "PASS: test_help"
