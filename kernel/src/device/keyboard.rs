// kernel/src/device/keyboard.rs
// PS/2 Keyboard driver — scancode set 1 → ASCII
// Agent: DEVICE (Agent-06)

const KBD_DATA:   u16 = 0x60;
const KBD_STATUS: u16 = 0x64;
const KB_BUF_SIZE: usize = 64;

unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!("in al, dx", out("al") v, in("dx") port,
                     options(nomem, nostack, preserves_flags));
    v
}

/// Simple scancode set 1 → ASCII map for printable keys.
/// Index = scancode, value = ASCII (0 = unmapped).
static SCANCODE_MAP: [u8; 58] = [
    0,   0,   b'1', b'2', b'3', b'4', b'5', b'6',   // 0x00-0x07
    b'7', b'8', b'9', b'0', b'-', b'=', 0x08, b'\t', // 0x08-0x0F  (0x08=backspace)
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',  // 0x10-0x17
    b'o', b'p', b'[', b']', b'\n', 0,   b'a', b's',  // 0x18-0x1F  (0=ctrl)
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';',  // 0x20-0x27
    b'\'', b'`', 0,   b'\\', b'z', b'x', b'c', b'v', // 0x28-0x2F (0=lshift)
    b'b', b'n', b'm', b',', b'.', b'/', 0,   b'*',   // 0x30-0x37 (0=rshift)
    0,   b' ',                                          // 0x38-0x39 (0=lalt, space)
];

/// Circular keyboard buffer
static mut KBD_BUF: [u8; KB_BUF_SIZE] = [0u8; KB_BUF_SIZE];
static mut KBD_HEAD: usize = 0;
static mut KBD_TAIL: usize = 0;

/// Initialize the PS/2 keyboard.
pub fn keyboard_init() {
    // Flush any pending scancodes
    unsafe {
        while inb(KBD_STATUS) & 0x01 != 0 {
            let _ = inb(KBD_DATA);
        }
    }
}

/// Called from IRQ1 handler — read one scancode and buffer it.
pub fn keyboard_handle_irq() {
    // SAFETY: Reading port 0x60 (keyboard data) is always safe in kernel context.
    let scancode = unsafe { inb(KBD_DATA) };

    // Ignore key-up events (bit 7 set = key release)
    if scancode & 0x80 != 0 { return; }

    // Convert scancode to ASCII
    let ascii = if (scancode as usize) < SCANCODE_MAP.len() {
        SCANCODE_MAP[scancode as usize]
    } else {
        0
    };

    if ascii == 0 { return; }  // unmapped key

    // Buffer the character (SAFETY: single core, called from IRQ with IF=0)
    unsafe {
        let next_tail = (KBD_TAIL + 1) % KB_BUF_SIZE;
        if next_tail != KBD_HEAD {
            KBD_BUF[KBD_TAIL] = ascii;
            KBD_TAIL = next_tail;
        }
        // Buffer full: silently drop (don't block in IRQ handler)
    }
}

/// Non-blocking read of one character from the keyboard buffer.
/// Returns None if no key has been pressed.
pub fn keyboard_read_char() -> Option<char> {
    // SAFETY: Single core. Called from kernel context.
    unsafe {
        if KBD_HEAD == KBD_TAIL { return None; }  // buffer empty
        let ch = KBD_BUF[KBD_HEAD];
        KBD_HEAD = (KBD_HEAD + 1) % KB_BUF_SIZE;
        Some(ch as char)
    }
}
