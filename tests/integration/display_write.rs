// tests/integration/display_write.rs
// Test: VGA text mode writes characters to the tape buffer correctly.
// Domain: DISPLAY | Run: make qemu-test TEST=display_write

pub fn run() {
    use crate::tape::regions::VGA_BASE;
    crate::serial_println!("[TEST START: display_write]");

    {
        let mut w = crate::display::WRITER.lock();
        w.clear();
        w.write_byte(b'T');
        w.write_byte(b'M');
        w.write_byte(b'O');
        w.write_byte(b'S');
    }

    // Read back the VGA buffer and verify the first 4 characters
    // Each VGA cell is 2 bytes: [char_byte, color_byte]
    // SAFETY: VGA_BASE is a valid, initialized MMIO tape region.
    let c0: u16 = unsafe { crate::tape::tape_read(VGA_BASE + 0) };
    let c1: u16 = unsafe { crate::tape::tape_read(VGA_BASE + 2) };
    let c2: u16 = unsafe { crate::tape::tape_read(VGA_BASE + 4) };
    let c3: u16 = unsafe { crate::tape::tape_read(VGA_BASE + 6) };

    let chars = [c0 & 0xFF, c1 & 0xFF, c2 & 0xFF, c3 & 0xFF];

    crate::serial_println!("  DISPLAY: VGA[0..4] = {:?}", chars);

    assert_eq!(chars[0] as u8, b'T', "VGA[0] != 'T'");
    assert_eq!(chars[1] as u8, b'M', "VGA[1] != 'M'");
    assert_eq!(chars[2] as u8, b'O', "VGA[2] != 'O'");
    assert_eq!(chars[3] as u8, b'S', "VGA[3] != 'S'");

    crate::serial_println!("[TEST PASS: display_write]");
    super::head_boot_basic::qemu_exit_success();
}
