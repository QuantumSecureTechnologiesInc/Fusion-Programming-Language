#!/bin/bash
set -e

# Fusion Native Installer
# Strictly No Rust Dependencies
# Updated 2026-06-25: uses relative paths from script location.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo ">>> Building Fusion Native Runtime..."

# 1. Compile Native Runtime (C)
if command -v clang >/dev/null 2>&1; then
    CC="clang"
elif command -v gcc >/dev/null 2>&1; then
    CC="gcc"
elif command -v cl >/dev/null 2>&1; then
    CC="cl"
else
    echo "Error: C compiler (clang, gcc, or MSVC cl) required for runtime."
    exit 1
fi

mkdir -p bin

echo "Compiling runtime with $CC..."
if [ "$CC" = "cl" ]; then
    # MSVC path
    $CC /c runtime/native/fusionrt.c /Fo:bin/fusionrt.obj /I runtime/native
    lib /out:bin/fusionrt.lib bin/fusionrt.obj
    echo ">>> Runtime built at bin/fusionrt.lib"
else
    # GCC/Clang path
    $CC -c runtime/native/fusionrt.c -o runtime/native/fusionrt.o -I runtime/native
    ar rcs bin/libfusionrt.a runtime/native/fusionrt.o
    echo ">>> Runtime built at bin/libfusionrt.a"
fi

# 2. Check for existing compiler to bootstrap
if [ -f "bin/fuc.exe" ]; then
    COMPILER="bin/fuc.exe"
    echo "Using existing compiler: $COMPILER"
elif [ -f "bin/fuc" ]; then
    COMPILER="bin/fuc"
    echo "Using existing compiler: $COMPILER"
else
    echo "Msg: No pre-built compiler found at bin/fuc or bin/fuc.exe."
    echo "Assuming standard distribution or previous build."
    exit 1
fi

# 3. Compile Fusion Compiler (Self-Hosting)
echo ">>> Compiling Fusion Compiler (fuc)..."

$COMPILER crates/fuc/src/main.fu \
    --output bin/fuc_new.o \
    --lib
echo ">>> SUCCESS: New compiler built at bin/fuc_new.o"
echo ">>> Link manually with your runtime library to produce bin/fuc_new."
#!/bin/bash
set -e

# Fusion Native Installer
# Strictly No Rust Dependencies
# Updated 2026-06-25: uses relative paths from script location.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo ">>> Building Fusion Native Runtime..."

# 1. Compile Native Runtime (C)
if command -v clang >/dev/null 2>&1; then
    CC="clang"
elif command -v gcc >/dev/null 2>&1; then
    CC="gcc"
elif command -v cl >/dev/null 2>&1; then
    CC="cl"
else
    echo "Error: C compiler (clang, gcc, or MSVC cl) required for runtime."
    exit 1
fi

mkdir -p bin

echo "Compiling runtime with $CC..."
if [ "$CC" = "cl" ]; then
    # MSVC path
    $CC /c runtime/native/fusionrt.c /Fo:bin/fusionrt.obj /I runtime/native
    lib /out:bin/fusionrt.lib bin/fusionrt.obj
    echo ">>> Runtime built at bin/fusionrt.lib"
else
    # GCC/Clang path
    $CC -c runtime/native/fusionrt.c -o runtime/native/fusionrt.o -I runtime/native
    ar rcs bin/libfusionrt.a runtime/native/fusionrt.o
    echo ">>> Runtime built at bin/libfusionrt.a"
fi

# 2. Check for existing compiler to bootstrap
if [ -f "bin/fuc.exe" ]; then
    COMPILER="bin/fuc.exe"
    echo "Using existing compiler: $COMPILER"
elif [ -f "bin/fuc" ]; then
    COMPILER="bin/fuc"
    echo "Using existing compiler: $COMPILER"
else
    echo "Msg: No pre-built compiler found at bin/fuc or bin/fuc.exe."
    echo "Assuming standard distribution or previous build."
    exit 1
fi

# 3. Compile Fusion Compiler (Self-Hosting)
echo ">>> Compiling Fusion Compiler (fuc)..."

$COMPILER crates/fuc/src/main.fu \
    --output bin/fuc_new.o \
    --lib
echo ">>> SUCCESS: New compiler built at bin/fuc_new.o"
echo ">>> Link manually with your runtime library to produce bin/fuc_new."
#!/bin/bash
set -e

# Fusion Native Installer
# Strictly No Rust Dependencies
# Updated 2026-06-25: uses relative paths from script location.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo ">>> Building Fusion Native Runtime..."

# 1. Compile Native Runtime (C)
if command -v clang >/dev/null 2>&1; then
    CC="clang"
elif command -v gcc >/dev/null 2>&1; then
    CC="gcc"
elif command -v cl >/dev/null 2>&1; then
    CC="cl"
else
    echo "Error: C compiler (clang, gcc, or MSVC cl) required for runtime."
    exit 1
fi

mkdir -p bin

