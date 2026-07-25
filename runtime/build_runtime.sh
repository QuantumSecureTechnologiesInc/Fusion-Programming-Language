#!/usr/bin/env bash
# build_runtime.sh — Compile all Fusion C runtime source files into object files.
# Usage: bash build_runtime.sh [output_dir]
#
# On Windows (MSYS2/MinGW): the script detects the environment and adjusts flags.
# Output defaults to the current directory (or $1 if supplied).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="${1:-$SCRIPT_DIR}"

# Detect compiler
if command -v cl >/dev/null 2>&1; then
    # MSVC
    CC="cl"
    CFLAGS="/nologo /O2 /W3 /D_CRT_SECURE_NO_WARNINGS /DWIN32_LEAN_AND_MEAN"
    OUT_EXT=".obj"
elif command -v gcc >/dev/null 2>&1; then
    CC="gcc"
    CFLAGS="-O2 -Wall -Wextra -std=c11 -fPIC"
    OUT_EXT=".o"
elif command -v clang >/dev/null 2>&1; then
    CC="clang"
    CFLAGS="-O2 -Wall -Wextra -std=c11 -fPIC"
    OUT_EXT=".o"
elif command -v cc >/dev/null 2>&1; then
    CC="cc"
    CFLAGS="-O2 -Wall -Wextra -std=c11 -fPIC"
    OUT_EXT=".o"
else
    echo "error: no C compiler found (tried cl, gcc, clang, cc)" >&2
    exit 1
fi

echo "Compiler: $CC"
echo "Output:  $OUT_DIR"
echo ""

compile() {
    local src="$1"
    local base
    base="$(basename "$src" .c)"
    local out="$OUT_DIR/${base}${OUT_EXT}"
    echo "  CC  $src -> $(basename "$out")"
    $CC $CFLAGS -c "$src" -o "$out"
}

echo "=== Compiling core runtime ==="
compile "$SCRIPT_DIR/runtime.c"

echo ""
echo "=== Compiling native runtime (fusionrt) ==="
compile "$SCRIPT_DIR/native/fusionrt.c"

echo ""
echo "=== Compiling vector runtime ==="
compile "$SCRIPT_DIR/vector_runtime.c"

echo ""
echo "=== Compiling hashmap runtime ==="
compile "$SCRIPT_DIR/hashmap_runtime.c"

echo ""
echo "=== Compiling hashset runtime ==="
compile "$SCRIPT_DIR/hashset_runtime.c"

echo ""
echo "=== Done ==="
echo "Object files written to: $OUT_DIR"
ls -1 "$OUT_DIR"/*"$OUT_EXT" 2>/dev/null || true
