#!/usr/bin/env python3

import mmap
import os
from typing import Optional

TAPE_SIZE = 65536
TAPE_PATH = "/tmp/turingos_tape.bin"
META_SIZE = 64
META_PATH = "/tmp/turingos_meta.bin"

_tape_file = None
_tape_map = None
_meta_file = None
_meta_map = None


def _resolved_tape_path() -> str:
    # macOS maps /tmp -> /private/tmp; realpath avoids path mismatch.
    return os.path.realpath(TAPE_PATH)


def _resolved_meta_path() -> str:
    # Keep path handling consistent with tape snapshot mapping.
    return os.path.realpath(META_PATH)


def _close_handles() -> None:
    global _tape_map, _tape_file, _meta_map, _meta_file
    if _tape_map is not None:
        _tape_map.close()
        _tape_map = None
    if _tape_file is not None:
        _tape_file.close()
        _tape_file = None
    if _meta_map is not None:
        _meta_map.close()
        _meta_map = None
    if _meta_file is not None:
        _meta_file.close()
        _meta_file = None


def _ensure_map() -> bool:
    global _tape_map, _tape_file
    path = _resolved_tape_path()

    if _tape_map is not None:
        return True
    try:
        if not os.path.exists(path):
            return False
        if os.path.getsize(path) < TAPE_SIZE:
            return False
        _tape_file = open(path, "rb")
    except OSError:
        return False
    try:
        _tape_map = mmap.mmap(_tape_file.fileno(), TAPE_SIZE, access=mmap.ACCESS_READ)
    except (OSError, ValueError):
        _close_handles()
        return False
    return True


def _ensure_meta_map() -> bool:
    global _meta_map, _meta_file
    path = _resolved_meta_path()

    if _meta_map is not None:
        return True
    try:
        if not os.path.exists(path):
            return False
        if os.path.getsize(path) < META_SIZE:
            return False
        _meta_file = open(path, "rb")
    except OSError:
        return False
    try:
        _meta_map = mmap.mmap(_meta_file.fileno(), META_SIZE, access=mmap.ACCESS_READ)
    except (OSError, ValueError):
        _close_handles()
        return False
    return True


def get_tape_map() -> Optional[mmap.mmap]:
    if not _ensure_map():
        return None
    return _tape_map


def get_tape() -> Optional[bytes]:
    tape_map = get_tape_map()
    if tape_map is None:
        return None
    try:
        tape_map.seek(0)
        return tape_map.read(TAPE_SIZE)
    except (ValueError, OSError):
        _close_handles()
        return None


def get_meta_map() -> Optional[mmap.mmap]:
    if not _ensure_meta_map():
        return None
    return _meta_map


def get_meta() -> Optional[dict]:
    meta_map = get_meta_map()
    if meta_map is None:
        return None

    try:
        meta_map.seek(0)
        raw = meta_map.read(META_SIZE)
    except (ValueError, OSError):
        _close_handles()
        return None

    return {
        "state": raw[0],
        "steps": int.from_bytes(raw[1:5], byteorder="little", signed=False),
        "dirty_map": raw[5:37],
        "pc": (raw[37] << 8) | raw[38],
    }


def get_pc() -> Optional[int]:
    meta = get_meta()
    if meta is None:
        return None
    return meta["pc"]


def get_state() -> Optional[int]:
    meta = get_meta()
    if meta is None:
        return None
    return meta["state"]


def close() -> None:
    _close_handles()
