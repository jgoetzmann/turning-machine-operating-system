#!/usr/bin/env python3
"""test_runner.py — QEMU i386 kernel test runner."""

import os, re, sys, json, argparse, subprocess, threading, time
from datetime import datetime
from pathlib import Path

REPO_ROOT   = Path(__file__).parent.parent
RESULTS_DIR = REPO_ROOT / "tests" / "results"
KERNEL_ELF  = REPO_ROOT / "target" / "i686-unknown-none" / "release" / "kernel"
FAT12_IMG   = REPO_ROOT / "tests" / "fixtures" / "fat12.img"
QEMU        = "qemu-system-i386"    # 32-BIT — never qemu-system-x86_64

ALL_TESTS = [
    "head_boot_basic",
    "tape_alloc", "tape_regions",
    "ctrl_states",
    "trans_syscall", "trans_timer",
    "device_timer", "device_keyboard", "device_ata",
    "display_write",
    "store_fat12",
    "proc_create", "proc_context_switch",
    "shell_echo", "shell_exit",
]
DISK_TESTS = {"device_ata", "store_fat12", "shell_echo", "shell_exit"}


def build_qemu_cmd(test: str) -> list:
    cmd = [
        QEMU, "-kernel", str(KERNEL_ELF),
        "-serial", "stdio",
        "-display", "none",
        "-m", "128M",
        "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-append", f"test={test}",
    ]
    if test in DISK_TESTS and FAT12_IMG.exists():
        cmd += ["-drive", f"file={FAT12_IMG},format=raw,if=ide"]
    return cmd


class Result:
    def __init__(self, name):
        self.name, self.passed, self.timed_out = name, False, False
        self.fail_reason, self.serial = "", ""
        self.duration = 0.0

    def status(self): return "TIMEOUT" if self.timed_out else ("PASS" if self.passed else "FAIL")


def run_test(name: str, timeout: int, verbose: bool = False) -> Result:
    r = Result(name)
    if not KERNEL_ELF.exists():
        r.fail_reason = f"Kernel not found: {KERNEL_ELF}. Run 'make build'."
        return r

    cmd = build_qemu_cmd(name)
    lines = []
    start = time.time()

    try:
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                 text=True, bufsize=1)

        def read():
            for line in proc.stdout:
                line = line.rstrip()
                lines.append(line)
                if verbose: print(f"    | {line}")
                if f"[TEST PASS: {name}]" in line:
                    r.passed = True; proc.terminate(); return
                if f"[TEST FAIL: {name}]" in line:
                    m = re.search(r"\[TEST FAIL: \w+\] (.+)", line)
                    r.fail_reason = m.group(1) if m else line
                    proc.terminate(); return
                if "KERNEL PANIC" in line or "PANIC" in line.upper():
                    r.fail_reason = line; proc.terminate(); return

        t = threading.Thread(target=read, daemon=True)
        t.start(); t.join(timeout=timeout)
        if t.is_alive():
            proc.terminate(); r.timed_out = True
            r.fail_reason = f"Timed out after {timeout}s"
        proc.wait(timeout=2)

    except FileNotFoundError:
        r.fail_reason = f"'{QEMU}' not found. Install qemu-system-i386 (NOT x86_64)."
    except Exception as e:
        r.fail_reason = str(e)

    r.serial = "\n".join(lines)
    r.duration = time.time() - start
    return r


def save_result(r: Result):
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    today = datetime.now().strftime("%Y-%m-%d")
    (RESULTS_DIR / f"{today}_{r.name}_{r.status()}.log").write_text(r.serial)
    sf = RESULTS_DIR / f"{today}_summary.json"
    summary = json.loads(sf.read_text()) if sf.exists() else []
    summary.append({"test": r.name, "status": r.status(),
                     "duration_s": round(r.duration, 2), "fail_reason": r.fail_reason})
    sf.write_text(json.dumps(summary, indent=2))


def main():
    p = argparse.ArgumentParser(description="TM OS test runner (qemu-system-i386)")
    p.add_argument("--test"); p.add_argument("--all", action="store_true")
    p.add_argument("--list", action="store_true")
    p.add_argument("--timeout", type=int, default=30)
    p.add_argument("--verbose", "-v", action="store_true")
    args = p.parse_args()

    if args.list:
        print("Available tests:")
        for t in ALL_TESTS:
            print(f"  {t}" + (" [needs-disk]" if t in DISK_TESTS else ""))
        return

    tests = [args.test] if args.test else (ALL_TESTS if args.all else None)
    if not tests: p.print_help(); sys.exit(1)

    print(f"\n{'='*50}")
    print(f"TM OS TEST RUNNER  (qemu-system-i386, 32-bit)")
    print(f"Kernel: {KERNEL_ELF}")
    print(f"Tests: {len(tests)} | Timeout: {args.timeout}s")
    print(f"{'='*50}\n")

    results, passed = [], 0
    for name in tests:
        print(f"Running: {name}")
        r = run_test(name, args.timeout, args.verbose)
        results.append(r)
        save_result(r)
        sym = "✅" if r.passed else ("⏱️ " if r.timed_out else "❌")
        print(f"  {sym} {r.status()}  {name}  ({r.duration:.1f}s)")
        if not r.passed and r.fail_reason:
            print(f"     → {r.fail_reason}")
        if r.passed: passed += 1

    total = len(results)
    print(f"\n{'='*50}")
    print(f"RESULTS: {passed}/{total} passed")
    print(f"{'='*50}\n")
    sys.exit(0 if passed == total else 1)

if __name__ == "__main__":
    main()
