# DISPATCH_SHELL_001 — Userspace Init + Shell

## Header
**Domain:** SHELL | **Agent:** Agent-10 | **Status:** PENDING
**Priority:** NORMAL | **Created:** PROJECT_START
**Branch:** agent/shell-userspace

## Prerequisites
- [ ] DISPATCH_PROC_001 COMPLETE — process creation + context switch
- [ ] DISPATCH_TRANS_001 COMPLETE — int 0x80 syscall working
- [ ] DISPATCH_STORE_001 COMPLETE — filesystem (to load shell ELF from disk)
- [ ] DISPATCH_DISPLAY_001 COMPLETE — VGA for shell output

## Context Snapshot
None — load `docs/agents/SHELL.md` only.

## Objective
Build a minimal userspace: a libc stub (`int 0x80` wrappers) and a shell that reads keyboard input, runs built-in commands (`echo`, `help`, `exit`), and displays output on VGA. This validates the entire TM stack end-to-end.

## Acceptance Criteria
- [ ] `userspace/libc/syscall.asm`: `sys_write(fd, buf, len)`, `sys_exit(code)`, `sys_getpid()`, `sys_yield()` via `int 0x80`
- [ ] Shell compiled with `i686-elf-gcc` (cross-compiler), statically linked
- [ ] Shell ELF stored on FAT12 disk image as `SHELL.ELF`
- [ ] Kernel loads `SHELL.ELF` from disk into `PROC_TAPE_BASE`, transfers head to entry point
- [ ] Shell displays prompt `TM> ` on VGA
- [ ] `echo hello` command works: prints `hello` to VGA
- [ ] `help` command lists available commands
- [ ] `exit` command calls `sys_exit(0)`, machine transitions to `Halt`
- [ ] `make qemu-test TEST=shell_echo` passes
- [ ] `make qemu-test TEST=shell_exit` passes (machine halts cleanly)
- [ ] **All syscalls use `int 0x80`, not `syscall` instruction**

## Files To Touch
- CREATE: `userspace/libc/syscall.asm` — int 0x80 wrappers (NASM 32-bit)
- CREATE: `userspace/libc/string.c` — strlen, memcpy, memset
- CREATE: `userspace/shell/main.c` — shell loop
- CREATE: `userspace/shell/builtin.c` — echo, help, exit
- MODIFY: `Makefile` — add `build-userspace` target, add `SHELL.ELF` to fat12.img
- MODIFY: `kernel/src/store/mod.rs` — add `load_elf(name)` function (coordinate with STORE agent)
- CREATE: `tests/integration/shell_echo.rs`
- CREATE: `tests/integration/shell_exit.rs`

## API Spec (userspace libc)
```c
/* userspace/libc/syscall.h */
long sys_write(int fd, const void *buf, long len);
void sys_exit(int code) __attribute__((noreturn));
long sys_getpid(void);
void sys_yield(void);
```

## Pitfalls
- See SHELL.md: `int 0x80`, NOT `syscall`. This is 32-bit i386 ABI.
- Cross-compiler: `i686-elf-gcc -m32 -nostdlib -nostdinc -static`. Host GCC will produce 64-bit code.
- ELF entry point: read `e_entry` from ELF header. Do NOT assume `0x400000`.
- Stack for shell: set `esp` to user stack top before transferring head.
- `sys_write(fd=1, ...)` must route to VGA output in the kernel's `SYS_WRITE` handler.

## Test
```bash
make qemu-test TEST=shell_echo TIMEOUT=20
make qemu-test TEST=shell_exit TIMEOUT=20
```

## On Completion
This is the Phase 1 completion milestone. The TM OS is running end-to-end.
Generate: Review MASTER.md and create Phase 2 planning dispatch cards.
