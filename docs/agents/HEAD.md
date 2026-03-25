# HEAD.md — Boot & CPU Initialization Agent
> **Domain ID:** `HEAD` | **Agent Slot:** Agent-02
> **TM Role:** Position the tape head. Set up the machine so it can begin reading.
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

The HEAD domain initializes the x86 CPU so the tape machine can operate:

- **Multiboot2 header**: lets GRUB/QEMU load the kernel ELF
- **Protected mode entry**: CPU is already in protected mode from GRUB, but we set up our own GDT
- **GDT**: flat 32-bit segmentation — code segment `0x08`, data segment `0x10`, NO paging
- **IDT**: 32-bit interrupt descriptor table — wire up all exception and IRQ vectors
- **Stack setup**: 16KB kernel stack at fixed tape address
- **COM1 serial**: earliest possible debug output before anything else works
- **Memory map parsing**: read what GRUB tells us about the tape (physical memory map)
- **Kernel entry**: call `ctrl::machine_boot()` and never return

---

## 2. File Ownership

```
kernel/src/head/
├── mod.rs              # boot_init() — called from _start
├── boot.asm            # Multiboot2 header + _start (NASM, 32-bit)
├── gdt.rs              # Flat 32-bit GDT (3 entries: null, code, data)
├── idt.rs              # IDT — 256 entries, wired to TRANS domain handlers
├── serial.rs           # COM1 UART init + write (before anything else works)
└── memmap.rs           # Parse Multiboot2 memory map → tape::regions
```

**You MAY modify:**
- `kernel/src/main.rs` — only to adjust the `_start` → `boot_init` call

**You MUST NOT modify:**
- `kernel/src/tape/` — owned by TAPE
- `kernel/src/ctrl/` — owned by CTRL
- Anything in `kernel/src/trans/` — IDT entries point there, but you don't own the handlers

---

## 3. API Contracts (What You Export)

```rust
// head/mod.rs
/// Entry point called from _start after minimal ASM setup.
/// Sets up GDT, IDT, serial, parses memory map, then calls ctrl::machine_boot().
pub fn boot_init(multiboot_magic: u32, multiboot_info_phys: u32) -> !;

/// Physical tape regions discovered from multiboot2 memory map.
/// TAPE domain reads this to initialize its allocator.
pub fn get_tape_memory_map() -> &'static [TapeRegion];

// head/serial.rs — available to ALL domains for debug output
pub fn serial_init();
pub fn serial_write_byte(b: u8);
pub fn serial_write_str(s: &str);

#[macro_export] macro_rules! serial_print { ... }
#[macro_export] macro_rules! serial_println { ... }

// head/gdt.rs
pub struct GdtDescriptor { limit: u16, base: u32 }
pub fn gdt_init();
pub fn gdt_load();

// head/idt.rs
pub fn idt_init();        // zeros all 256 entries
pub fn idt_set_gate(vector: u8, handler: u32, flags: u8);
pub fn idt_load();
```

---

## 4. GDT Layout (Flat 32-bit — Do Not Change)

```
Index 0 (0x00): Null descriptor
Index 1 (0x08): Code segment — base=0, limit=4GB, DPL=0, executable, readable
Index 2 (0x10): Data segment — base=0, limit=4GB, DPL=0, writable
Index 3 (0x18): User code  — base=0, limit=4GB, DPL=3, executable, readable
Index 4 (0x20): User data  — base=0, limit=4GB, DPL=3, writable
Index 5 (0x28): TSS        — base=&tss, limit=sizeof(TSS), DPL=0 (for ring 3 → ring 0)
```

**"Flat" means all segments have base=0, limit=4GB.** There is no segmentation protection — the tape is accessible from any segment. This is intentional: we are a Turing Machine, the tape is flat.

---

## 5. Boot Sequence

```
QEMU -kernel loads kernel ELF at 0x00100000
    → Jumps to _start (in boot.asm)

_start (NASM, 32-bit protected mode):
    1. Validate multiboot magic (eax == 0x36d76289) — halt if wrong
    2. Set up initial stack: esp = KERNEL_STACK_TOP (defined in linker.ld)
    3. Zero BSS segment
    4. Push multiboot_info address and magic onto stack
    5. Call boot_init(magic, multiboot_info_phys) [Rust]

boot_init() [Rust]:
    1. serial_init()        — COM1 up, now we can print
    2. serial_println!("HEAD: tape machine booting")
    3. gdt_init() + gdt_load()
    4. idt_init()           — zeros IDT (TRANS fills gates later)
    5. parse_memory_map(multiboot_info_phys)
    6. serial_println!("HEAD: tape regions mapped")
    7. Call ctrl::machine_boot()   — hand off to CTRL domain
    8. loop {} — should never reach here
```

---

## 6. Critical Constraints

- **NO heap allocation in HEAD**. The allocator doesn't exist yet. All data structures are `static` or on the stack.
- **NO paging**. Do not touch CR0.PG, CR3, or any paging register. See INV-04.
- **Stack is at a fixed tape address**: `0x00007000` (below the kernel, in low memory). This is set in `boot.asm` before calling Rust.
- **Multiboot2 magic validation**: if EAX ≠ `0x36d76289` on entry, print error to serial and `hlt`.
- **IDT is initialized empty here** — the actual gate entries are filled by TRANS domain during `ctrl::machine_boot()`. HEAD just allocates the IDT table and calls `lidt`.
- **Flat segments**: after `gdt_load()`, CS=0x08, DS=ES=FS=GS=SS=0x10. Never use segment overrides.
- **32-bit everything**: all registers are 32-bit. Do not generate `rex` prefixes or 64-bit operand sizes.

---

## 7. Tape Regions Discovered at Boot

HEAD reads the Multiboot2 memory map and populates:

```rust
pub struct TapeRegion {
    pub base: u32,
    pub length: u32,
    pub kind: TapeRegionKind,
}

pub enum TapeRegionKind {
    Available,   // usable RAM — TAPE domain can allocate here
    Reserved,    // BIOS/MMIO — do not touch
    AcpiReclaimable,
    BadMemory,
}
```

TAPE domain reads `head::get_tape_memory_map()` to know what physical tape is usable.

---

## 8. Known Mistakes to Avoid

> Always check `docs/MISTAKES.md` for `[HEAD]` entries.

- **MISTAKE: Using a stack before setting ESP** — `boot.asm` must set ESP before calling any Rust. Rust function prologues use the stack immediately.
- **MISTAKE: Calling Rust before zeroing BSS** — static variables with value `0` are in BSS. If BSS isn't zeroed, they have garbage values. Zero it in `_start` before calling `boot_init`.
- **MISTAKE: IDT base as virtual address** — we have no paging, so physical = virtual. But the pointer you load with `lidt` must point to the actual IDT table in memory. Don't accidentally take an offset.
- **MISTAKE: Not reloading segment registers after GDT load** — after `lgdt`, CS still has the old value. You must do a far jump (`ljmp 0x08:reload_cs`) to reload CS. DS/ES/FS/GS/SS must be explicitly loaded.

---

## 9. Task Workflow

```bash
# Claim task
echo "agent: Agent-02 / task: multiboot-entry / started: $(date)" > task-locks/HEAD_boot-entry.lock

# Build
make build

# Test  
make qemu-test TEST=head_boot_basic   # expects "[TEST PASS: head_boot_basic]" on serial

# Complete
git rm task-locks/HEAD_boot-entry.lock
# append to docs/ITERATIONS.md
# update dispatch card
# open PR
```
