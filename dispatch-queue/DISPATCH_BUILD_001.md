# DISPATCH_BUILD_001 — Bootstrap 32-bit Build System

## Header
**Domain:** BUILD | **Agent:** Agent-01 | **Status:** PENDING
**Priority:** CRITICAL | **Created:** PROJECT_START
**Branch:** agent/build-bootstrap

## Prerequisites
- None — start immediately.

## Context Snapshot
None — first task.

## Objective
Set up the `i686-unknown-none` Rust cross-compile environment so `make build` produces a 32-bit kernel ELF and `make qemu` launches `qemu-system-i386`. The kernel does nothing yet — just compiles and boots without crashing. Python venv works.

## Acceptance Criteria
- [ ] `rust-toolchain.toml` pins nightly, `i686-unknown-none` target
- [ ] `.cargo/config.toml` sets 32-bit target, correct rustflags
- [ ] `kernel/Cargo.toml` is `no_std`, `panic = "abort"`
- [ ] `kernel/linker.ld` maps kernel to `0x00100000` (32-bit flat, no higher-half)
- [ ] `kernel/src/main.rs` has `#![no_std] #![no_main]` and compiles
- [ ] `make build` succeeds, produces `target/i686-unknown-none/release/kernel`
- [ ] `make venv` creates `.venv/` with requirements installed
- [ ] `.gitignore` includes `target/`, `.venv/`, `*.img`
- [ ] Full directory structure from `MASTER.md` Section 7 created (`.gitkeep` in empty dirs)
- [ ] **VERIFY: ELF is 32-bit**: `file target/.../kernel` must say "ELF 32-bit"

## Files To Touch
- CREATE: `rust-toolchain.toml`
- CREATE: `.cargo/config.toml`
- CREATE: `kernel/Cargo.toml`
- CREATE: `kernel/build.rs` — linker script argument
- CREATE: `kernel/linker.ld` — per BUILD.md Section 3
- CREATE: `kernel/src/main.rs` — minimal stub
- CREATE: `Makefile` — per BUILD.md Section 4
- CREATE: `tools/setup_venv.sh`
- CREATE: `tools/requirements.txt`
- CREATE: `.gitignore`
- CREATE: all empty dirs with `.gitkeep`

## API Spec
```
# Verify output is 32-bit:
file target/i686-unknown-none/release/kernel
# Must output: ELF 32-bit LSB executable, Intel 80386
```

## Pitfalls
- See MISTAKE-001: target must be `i686-unknown-none`, NOT `x86_64-unknown-none`
- See BUILD.md MISTAKE section: `panic = "unwind"` will break; use `panic = "abort"`
- Linker script base address is `0x00100000` (1MB), NOT `0xFFFFFFFF80000000`
- QEMU binary is `qemu-system-i386`, not `qemu-system-x86_64`

## Test
```bash
make build
file target/i686-unknown-none/release/kernel
# expected: ELF 32-bit LSB
make venv
```

## On Completion
1. Check all criteria, especially the ELF 32-bit check
2. Append ITER-001 to `docs/ITERATIONS.md`
3. Status: COMPLETE
4. Delete lock
5. PR: `build: bootstrap i686 build system — DISPATCH_BUILD_001`
6. Generate `DISPATCH_HEAD_001`

## Related Cards
- DISPATCH_HEAD_001 (blocked on this)
