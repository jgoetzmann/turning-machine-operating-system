# CONTEXT_PROTOCOL.md — Context Compression

## When To Compress

- Context > 60% full
- Completing a task boundary
- Handing off to another session

## Compression Steps

**Step 1** — Paste this prompt to yourself:

```
CONTEXT COMPRESSION — [DOMAIN] — v[N]

Generate a snapshot:

## CONTEXT SNAPSHOT — [DOMAIN] — [DATE] — v[N]

### Working State
- What is DONE and tested: (bullet list)
- What is IN PROGRESS: (current task)
- What is NOT started yet: (so I don't assume it's done)

### Locked Decisions
- LOCKED: [decision] — [reason]

### Open Problems
- OPEN: [problem] — [what's needed]

### Files Changed
- CREATED/MODIFIED: path — what changed

### Mistakes Found
- LOGGED: MISTAKE-NNN — title (if any)

### Next Action
[Single sentence. What to do first in the next session.]

### Handoff Note
[3-5 sentences briefing a cold reader on current state.
Include any 32-bit specific gotchas you discovered.]
```

**Step 2** — Save to `context-snapshots/YYYY-MM-DD_DOMAIN_vN.md`, commit.

**Step 3** — New session: paste Bootstrap Prompt + Domain MD + this snapshot + dispatch card.

## What To Prune (Do NOT include in snapshot)

- ❌ Debugging steps that led to a solution (keep only the solution)
- ❌ Code you wrote then deleted
- ❌ Long exploratory reasoning (keep the conclusion)
- ❌ Full file contents (reference path + what changed)

## What To Keep

- ✅ Every design decision that must not be revisited
- ✅ Every bug found and how it was fixed
- ✅ Current state of owned files
- ✅ What other domains you're waiting on
- ✅ 32-bit specific discoveries (important — this is not a standard target)

## Auto-Generate Snapshot

```bash
source .venv/bin/activate
python3 tools/context_compressor.py --domain CTRL --version 2
```
