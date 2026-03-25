# DISPATCH_DISPLAY_001 — VGA Text Mode Writer

## Header
**Domain:** DISPLAY | **Agent:** Agent-07 | **Status:** PENDING
**Priority:** HIGH | **Created:** PROJECT_START
**Branch:** agent/display-vga-writer

## Prerequisites
- [ ] DISPATCH_HEAD_001 COMPLETE — GDT and memory accessible
- [ ] DISPATCH_TAPE_001 COMPLETE — need `tape::regions::VGA_BASE` constant

## Context Snapshot
None — load `docs/agents/DISPLAY.md` only.

## Objective
Implement a VGA text mode writer that writes to the physical buffer at `tape::regions::VGA_BASE`. Provide `kprint!` / `kprintln!` macros as the kernel's standard output. After this task, all domains can use `kprintln!` to write colored text to the screen.

## Acceptance Criteria
- [ ] `VgaWriter` writes to `tape::regions::VGA_BASE` via `tape::tape_write` (volatile)
- [ ] 80 columns × 25 rows, 2 bytes per cell (char + color)
- [ ] `write_char()` handles `\n` (newline), `\r`, `\t` (4 spaces), `\x08` (backspace)
- [ ] Scrolling: when row 25 is reached, shift rows 1–24 to 0–23, clear row 24
- [ ] `kprint!` and `kprintln!` macros work (implement `core::fmt::Write`)
- [ ] `set_panic_colors()` sets red background + white foreground
- [ ] Global `WRITER: spin::Mutex<VgaWriter>` initialized with light-grey on black
- [ ] `make qemu-test TEST=display_write` passes
- [ ] VGA buffer content verified in test (read back via `tape::tape_read`)
- [ ] **Address is `tape::regions::VGA_BASE` (`u32`), not hardcoded `0xB8000`**

## Files To Touch
- CREATE: `kernel/src/display/vga.rs` — VgaCell, Color, ColorCode, low-level write
- CREATE: `kernel/src/display/mod.rs` — VgaWriter, WRITER, kprint!, kprintln!
- CREATE: `tests/integration/display_write.rs`

## API Spec
```rust
pub static WRITER: spin::Mutex<VgaWriter>;
#[macro_export] macro_rules! kprint { ($($arg:tt)*) => { ... } }
#[macro_export] macro_rules! kprintln { ($($arg:tt)*) => { ... } }
```

## Pitfalls
- See MISTAKE-007: VGA writes MUST be volatile. Use `tape::tape_write`, not `*ptr = val`.
- See DISPLAY.md: panic handler needs a fallback write path that bypasses the Mutex (call `WRITER.force_unlock()` first).
- VGA cell = `u16`: high byte = color attribute, low byte = ASCII character. Write as one `u16`, not two `u8`s.
- Scrolling: copy rows 1–24 to rows 0–23, then `memset` row 24 to spaces. All via `tape_write` in a loop.

## Test
```bash
make qemu-test TEST=display_write TIMEOUT=10
```

---
