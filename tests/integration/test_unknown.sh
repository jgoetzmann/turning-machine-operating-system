#!/usr/bin/env sh
set -eu

OUT="$(printf 'xyzunknown\n' | ./build/turingos 2>&1)"

case "$OUT" in *"?"*) ;;
  *)
    echo "FAIL: expected ? for unknown command" >&2
    exit 1
    ;;
esac
case "$OUT" in *"A> "*) ;; *)
    echo "FAIL: expected shell prompt in unknown-command test" >&2
    exit 1
    ;;
esac

echo "PASS: test_unknown"
