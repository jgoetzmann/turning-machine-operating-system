#!/usr/bin/env sh
set -eu

CC_CASE=echo ./build/tests/compiler/test_expected_outputs >/dev/null
echo "PASS: test_echo"
