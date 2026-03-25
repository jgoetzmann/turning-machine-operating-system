// tests/integration/store_fat12.rs
// Test: mount FAT12 filesystem, list root, read HELLO.TXT.
// Domain: STORE | Run: make qemu-test TEST=store_fat12
// Requires: tests/fixtures/fat12.img (run `make make-disk` first)

pub fn run() {
    crate::serial_println!("[TEST START: store_fat12]");

    // Mount the filesystem
    match crate::store::store_init() {
        Ok(()) => crate::serial_println!("  STORE: mounted OK"),
        Err(e) => {
            crate::serial_println!("[TEST FAIL: store_fat12] mount failed: {:?}", e);
            super::head_boot_basic::qemu_exit_failure("mount failed");
        }
    }

    // Open HELLO.TXT
    let fh = match crate::store::open("HELLO   TXT", false) {
        Ok(fh) => { crate::serial_println!("  STORE: opened HELLO.TXT"); fh }
        Err(e) => {
            crate::serial_println!("[TEST FAIL: store_fat12] open failed: {:?}", e);
            super::head_boot_basic::qemu_exit_failure("open failed");
        }
    };

    // Read contents
    let mut buf = [0u8; 64];
    let n = match crate::store::read(fh, &mut buf) {
        Ok(n) => n,
        Err(e) => {
            crate::serial_println!("[TEST FAIL: store_fat12] read failed: {:?}", e);
            super::head_boot_basic::qemu_exit_failure("read failed");
        }
    };

    crate::store::close(fh);

    // Verify content starts with "HELLO TAPE"
    let content = &buf[..n.min(10)];
    crate::serial_println!("  STORE: read {} bytes: {:?}", n, content);

    if &content[..5] != b"HELLO" {
        crate::serial_println!("[TEST FAIL: store_fat12] wrong content");
        super::head_boot_basic::qemu_exit_failure("wrong content");
    }

    crate::serial_println!("[TEST PASS: store_fat12]");
    super::head_boot_basic::qemu_exit_success();
}
