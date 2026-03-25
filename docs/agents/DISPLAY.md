# DISPLAY.md — VGA Text Output Agent
> **Domain ID:** `DISPLAY` | **Agent Slot:** Agent-07
> **TM Role:** Write symbols to the screen tape region. The display is just another tape region.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

Write characters to the VGA text buffer at `tape::regions::VGA_BASE` (`0x000B8000`).

- VGA text mode: 80 columns × 25 rows, 2 bytes per cell (char + attribute)
- Cursor tracking: software cursor (write to CRT port for hardware cursor)
- Color attributes: 16 foreground + 8 background colors
- Scrolling: shift rows up when reaching row 25
- `kprint!` / `kprintln!` macros for kernel output
- Panic screen: red background, white text, freeze

---

## 2. File Ownership

```
kernel/src/display/
├── mod.rs          # VgaWriter, WRITER global, kprint!/kprintln! macros
└── vga.rs          # Low-level cell write, color types, cursor
```

---

## 3. API Contracts

```rust
// display/vga.rs
#[derive(Copy, Clone)] #[repr(u8)]
pub enum Color { Black=0, Blue=1, Green=2, Cyan=3, Red=4, Magenta=5,
                 Brown=6, LightGrey=7, DarkGrey=8, LightBlue=9,
                 LightGreen=10, LightCyan=11, LightRed=12, Pink=13,
                 Yellow=14, White=15 }

#[derive(Copy, Clone)]
pub struct ColorCode(u8);  // upper nibble = bg, lower = fg

// A single VGA cell: character + color
#[repr(C)]
pub struct VgaCell { pub char: u8, pub color: ColorCode }

// display/mod.rs
pub struct VgaWriter { col: usize, row: usize, color: ColorCode }

impl VgaWriter {
    pub fn write_char(&mut self, c: char);
    pub fn write_str(&mut self, s: &str);
    pub fn clear(&mut self);
    pub fn set_color(&mut self, color: ColorCode);
    pub fn set_panic_colors(&mut self);  // red bg, white fg
}

impl core::fmt::Write for VgaWriter { ... }  // enables write!/writeln!

// Global writer — wrapped in spin::Mutex
pub static WRITER: spin::Mutex<VgaWriter> = ...;

#[macro_export] macro_rules! kprint { ... }
#[macro_export] macro_rules! kprintln { ... }
```

---

## 4. Critical Constraints

- **VGA buffer address comes from `tape::regions::VGA_BASE`** — not hardcoded `0xB8000`.
- **All writes are `tape::tape_write` (volatile)** — the VGA buffer is MMIO. Regular writes may be eliminated by optimizer.
- **`kprintln!` wraps the `spin::Mutex<VgaWriter>`** — safe to call from any domain.
- **Panic screen must work without acquiring any lock** — the panic handler runs after a state corruption. Have a `WRITER.force_unlock()` + write fallback path, or a dedicated bare-metal panic writer that bypasses the mutex.
- **No heap in DISPLAY** — `VgaWriter` is stack-allocated or static. No `Box`, no `Vec`.

---

## 5. Known Mistakes to Avoid

- **MISTAKE: VGA cell is `u16`, not `(u8, u8)` separately** — write both char and attribute in one `u16` write to avoid tearing.
- **MISTAKE: Scrolling by `memmove` on the VGA buffer without volatile** — the move must use volatile copies cell by cell, or the compiler will optimize it out.
---
