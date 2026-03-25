# CONTRIBUTING.md — Code Standards & PR Rules

---

## The #1 Rule: This Is 32-bit

Every line of kernel code must be 32-bit. Check before committing:

| ✅ Correct | ❌ Wrong — will fail or silently break |
|---|---|
| `u32` for addresses | `u64` for addresses |
| `eax`, `esp` in asm | `rax`, `rsp` in asm |
| `int 0x80` syscall | `syscall` instruction |
| `qemu-system-i386` | `qemu-system-x86_64` |
| `i686-unknown-none` target | `x86_64-unknown-none` target |
| Physical addresses only | Virtual addresses / page tables |

Run `make verify-32bit` after every build. It will tell you immediately if you built 64-bit by accident.

---

## Code Rules

```rust
// ✅ REQUIRED on every unsafe block
unsafe {
    // SAFETY: [explain why this is sound — specific reason]
    *ptr = value;
}

// ✅ REQUIRED on every exported function
/// Brief description.
/// # Safety / Panics / Returns — as applicable
pub fn exported_fn(addr: u32) -> u32 { ... }

// ✅ REQUIRED: always use tape region constants
use kernel::tape::regions::VGA_BASE;  // ✅
// NOT:
let vga = 0xB8000 as *mut u16;        // ❌ hardcoded address

// ✅ REQUIRED: volatile for all hardware memory
tape::tape_write(VGA_BASE + offset, cell as u32);  // ✅
// NOT:
unsafe { *(0xB8000 as *mut u16) = cell; }          // ❌ non-volatile

// ❌ FORBIDDEN
use std::*;                            // no std in kernel
unsafe { *ptr = v; }                   // no unsafe without SAFETY comment
let addr: u64 = 0xB8000;              // no u64 addresses
asm!("mov rax, rbx");                  // no 64-bit registers
```

---

## Commit Format

```
type(domain): short description — DISPATCH_DOMAIN_NNN

type: feat | fix | test | docs | build | refactor | lock | unlock
domain: build | head | tape | ctrl | trans | device | display | store | proc | shell | test
```

Examples:
```
feat(tape): implement bump allocator — DISPATCH_TAPE_001
fix(trans): send PIC EOI in timer handler — Fixes: MISTAKE-003
lock(ctrl): claim state machine task [Agent-04]
```

---

## PR Checklist

- [ ] `make build` passes
- [ ] `make verify-32bit` passes (ELF is Intel 80386)
- [ ] `make qemu-test TEST=<relevant>` passes
- [ ] No `unsafe` without `// SAFETY:`
- [ ] No hardcoded addresses (use `tape::regions::*`)
- [ ] ITERATIONS.md has an entry for this task
- [ ] Task lock file deleted in this PR
- [ ] Dispatch card ID in PR description

---

## File Structure Rules

- Kernel source: `kernel/src/[domain]/` only
- Python tools: `tools/` only, never in `kernel/`
- Tests: `tests/integration/` (QEMU) and `#[cfg(test)]` (unit, in source files)
- Dispatch cards: `dispatch-queue/DISPATCH_DOMAIN_NNN.md` exactly
- Context snapshots: `context-snapshots/YYYY-MM-DD_DOMAIN_vN.md` exactly
- Task locks: `task-locks/DOMAIN_taskname.lock` exactly

Never create: `src/` at root, `lib/`, `include/`, `v1/`, `tmp/`, `scratch/`

---

## Domain Ownership

Do not write to another domain's files without a recorded handoff in AGENT_WORKFLOW.md. Check `MASTER.md` Section 6 for who owns what path.
