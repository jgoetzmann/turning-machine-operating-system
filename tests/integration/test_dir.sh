#!/usr/bin/env sh
set -eu

OUT="$(printf 'dir\n' | ./build/turingos 2>&1)"

case "$OUT" in
  *"A> "*) ;;
  *)
    echo "FAIL: expected shell prompt in dir test output" >&2
    exit 1
    ;;
esac
case "$OUT" in
  *"(empty)"*)
    ;;
  *)
    echo "FAIL: expected empty directory listing after dir (run make disk)" >&2
    exit 1
    ;;
esac

echo "PASS: test_dir"
