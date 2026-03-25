// kernel/src/display/vga.rs
// VGA text mode buffer — 80×25 chars, 2 bytes per cell.
// Agent: DISPLAY (Agent-07)
//
// The VGA buffer is a tape region at tape::regions::VGA_BASE (0x000B8000).
// All writes MUST be volatile (see MISTAKE-007 in docs/MISTAKES.md).
// All writes go through tape::tape_write(), NOT raw pointer writes.

use crate::tape::regions::{VGA_BASE, VGA_COLS, VGA_ROWS};

/// VGA 16-color palette.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black=0, Blue=1, Green=2, Cyan=3, Red=4, Magenta=5,
    Brown=6, LightGrey=7, DarkGrey=8, LightBlue=9,
    LightGreen=10, LightCyan=11, LightRed=12, Pink=13,
    Yellow=14, White=15,
}

/// Combined foreground + background color (one byte).
/// High nibble = background (3 bits used), low nibble = foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
    pub const fn panic_colors() -> Self {
        ColorCode::new(Color::White, Color::Red)
    }
}

/// A single VGA cell: character byte + color byte, packed as u16.
/// Written to the tape as one atomic 16-bit write.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VgaCell(u16);

impl VgaCell {
    pub const fn new(ascii: u8, color: ColorCode) -> Self {
        VgaCell(ascii as u16 | (color.0 as u16) << 8)
    }
    pub const fn blank(color: ColorCode) -> Self {
        VgaCell::new(b' ', color)
    }
}

/// VGA text mode writer.
pub struct VgaWriter {
    col:   u32,
    row:   u32,
    color: ColorCode,
}

impl VgaWriter {
    pub const fn new(color: ColorCode) -> Self {
        VgaWriter { col: 0, row: 0, color }
    }

    /// Write a single ASCII character to the current cursor position.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => { self.col = 0; }
            0x08  => { // backspace
                if self.col > 0 { self.col -= 1; }
                self.write_cell_at(self.col, self.row, VgaCell::blank(self.color));
            }
            byte  => {
                self.write_cell_at(self.col, self.row, VgaCell::new(byte, self.color));
                self.col += 1;
                if self.col >= VGA_COLS {
                    self.newline();
                }
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= VGA_ROWS {
            self.scroll();
            self.row = VGA_ROWS - 1;
        }
    }

    /// Scroll the screen up by one row.
    fn scroll(&mut self) {
        // Copy rows 1..VGA_ROWS to rows 0..VGA_ROWS-1
        for row in 0..(VGA_ROWS - 1) {
            for col in 0..VGA_COLS {
                let src_offset = ((row + 1) * VGA_COLS + col) * 2;
                let dst_offset = (row * VGA_COLS + col) * 2;
                // SAFETY: Both offsets are within VGA buffer bounds.
                let cell: u16 = unsafe { crate::tape::tape_read(VGA_BASE + src_offset) };
                unsafe { crate::tape::tape_write(VGA_BASE + dst_offset, cell) };
            }
        }
        // Clear the last row
        for col in 0..VGA_COLS {
            let offset = ((VGA_ROWS - 1) * VGA_COLS + col) * 2;
            unsafe { crate::tape::tape_write(VGA_BASE + offset, VgaCell::blank(self.color).0) };
        }
    }

    fn write_cell_at(&self, col: u32, row: u32, cell: VgaCell) {
        let offset = (row * VGA_COLS + col) * 2;
        // SAFETY: col < VGA_COLS and row < VGA_ROWS, so offset is within VGA_BASE..VGA_BASE+VGA_SIZE.
        // VGA_BASE is the hardware VGA tape region. Must be volatile (see MISTAKE-007).
        unsafe { crate::tape::tape_write(VGA_BASE + offset, cell.0) };
    }

    /// Clear the entire screen.
    pub fn clear(&mut self) {
        for row in 0..VGA_ROWS {
            for col in 0..VGA_COLS {
                self.write_cell_at(col, row, VgaCell::blank(self.color));
            }
        }
        self.col = 0;
        self.row = 0;
    }

    /// Set the color for subsequent writes.
    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    /// Switch to panic colors (white on red). Used by panic handler.
    pub fn set_panic_colors(&mut self) {
        self.color = ColorCode::panic_colors();
    }
}

impl core::fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII range + basic control chars
                0x20..=0x7E | b'\n' | b'\r' | 0x08 => self.write_byte(byte),
                _ => self.write_byte(b'?'),  // replace unprintable with ?
            }
        }
        Ok(())
    }
}
