# TEST.md — Test Infrastructure Agent
> **Domain ID:** `TEST` | **Agent Slot:** Agent-11
> **Read MASTER.md → MISTAKES.md → AGENT_WORKFLOW.md first.**

---

## 1. Domain Purpose

QEMU-based test harness:
- Boot kernel in `qemu-system-i386` (32-bit)
- Capture serial output
- Look for `[TEST PASS: name]` / `[TEST FAIL: name]` sentinels
- Exit via `isa-debug-exit` device

---

## 2. Test List

| Test | Domain | Verifies |
|---|---|---|
| `head_boot_basic` | HEAD | Boots, prints to serial, no panic |
| `tape_alloc` | TAPE | Bump allocator works |
| `tape_regions` | TAPE | Region constants correct |
| `ctrl_states` | CTRL | State transitions fire correctly |
| `trans_syscall` | TRANS | `int 0x80` returns from ring 3 |
| `device_timer` | DEVICE | PIT fires at ~100Hz |
| `device_keyboard` | DEVICE | Keypress produces scancode |
| `display_write` | DISPLAY | Text appears in VGA buffer |
| `store_fat12` | STORE | Mount, list root, read file |
| `proc_create` | PROC | Create process, run, exit |
| `shell_echo` | SHELL | `echo hello` prints to VGA |

---

## 3. QEMU Command (32-bit!)

```bash
qemu-system-i386 \
  -kernel target/i686-unknown-none/release/kernel \
  -serial stdio \
  -display none \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -drive file=tests/fixtures/fat12.img,format=raw,if=ide \
  -m 128M \
  -no-reboot \
  -append "test=TEST_NAME"
```

---

## 4. Known Mistakes to Avoid

- **MISTAKE: Using `qemu-system-x86_64` instead of `qemu-system-i386`** — the binary is 32-bit. Using the 64-bit QEMU will fail to boot or behave unexpectedly.
- **MISTAKE: Test hangs without `isa-debug-exit`** — always configure this device. Without it, the test runner hangs on timeout.

---
