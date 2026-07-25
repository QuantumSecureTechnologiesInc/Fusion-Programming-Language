@echo off
REM build_runtime.bat — Compile all Fusion C runtime source files into object files.
REM Usage: build_runtime.bat [output_dir]
REM
REM Requires MSVC (cl.exe) or MinGW (gcc.exe) on PATH.
REM Output defaults to the current directory (or %1 if supplied).

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "OUT_DIR=%~1"
if "%OUT_DIR%"=="" set "OUT_DIR=%SCRIPT_DIR%"

REM Detect compiler
where cl >nul 2>&1
if %errorlevel% equ 0 (
    set "CC=cl"
    set "CFLAGS=/nologo /O2 /W3 /D_CRT_SECURE_NO_WARNINGS /DWIN32_LEAN_AND_MEAN"
    set "OUT_EXT=.obj"
    goto :found_compiler
)

where gcc >nul 2>&1
if %errorlevel% equ 0 (
    set "CC=gcc"
    set "CFLAGS=-O2 -Wall -Wextra -std=c11 -fPIC"
    set "OUT_EXT=.o"
    goto :found_compiler
)

where clang >nul 2>&1
if %errorlevel% equ 0 (
    set "CC=clang"
    set "CFLAGS=-O2 -Wall -Wextra -std=c11 -fPIC"
    set "OUT_EXT=.o"
    goto :found_compiler
)

echo error: no C compiler found (tried cl, gcc, clang)>&2
exit /b 1

:found_compiler
echo Compiler: %CC%
echo Output:   %OUT_DIR%
echo.

echo === Compiling core runtime ===
%CC% %CFLAGS% /c "%SCRIPT_DIR%runtime.c" /Fo"%OUT_DIR%\runtime%OUT_EXT%"
if %errorlevel% neq 0 (echo FAILED: runtime.c & exit /b 1)
echo   OK  runtime.c

echo.
echo === Compiling native runtime (fusionrt) ===
%CC% %CFLAGS% /c "%SCRIPT_DIR%native\fusionrt.c" /Fo"%OUT_DIR%\fusionrt%OUT_EXT%"
if %errorlevel% neq 0 (echo FAILED: native/fusionrt.c & exit /b 1)
echo   OK  native/fusionrt.c

echo.
echo === Compiling vector runtime ===
%CC% %CFLAGS% /c "%SCRIPT_DIR%vector_runtime.c" /Fo"%OUT_DIR%\vector_runtime%OUT_EXT%"
if %errorlevel% neq 0 (echo FAILED: vector_runtime.c & exit /b 1)
echo   OK  vector_runtime.c

echo.
echo === Compiling hashmap runtime ===
%CC% %CFLAGS% /c "%SCRIPT_DIR%hashmap_runtime.c" /Fo"%OUT_DIR%\hashmap_runtime%OUT_EXT%"
if %errorlevel% neq 0 (echo FAILED: hashmap_runtime.c & exit /b 1)
echo   OK  hashmap_runtime.c

echo.
echo === Compiling hashset runtime ===
%CC% %CFLAGS% /c "%SCRIPT_DIR%hashset_runtime.c" /Fo"%OUT_DIR%\hashset_runtime%OUT_EXT%"
if %errorlevel% neq 0 (echo FAILED: hashset_runtime.c & exit /b 1)
echo   OK  hashset_runtime.c

echo.
echo === Done ===
echo Object files written to: %OUT_DIR%
dir /b "%OUT_DIR%\*%OUT_EXT%" 2>nul

endlocal
