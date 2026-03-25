# TASK_DISPATCH.md — Dispatch Card System

---

## Dispatch Card Schema

**Filename:** `dispatch-queue/DISPATCH_[DOMAIN]_[NNN].md`

```markdown
# DISPATCH_[DOMAIN]_[NNN] — Task Title

## Header
**Domain:** DOMAIN | **Agent:** Agent-XX | **Status:** PENDING
**Priority:** CRITICAL / HIGH / NORMAL | **Created:** YYYY-MM-DD
**Branch:** agent/domain-taskname

## Prerequisites
- [ ] DISPATCH_XXX_NNN must be COMPLETE

## Context Snapshot
`context-snapshots/YYYY-MM-DD_DOMAIN_vN.md` (or "none — first task")

## Objective
One paragraph. Specific end state with test name.

## Acceptance Criteria
- [ ] Specific testable item
- [ ] `make qemu-test TEST=name` passes
- [ ] No `unsafe` without `// SAFETY:`
- [ ] ITERATIONS.md updated
- [ ] **All addresses are 32-bit `u32`, no `u64`**

## Files To Touch
- CREATE: path — purpose
- MODIFY: path — what changes
- DO NOT TOUCH: path (owned by DOMAIN)

## API Spec
```rust
pub fn exact_signature(arg: u32) -> u32;
```

## Pitfalls
- See MISTAKE-NNN: one-line summary

## Test
```bash
make qemu-test TEST=test_name
```

## On Completion
1. Check acceptance criteria
2. Append ITER-NNN to ITERATIONS.md
3. Set Status: COMPLETE
4. Delete task lock
5. PR to `dev`
6. Generate next dispatch card
```

---

## How To Send An Agent

Paste in this order into a new Cursor session:
1. Contents of `BOOTSTRAP_PROMPT.md`
2. Contents of `docs/agents/DOMAIN.md`
3. Contents of `dispatch-queue/DISPATCH_DOMAIN_NNN.md`
4. Contents of latest `context-snapshots/DATE_DOMAIN_vN.md` (if exists)

Then: `"Read all of the above. Claim your task lock. Begin work on the dispatch card."`

---

## Auto-Generate Next Dispatch Card

At task completion, paste this prompt:

```
DISPATCH AUTO-GENERATION

Just completed: [DISPATCH_DOMAIN_NNN — title]

Built:
[what was implemented]

Now available to other domains:
[exported APIs]

Still TODO in my domain:
[next logical task]

Other domains now unblocked:
[list]

Generate:
1. DISPATCH_[MYDOMAIN]_[N+1] for my next task
2. DISPATCH_[DOMAIN]_NNN for each domain I unblocked

Use schema from TASK_DISPATCH.md.
Pre-fill pitfalls from docs/MISTAKES.md.
All addresses must be u32, all registers 32-bit.
```

---

## Queue Status

```bash
source .venv/bin/activate
python3 tools/dispatch_generator.py --status

# Create new card
python3 tools/dispatch_generator.py --create --domain CTRL --title "Implement machine loop"
```