echo "Compiling runtime with $CC..."
if [ "$CC" = "cl" ]; then
    # MSVC path
    $CC /c runtime/native/fusionrt.c /Fo:bin/fusionrt.obj /I runtime/native
    lib /out:bin/fusionrt.lib bin/fusionrt.obj
    echo ">>> Runtime built at bin/fusionrt.lib"
else
    # GCC/Clang path
    $CC -c runtime/native/fusionrt.c -o runtime/native/fusionrt.o -I runtime/native
    ar rcs bin/libfusionrt.a runtime/native/fusionrt.o
    echo ">>> Runtime built at bin/libfusionrt.a"
fi

# 2. Check for existing compiler to bootstrap
if [ -f "bin/fuc.exe" ]; then
    COMPILER="bin/fuc.exe"
    echo "Using existing compiler: $COMPILER"
elif [ -f "bin/fuc" ]; then
    COMPILER="bin/fuc"
    echo "Using existing compiler: $COMPILER"
else
    echo "Msg: No pre-built compiler found at bin/fuc or bin/fuc.exe."
    echo "Assuming standard distribution or previous build."
    exit 1
fi

# 3. Compile Fusion Compiler (Self-Hosting)
echo ">>> Compiling Fusion Compiler (fuc)..."

$COMPILER crates/fuc/src/main.fu \
    --output bin/fuc_new.o \
    --lib
echo ">>> SUCCESS: New compiler built at bin/fuc_new.o"
echo ">>> Link manually with your runtime library to produce bin/fuc_new."
#!/bin/bash
set -e

# Fusion Native Installer
# Strictly No Rust Dependencies
# Updated 2026-06-25: uses relative paths from script location.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo ">>> Building Fusion Native Runtime..."

# 1. Compile Native Runtime (C)
if command -v clang >/dev/null 2>&1; then
    CC="clang"
elif command -v gcc >/dev/null 2>&1; then
    CC="gcc"
elif command -v cl >/dev/null 2>&1; then
    CC="cl"
else
    echo "Error: C compiler (clang, gcc, or MSVC cl) required for runtime."
    exit 1
fi

mkdir -p bin

echo "Compiling runtime with $CC..."
if [ "$CC" = "cl" ]; then
    # MSVC path
    $CC /c runtime/native/fusionrt.c /Fo:bin/fusionrt.obj /I runtime/native
    lib /out:bin/fusionrt.lib bin/fusionrt.obj
    echo ">>> Runtime built at bin/fusionrt.lib"
else
    # GCC/Clang path
    $CC -c runtime/native/fusionrt.c -o runtime/native/fusionrt.o -I runtime/native
    ar rcs bin/libfusionrt.a runtime/native/fusionrt.o
    echo ">>> Runtime built at bin/libfusionrt.a"
fi

# 2. Check for existing compiler to bootstrap
if [ -f "bin/fuc.exe" ]; then
    COMPILER="bin/fuc.exe"
    echo "Using existing compiler: $COMPILER"
elif [ -f "bin/fuc" ]; then
    COMPILER="bin/fuc"
    echo "Using existing compiler: $COMPILER"
else
    echo "Msg: No pre-built compiler found at bin/fuc or bin/fuc.exe."
    echo "Assuming standard distribution or previous build."
    exit 1
fi

# 3. Compile Fusion Compiler (Self-Hosting)
echo ">>> Compiling Fusion Compiler (fuc)..."

$COMPILER crates/fuc/src/main.fu \
    --output bin/fuc_new.o \
    --lib
echo ">>> SUCCESS: New compiler built at bin/fuc_new.o"
echo ">>> Link manually with your runtime library to produce bin/fuc_new."
#!/bin/bash
set -e

# Fusion Native Installer
# Strictly No Rust Dependencies

echo ">>> Building Fusion Native Runtime..."

# 1. Compile Native Runtime (C)
# Requires: gcc or clang
if command -v clang >/dev/null; then
    CC="clang"
elif command -v gcc >/dev/null; then
    CC="gcc"
else
    echo "Error: C compiler (gcc or clang) required for runtime."
    exit 1
fi

mkdir -p bin

echo "Compiling libfusionrt.a with $CC..."
$CC -c runtime/native/fusionrt.c -o runtime/native/fusionrt.o -I runtime/native
ar rcs bin/libfusionrt.a runtime/native/fusionrt.o

echo ">>> Runtime built at bin/libfusionrt.a"

# 2. Check for existing compiler to bootstrap
if [ -f "bin/fuc" ]; then
    COMPILER="bin/fuc"
    echo "Using existing compiler: $COMPILER"
else
    echo "Msg: No pre-built compiler found at bin/fuc."
    echo "Assuming standard distribution or previous build."
    # In a real scenario, we might download a stage0 binary here
    # For now, we assume the user has one if they want to self-host
    exit 1
fi

# 3. Compile Fusion Compiler (Self-Hosting)
# Compiling crates/fuc/src/main.fu -> bin/fuc_new
echo ">>> Compiling Fusion Compiler (fuc)..."

$COMPILER crates/fuc/src/main.fu \
    --output bin/fuc_new.o \
    --lib
echo ">>> SUCCESS: New compiler built at bin/fuc_new"
echo ">>> You can replace bin/fuc with bin/fuc_new to upgrade."
