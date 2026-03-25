// kernel/src/head/serial.rs
// COM1 UART (16550) — earliest debug output, available before any other subsystem.
// Agent: HEAD (Agent-02)
//
// Port layout (COM1 base = 0x3F8):
//   0x3F8 + 0 = Data register (read: receive, write: transmit)
//   0x3F8 + 1 = Interrupt Enable Register
//   0x3F8 + 2 = FIFO Control Register
//   0x3F8 + 3 = Line Control Register (DLAB bit)
//   0x3F8 + 4 = Modem Control Register
//   0x3F8 + 5 = Line Status Register (bit 5 = transmit empty)
//
// With DLAB=1:
//   0x3F8 + 0 = Divisor Latch Low  (115200 / divisor = baud rate)
//   0x3F8 + 1 = Divisor Latch High
//
// We use 115200 baud, 8 data bits, no parity, 1 stop bit (8N1).
// Divisor for 115200 baud = 1.

const COM1: u16 = 0x3F8;

/// Read a byte from an x86 I/O port.
///
/// # Safety
/// Caller must ensure the port is valid for reading.
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Write a byte to an x86 I/O port.
///
/// # Safety
/// Caller must ensure the port is valid for writing and the value is appropriate.
#[inline]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Initialize COM1 at 115200 baud, 8N1.
/// Must be called before any serial output.
pub fn serial_init() {
    unsafe {
        // Disable all interrupts
        outb(COM1 + 1, 0x00);

        // Enable DLAB (Divisor Latch Access Bit) to set baud rate
        outb(COM1 + 3, 0x80);

        // Set divisor = 1 → 115200 baud
        // Low byte of divisor
        outb(COM1 + 0, 0x01);
        // High byte of divisor
        outb(COM1 + 1, 0x00);

        // 8 bits, no parity, 1 stop bit (8N1) — clear DLAB
        outb(COM1 + 3, 0x03);

        // Enable and clear FIFO, with 14-byte threshold
        outb(COM1 + 2, 0xC7);

        // Enable DTR, RTS, OUT2 (required for interrupts, harmless without them)
        outb(COM1 + 4, 0x0B);
    }
}

/// Returns true when the transmit buffer is empty (ready to send).
#[inline]
fn is_transmit_empty() -> bool {
    // SAFETY: Reading Line Status Register is always safe for COM1
    unsafe { (inb(COM1 + 5) & 0x20) != 0 }
}

/// Write a single byte to COM1. Blocks until transmit buffer is ready.
pub fn serial_write_byte(byte: u8) {
    // Busy-wait until transmit buffer is empty
    while !is_transmit_empty() {}
    // SAFETY: Writing to COM1 data register is safe after serial_init()
    unsafe { outb(COM1, byte) }
}

/// Write a string slice to COM1.
pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        // Convert \n to \r\n for serial terminals
        if byte == b'\n' {
            serial_write_byte(b'\r');
        }
        serial_write_byte(byte);
    }
}

// ── Implement core::fmt::Write so we can use write!/writeln! ──────────────

pub struct SerialWriter;

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        serial_write_str(s);
        Ok(())
    }
}

/// Global serial writer for use with serial_print! macro.
pub static SERIAL_WRITER: spin::Mutex<SerialWriter> = spin::Mutex::new(SerialWriter);

// ── Macros ────────────────────────────────────────────────────────────────

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut writer = $crate::head::serial::SERIAL_WRITER.lock();
            let _ = write!(writer, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! serial_println {
    () => { $crate::serial_print!("\n") };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut writer = $crate::head::serial::SERIAL_WRITER.lock();
            let _ = writeln!(writer, $($arg)*);
        }
    };
}
