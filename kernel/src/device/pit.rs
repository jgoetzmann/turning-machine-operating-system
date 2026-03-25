// kernel/src/device/pit.rs
// 8253/8254 Programmable Interval Timer — system clock
// Agent: DEVICE (Agent-06)
//
// PIT channel 0 generates IRQ0 at a configurable frequency.
// We use 100Hz = 10ms tick. Divisor = 1193182 / 100 = 11931.
//
// Ports: channel 0 data=0x40, command=0x43

const PIT_CHANNEL0: u16 = 0x40;
const PIT_CMD:      u16 = 0x43;
const PIT_BASE_HZ:  u32 = 1_193_182;  // PIT input frequency (~1.193 MHz)

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val,
                     options(nomem, nostack, preserves_flags));
}

/// Global tick counter — incremented on every IRQ0.
/// u64 at 100Hz wraps in ~5.8 billion years.
static TICK_COUNT: spin::Mutex<u64> = spin::Mutex::new(0);

/// Initialize PIT channel 0 at the given frequency in Hz.
/// Typical value: 100 (100Hz = 10ms per tick).
pub fn pit_init(freq_hz: u32) {
    let divisor = (PIT_BASE_HZ / freq_hz) as u16;

    unsafe {
        // Mode command: channel 0, lobyte/hibyte, mode 3 (square wave)
        outb(PIT_CMD, 0x36);
        // Send divisor: low byte first, then high byte
        outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0, (divisor >> 8) as u8);
    }
}

/// Called from IRQ0 handler — increment the tick counter.
/// Called by trans::irq::timer_handler.
pub fn pit_tick() {
    let mut ticks = TICK_COUNT.lock();
    *ticks = ticks.wrapping_add(1);
}

/// Return current tick count (monotonically increasing).
pub fn pit_get_ticks() -> u64 {
    *TICK_COUNT.lock()
}

/// Busy-wait for N ticks (each tick = 10ms at 100Hz).
/// Note: This blocks the entire machine — only use during init.
pub fn pit_sleep_ticks(ticks: u64) {
    let start = pit_get_ticks();
    while pit_get_ticks() - start < ticks {
        unsafe { core::arch::asm!("hlt") };
    }
}
