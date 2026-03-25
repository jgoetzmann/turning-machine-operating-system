# SHELL.md — Userspace & Shell Agent
> **Domain ID:** `SHELL` | **Agent Slot:** Agent-10
> **TM Role:** A program is a sequence of symbols on the process tape. The shell reads symbols from the keyboard tape and dispatches computations.
> **Only start after PROC + TRANS + STORE are stable.**

---

## 1. Domain Purpose

- **libc stub** (`userspace/libc/`): minimal C library, syscall wrappers via `int 0x80`
- **Shell** (`userspace/shell/`): reads keyboard input, parses commands, forks/execs
- **Init** (`userspace/init/`): PID 1, spawns shell, reaps zombies

Userspace is written in **C** (not Rust) to test the syscall interface.

---

## 2. Syscall ABI (int 0x80, 32-bit)

```c
// userspace/libc/syscall.asm (NASM)
_syscall3:
    push ebp
    mov  ebp, esp
    mov  eax, [ebp+8]   ; syscall number
    mov  ebx, [ebp+12]  ; arg1
    mov  ecx, [ebp+16]  ; arg2
    mov  edx, [ebp+20]  ; arg3
    int  0x80
    pop  ebp
    ret
```

---

## 3. Shell Features (Phase 1 Only)

- Run a single command: `echo hello`
- `exit` built-in
- `help` built-in
- Read line from keyboard, print result to VGA/serial

No pipes. No redirection. No background processes. That's Phase 2.

---

## 4. Critical Constraints

- **Only `int 0x80`** — not `syscall`. We are 32-bit protected mode.
- **Static linking only** — no dynamic linker.
- **No C++ in userspace** — plain C only. No exceptions, no RTTI.
- **Build with `i686-elf-gcc`** cross-compiler, not the host GCC.

---
