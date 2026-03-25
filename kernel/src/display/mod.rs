// kernel/src/display/mod.rs
// DISPLAY domain — VGA text mode writer
// Agent: Agent-07
// Read docs/agents/DISPLAY.md before modifying.

pub mod vga;

use vga::{VgaWriter, ColorCode, Color};
use spin::Mutex;

/// Global VGA writer — use via kprint!/kprintln! macros.
pub static WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new(
    ColorCode::new(Color::LightGrey, Color::Black)
));

/// Write a string to VGA directly (no format, no allocation).
/// Used by sys_write and panic handler.
pub fn kprint_str(s: &str) {
    use core::fmt::Write;
    let mut w = WRITER.lock();
    let _ = w.write_str(s);
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut w = $crate::display::WRITER.lock();
        let _ = write!(w, $($arg)*);
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut w = $crate::display::WRITER.lock();
        let _ = writeln!(w, $($arg)*);
    }};
}
