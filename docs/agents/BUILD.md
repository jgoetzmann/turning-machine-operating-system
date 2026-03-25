# BUILD.md — Toolchain Agent
> **Domain ID:** `BUILD` | **Agent Slot:** Agent-01**
> **This is the FIRST domain. Nothing works until this is done.**
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

Set up the 32-bit Rust cross-compile environment and build system.

---

## 2. Key Configuration

### `rust-toolchain.toml`
```toml
[toolchain]
channel = "nightly-2025-01-01"
components = ["rust-src", "rustfmt", "clippy"]
targets = ["i686-unknown-none"]
```

### `.cargo/config.toml`
```toml
[build]
target = "i686-unknown-none"

[target.i686-unknown-none]
rustflags = [
    "-C", "link-arg=-Tkernel/linker.ld",
    "-C", "relocation-model=static",
    "-C", "code-model=kernel",
]

[unstable]
build-std = ["core", "alloc", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]
```

### `kernel/Cargo.toml`
```toml
[package]
name = "kernel"
version = "0.1.0"
edition = "2021"

[dependencies]
spin = "0.9"
bitflags = "2"

[profile.dev]
panic = "abort"
opt-level = 1

[profile.release]
panic = "abort"
opt-level = 2
```

### `kernel/src/main.rs` required attributes
```rust
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
```

---

## 3. Linker Script (`kernel/linker.ld`) — 32-bit Flat

```ld
ENTRY(_start)

SECTIONS {
    . = 0x00100000;        /* Kernel loads at 1MB physical */

    _kernel_start = .;

    .boot : {
        KEEP(*(.multiboot2))
    }

    .text   : { *(.text .text.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data   : { *(.data .data.*) }

    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        _bss_end = .;
    }

    _kernel_end = .;

    /DISCARD/ : { *(.comment) *(.eh_frame) }
}
```

---

## 4. Makefile Targets

```makefile
build:       cargo build --release -p kernel
build-debug: cargo build -p kernel
qemu:        qemu-system-i386 -kernel ... (see TEST.md for full flags)
qemu-test:   python3 tools/test_runner.py --test $(TEST)
clean:       cargo clean
venv:        bash tools/setup_venv.sh
```

---

## 5. Known Mistakes to Avoid

- **MISTAKE: Using `i686-linux-gnu` instead of `i686-unknown-none`** — the `-linux-gnu` target links against Linux libc. We need the bare-metal `unknown-none` target.
- **MISTAKE: `panic = "unwind"` in kernel** — unwinding requires std. Always `panic = "abort"`.
- **MISTAKE: Not discarding `.eh_frame`** — exception handling frames bloat the binary and GRUB may reject the ELF.
- **MISTAKE: `qemu-system-x86_64` for a 32-bit kernel** — use `qemu-system-i386`.
