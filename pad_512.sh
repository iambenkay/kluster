#!/usr/bin/env bash
# Pad a file with NUL bytes until its size is a multiple of a block size.
# Usage: pad_512.sh <file>

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $(basename "$0") <file>" >&2
    exit 1
fi

file=$1
block=512

if [[ ! -f $file ]]; then
    echo "error: $file: not a regular file" >&2
    exit 1
fi

size=$(wc -c < "$file")
pad=$(( (block - size % block) % block ))

if (( pad == 0 )); then
    echo "$file already aligned to $block bytes ($size bytes)."
    exit 0
fi

head -c "$pad" /dev/zero >> "$file"

new_size=$(wc -c < "$file")
echo "Padded $file: $size -> $new_size bytes (+$pad NUL)."
