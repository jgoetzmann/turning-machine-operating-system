#!/usr/bin/env python3
"""
tools/c_smoke_tests.py

Build a few tiny freestanding 32-bit C programs and sanity-check their output
as ELF32 i386 binaries. This does NOT run them inside TMOS; it's a host-side
compiler/toolchain smoke test.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
C_TESTS_DIR = REPO_ROOT / "tests" / "c"
OUT_DIR = REPO_ROOT / "tests" / "results" / "c-build"


def which(*names: str) -> str | None:
    for n in names:
        p = shutil.which(n)
        if p:
            return p
    return None


def run(cmd: list[str], *, cwd: Path | None = None) -> None:
    subprocess.run(cmd, cwd=str(cwd) if cwd else None, check=True)


def detect_c_compiler(preferred: str | None) -> list[str]:
    """
    Returns argv prefix for compiling a freestanding 32-bit i386 ELF.

    Preference order:
    - user-specified --cc
    - i686-elf-gcc (matches repo userspace Makefile)
    - clang with -target i386-unknown-elf
    - gcc/cc with -m32 (may fail if multilib not installed)
    """
    if preferred:
        cc = which(preferred) or preferred
        return [cc]

    cc = which("i686-elf-gcc")
    if cc:
        return [cc]

    clang = which("clang")
    if clang:
        return [clang, "-target", "i386-unknown-elf"]

    gcc = which("gcc", "cc")
    if gcc:
        return [gcc, "-m32"]

    raise SystemExit("No suitable C compiler found (try i686-elf-gcc or clang).")


def compile_one(src: Path, out: Path, cc_prefix: list[str]) -> None:
    common = [
        "-O2",
        "-ffreestanding",
        "-fno-stack-protector",
        "-fno-pie",
        "-no-pie",
        "-nostdlib",
        "-nostdinc",
        "-static",
        "-Wl,--build-id=none",
    ]
    cmd = [*cc_prefix, *common, "-o", str(out), str(src)]
    run(cmd)


def ensure_elf32(path: Path) -> None:
    file_cmd = which("file")
    if not file_cmd:
        return
    out = subprocess.check_output([file_cmd, str(path)], text=True).strip()
    if "ELF 32-bit" not in out and "32-bit" not in out:
        raise SystemExit(f"Expected 32-bit ELF, got: {out}")


def main() -> None:
    p = argparse.ArgumentParser(description="Build tiny freestanding i386 C test programs")
    p.add_argument("--cc", help="C compiler to use (e.g. i686-elf-gcc or clang)")
    p.add_argument("--keep", action="store_true", help="Keep build outputs on success")
    args = p.parse_args()

    OUT_DIR.mkdir(parents=True, exist_ok=True)

    cc_prefix = detect_c_compiler(args.cc)
    print(f"[c-smoke] compiler: {' '.join(cc_prefix)}")

    sources = sorted(C_TESTS_DIR.glob("*.c"))
    if not sources:
        raise SystemExit(f"No C test sources found in {C_TESTS_DIR}")

    built: list[Path] = []
    for src in sources:
        out = OUT_DIR / (src.stem.upper() + ".ELF")
        print(f"[c-smoke] build: {src.relative_to(REPO_ROOT)} -> {out.relative_to(REPO_ROOT)}")
        compile_one(src, out, cc_prefix)
        ensure_elf32(out)
        built.append(out)

    print(f"[c-smoke] OK: built {len(built)} program(s)")

    if not args.keep:
        # Keep the directory but remove the binaries to reduce churn.
        for b in built:
            try:
                b.unlink()
            except FileNotFoundError:
                pass
        print("[c-smoke] cleaned binaries (use --keep to retain)")


if __name__ == "__main__":
    main()

