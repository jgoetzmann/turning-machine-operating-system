#!/usr/bin/env python3
"""dispatch_generator.py — Manage TM OS dispatch cards."""

import os, re, sys, argparse
from datetime import date
from pathlib import Path

REPO_ROOT     = Path(__file__).parent.parent
DISPATCH_DIR  = REPO_ROOT / "dispatch-queue"
LOCK_DIR      = REPO_ROOT / "task-locks"
DOCS_DIR      = REPO_ROOT / "docs"
MISTAKES_FILE = DOCS_DIR / "MISTAKES.md"

DOMAIN_AGENTS = {
    "BUILD": "Agent-01", "HEAD": "Agent-02", "TAPE": "Agent-03",
    "CTRL": "Agent-04",  "TRANS": "Agent-05", "DEVICE": "Agent-06",
    "DISPLAY": "Agent-07", "STORE": "Agent-08", "PROC": "Agent-09",
    "SHELL": "Agent-10", "TEST": "Agent-11",
}

DOMAIN_MD = {
    "BUILD": "BUILD.md", "HEAD": "HEAD.md", "TAPE": "TAPE.md",
    "CTRL": "CTRL.md", "TRANS": "TRANS.md", "DEVICE": "DEVICE.md",
    "DISPLAY": "DISPLAY.md", "STORE": "STORE.md", "PROC": "PROC.md",
    "SHELL": "SHELL.md", "TEST": "TEST.md",
}


def next_num(domain: str) -> str:
    existing = list(DISPATCH_DIR.glob(f"DISPATCH_{domain}_*.md")) if DISPATCH_DIR.exists() else []
    nums = [int(re.search(r"_(\d+)\.md$", f.name).group(1)) for f in existing
            if re.search(r"_(\d+)\.md$", f.name)]
    return str((max(nums) + 1) if nums else 1).zfill(3)


def domain_mistakes(domain: str) -> list:
    if not MISTAKES_FILE.exists(): return []
    content = MISTAKES_FILE.read_text()
    entries = []
    for m in re.finditer(rf"## \[(MISTAKE-\d+)\] \[([^\]]*{domain}[^\]]*)\] — (.+)", content):
        entries.append(f"- See {m.group(1)}: {m.group(3).strip()}")
    return entries


def print_status():
    states = {"PENDING": [], "IN_PROGRESS": [], "BLOCKED": [], "COMPLETE": [], "ABANDONED": []}
    if DISPATCH_DIR.exists():
        for f in sorted(DISPATCH_DIR.glob("DISPATCH_*.md")):
            m = re.search(r"\*\*Status:\*\* (\w+)", f.read_text())
            if m and m.group(1) in states:
                states[m.group(1)].append(f.name)

    print("\n=== DISPATCH QUEUE — TM OS ===\n")
    for state, cards in states.items():
        if cards:
            print(f"{state} ({len(cards)}):")
            for c in cards: print(f"  - {c}")

    if LOCK_DIR.exists():
        locks = list(LOCK_DIR.glob("*.lock"))
        if locks:
            print(f"\nACTIVE LOCKS ({len(locks)}):")
            import time
            for lf in locks:
                age_h = (time.time() - lf.stat().st_mtime) / 3600
                stale = " ⚠️  STALE" if age_h > 24 else ""
                print(f"  - {lf.name} ({age_h:.1f}h){stale}")
    print()


def create_card(domain: str, title: str, priority: str = "NORMAL") -> Path:
    if domain not in DOMAIN_AGENTS:
        print(f"Unknown domain '{domain}'. Valid: {list(DOMAIN_AGENTS.keys())}")
        sys.exit(1)

    DISPATCH_DIR.mkdir(exist_ok=True)
    num = next_num(domain)
    dispatch_id = f"DISPATCH_{domain}_{num}"
    agent = DOMAIN_AGENTS[domain]
    md_file = DOMAIN_MD[domain]
    today = date.today().isoformat()
    mistakes = domain_mistakes(domain)
    pitfalls = "\n".join(mistakes) if mistakes else "- No domain mistakes logged yet. Check docs/MISTAKES.md."
    branch = f"agent/{domain.lower()}-{title.lower().replace(' ','-')[:40]}"

    # Find latest snapshot
    snap_dir = REPO_ROOT / "context-snapshots"
    snaps = sorted(snap_dir.glob(f"*_{domain}_*.md")) if snap_dir.exists() else []
    snapshot_ref = f"`context-snapshots/{snaps[-1].name}`" if snaps else f"None — load `docs/agents/{md_file}` only."

    content = f"""# {dispatch_id} — {title}

## Header
**Domain:** {domain} | **Agent:** {agent} | **Status:** PENDING
**Priority:** {priority} | **Created:** {today}
**Branch:** {branch}

## Prerequisites
- [ ] [FILL: which dispatch cards must complete first, or "None"]

## Context Snapshot
{snapshot_ref}

## Objective
[FILL: One paragraph. Specific end state. Name the test that must pass.]

## Acceptance Criteria
- [ ] [FILL: specific testable item]
- [ ] `make qemu-test TEST=[test_name]` passes
- [ ] No `unsafe` without `// SAFETY:` comment
- [ ] ITERATIONS.md updated with this task's result
- [ ] **All addresses are `u32`. All registers 32-bit. No `u64` addresses.**

## Files To Touch
- CREATE/MODIFY: `path` — purpose
- DO NOT TOUCH: (files owned by other domains)

## API Spec
```rust
// [FILL: exact function signatures to implement]
```

## Pitfalls
{pitfalls}

## Test
```bash
make qemu-test TEST=[test_name] TIMEOUT=20
```
Expected: `[TEST PASS: [test_name]]` on serial output.

## On Completion
1. Check all acceptance criteria
2. Append to `docs/ITERATIONS.md`
3. Set Status: COMPLETE, fill Completed date
4. Delete `task-locks/{domain}_[taskname].lock`
5. PR to `dev`: `feat({domain.lower()}): {title} — {dispatch_id}`
6. Generate next dispatch card

## Related Cards
- (fill: cards this blocks or is blocked by)
"""

    path = DISPATCH_DIR / f"{dispatch_id}.md"
    path.write_text(content)
    print(f"✅ Created: {path}")
    return path


def main():
    p = argparse.ArgumentParser(description="TM OS dispatch queue tool")
    p.add_argument("--status", action="store_true")
    p.add_argument("--create", action="store_true")
    p.add_argument("--domain")
    p.add_argument("--title")
    p.add_argument("--priority", default="NORMAL")
    args = p.parse_args()

    if args.status:
        print_status()
    elif args.create:
        if not args.domain or not args.title:
            print("--create requires --domain and --title")
            sys.exit(1)
        create_card(args.domain, args.title, args.priority)
    else:
        p.print_help()

if __name__ == "__main__":
    main()
