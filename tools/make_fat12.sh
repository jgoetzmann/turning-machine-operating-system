#!/usr/bin/env bash
# tools/make_fat12.sh — Create a 1.44MB FAT12 test disk image
# Requires: mkfs.fat (dosfstools) or mformat (mtools)
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMG="$REPO_ROOT/tests/fixtures/fat12.img"
MOUNT=$(mktemp -d)

mkdir -p "$(dirname "$IMG")"

echo "Creating 1.44MB FAT12 disk image at $IMG..."

# Create blank 1.44MB image (2880 sectors × 512 bytes)
dd if=/dev/zero of="$IMG" bs=512 count=2880 2>/dev/null

# Format as FAT12
if command -v mkfs.fat &>/dev/null; then
    mkfs.fat -F 12 -n "TMOSTAPE" "$IMG"
elif command -v mformat &>/dev/null; then
    mformat -i "$IMG" -f 1440 ::
else
    echo "ERROR: need mkfs.fat (dosfstools) or mformat (mtools)"
    exit 1
fi

# Mount and add test files
if command -v mcopy &>/dev/null; then
    # Use mtools (no mount needed)
    echo "HELLO TAPE" | mcopy -i "$IMG" - ::HELLO.TXT
    echo "TM OS shell" | mcopy -i "$IMG" - ::README.TXT
    echo "Added files via mtools"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    sudo mount -o loop "$IMG" "$MOUNT"
    echo "HELLO TAPE" > "$MOUNT/HELLO.TXT"
    echo "TM OS shell" > "$MOUNT/README.TXT"
    sudo umount "$MOUNT"
    echo "Added files via mount"
else
    echo "WARNING: Could not add files to disk image (no mtools or linux mount)."
    echo "Image created but may be empty."
fi

rmdir "$MOUNT" 2>/dev/null || true
echo "✅ FAT12 image created: $IMG ($(du -sh "$IMG" | cut -f1))"
