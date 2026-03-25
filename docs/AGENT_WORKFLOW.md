# AGENT_WORKFLOW.md — Multi-Agent Coordination

## The Golden Rule
Agents communicate through files only. Every decision is a commit.

---

## Task Claim Protocol

```bash
# BEFORE any code:
ls task-locks/DOMAIN_*.lock        # check if claimed
echo "agent: Agent-XX
task: taskname
branch: agent/DOMAIN-taskname
started: $(date -u)" > task-locks/DOMAIN_taskname.lock
git add task-locks/ && git commit -m "lock: DOMAIN taskname [Agent-XX]"

# AFTER task complete (in PR):
git rm task-locks/DOMAIN_taskname.lock
```

Stale lock = >24h old with no commits on branch. Any agent may reclaim.

---

## Parallel Work Map

```
Phase 1 — BUILD alone:
  Agent-01 (BUILD) → toolchain, Makefile, venv

Phase 2 — HEAD alone (needs BUILD):
  Agent-02 (HEAD) → multiboot2, GDT, serial

Phase 3 — parallel (need HEAD):
  Agent-03 (TAPE)    → regions, allocator, heap
  Agent-06 (DEVICE)  → PIC, PIT, keyboard (can start IDT/PIC without TAPE)
  Agent-07 (DISPLAY) → VGA writer (VGA_BASE is fixed, no TAPE dependency)

Phase 4 — parallel (need TAPE + DEVICE):
  Agent-04 (CTRL)    → state machine, machine_loop
  Agent-05 (TRANS)   → syscall gate, IRQ handlers

Phase 5 — parallel (need CTRL + TRANS):
  Agent-08 (STORE)   → FAT12, ATA
  Agent-09 (PROC)    → TCB, context switch

Phase 6 — SHELL (needs PROC + TRANS + STORE):
  Agent-10 (SHELL) → libc, shell

Agent-11 (TEST) runs throughout — adds tests as domains complete.
```

---

## Cross-Domain Write Protocol

1. Create dispatch card for the owning domain: `DISPATCH_OTHERDOMAIN_NNN.md`
2. Mark your task `BLOCKED` in ITERATIONS.md
3. Do NOT write to their files — let the owning agent do it
4. If emergency: create lock file, document in ITERATIONS.md, keep change minimal

---

## PR Requirements

Every PR to `dev` must have:
- [ ] Dispatch card ID in description
- [ ] `make build` passing
- [ ] `make qemu-test TEST=<name>` passing
- [ ] ITERATIONS.md entry for this task
- [ ] No `unsafe` without `// SAFETY:`
- [ ] Lock file deleted
- [ ] All 32-bit (no `u64` addresses, no 64-bit registers)

---

## Branch Naming

```
agent/DOMAIN-taskname     (e.g., agent/HEAD-multiboot2-entry)
fix/DOMAIN-MISTAKE-NNN    (e.g., fix/TRANS-MISTAKE-003)
```
