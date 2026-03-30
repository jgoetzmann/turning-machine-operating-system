#!/usr/bin/env sh
set -eu

CC_CASE=add ./build/tests/compiler/test_expected_outputs >/dev/null
echo "PASS: test_add"
