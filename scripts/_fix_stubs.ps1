$ErrorActionPreference = 'Stop'
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$root = 'c:\Users\Matth\Downloads\Fusion v2.0 Vortex'

# 1. result.fu - Remove generic class Result<T,E> tail (lines 149-222)
$path = Join-Path $root 'stdlib\result.fu'
$lines = [System.IO.File]::ReadAllLines($path)
$keep = New-Object System.Collections.Generic.List[string]
foreach ($line in $lines) {
    if ($line -match '^// Result<T, E> - Rust-style result type') {
        $keep.Add('// NOTE: Generic Result<T, E> requires generics and first-class functions.')
        $keep.Add('// Use the concrete types above (ResultIntString, ResultBoolString, ResultStringString).')
        $keep.Add('// When the compiler gains generic support, a unified Result<T, E> can be added.')
        break
    }
    $keep.Add($line)
}
[System.IO.File]::WriteAllLines($path, $keep.ToArray(), $utf8NoBom)
Write-Host "result.fu: $($keep.Count) lines"

# 2. option.fu - Remove generic class Option<T> tail (lines 143-200)
$path = Join-Path $root 'stdlib\option.fu'
$lines = [System.IO.File]::ReadAllLines($path)
$keep = New-Object System.Collections.Generic.List[string]
foreach ($line in $lines) {
    if ($line -match '^// Option<T> - Rust-style optional value type') {
        $keep.Add('// NOTE: Generic Option<T> requires generics and first-class functions.')
        $keep.Add('// Use the concrete types above (OptionInt, OptionBool, OptionString).')
        $keep.Add('// When the compiler gains generic support, a unified Option<T> can be added.')
        break
    }
    $keep.Add($line)
}
[System.IO.File]::WriteAllLines($path, $keep.ToArray(), $utf8NoBom)
Write-Host "option.fu: $($keep.Count) lines"

# 3. iterator.fu - Remove generic Iterator trait + class section (lines 118-203)
$path = Join-Path $root 'stdlib\iterator.fu'
$lines = [System.IO.File]::ReadAllLines($path)
$keep = New-Object System.Collections.Generic.List[string]
foreach ($line in $lines) {
    if ($line -match '^// stdlib/iterator.fu - Iterator trait and utilities') {
        $keep.Add('// NOTE: Generic Iterator<T> trait and collect() require generics and first-class functions.')
        $keep.Add('// Use the concrete RangeIterator and ReverseRangeIterator above.')
        $keep.Add('// When the compiler gains generic support, a unified Iterator<T> trait can be added.')
        break
    }
    $keep.Add($line)
}
[System.IO.File]::WriteAllLines($path, $keep.ToArray(), $utf8NoBom)
Write-Host "iterator.fu: $($keep.Count) lines"

Write-Host "Done trimming generic class tails."
