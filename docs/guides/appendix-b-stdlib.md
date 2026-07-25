> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Appendix B: Standard Library Reference

This appendix provides an overview of Fusion's standard library modules and commonly used types.

---

## Core Types

### Primitives

```fusion
// Integers (signed)
let a: i8 = 127
let b: i16 = 32_767
let c: i32 = 2_147_483_647
let d: i64 = 9_223_372_036_854_775_807  // Also `int`
let e: i128 = 170_141_183_460_469_231_731_687_303_715_884_105_727
let f: isize = 0  // Pointer-sized

// Integers (unsigned)
let a: u8 = 255
let b: u16 = 65_535
let c: u32 = 4_294_967_295
let d: u64 = 18_446_744_073_709_551_615
let e: u128 = 340_282_366_920_938_463_463_374_607_431_768_211_455
let f: usize = 0  // Pointer-sized

// Floating point
let x: f32 = 3.14159
let y: f64 = 3.141592653589793  // Also `float`

// Boolean
let t: bool = true
let f: bool = false

// Character
let c: char = 'A'
let emoji: char = '🦀'
```text

### String Types

```fusion
// String slice (borrowed)
let s: &str = "Hello, World!"

// Owned String
let s: String = String::from("Hello")
let s: String = "Hello".to_string()

// Common operations
s.len()                    // Length in bytes
s.chars().count()          // Length in characters
s.is_empty()               // Check if empty
s.contains("ello")         // Substring check
s.starts_with("He")        // Prefix check
s.ends_with("lo")          // Suffix check
s.to_uppercase()           // Convert case
s.to_lowercase()
s.trim()                   // Remove whitespace
s.split(" ")               // Split by delimiter
s.replace("l", "L")        // Replace substring
s.push_str(" World")       // Append (mut)
s.push('!')                // Append char (mut)
```text

---

## std::collections

### Vec<T>

```fusion
use std::collections::Vec

let mut v: Vec<int> = Vec::new()
let v = vec![1, 2, 3]

v.push(4)                  // Add to end
v.pop()                    // Remove from end
v.insert(0, 0)             // Insert at index
v.remove(1)                // Remove at index
v.len()                    // Number of elements
v.is_empty()               // Check if empty
v.first()                  // First element
v.last()                   // Last element
v.get(0)                   // Safe indexing
v[0]                       // Direct indexing
v.iter()                   // Immutable iterator
v.iter_mut()               // Mutable iterator
v.sort()                   // Sort in place
v.reverse()                // Reverse in place
v.contains(&2)             // Check membership
v.clear()                  // Remove all elements
```text

### HashMap<K, V>

```fusion
use std::collections::HashMap

let mut map: HashMap<String, int> = HashMap::new()

map.insert("key".to_string(), 42)
map.get("key")                    // Returns Option<&V>
map.get_mut("key")                // Returns Option<&mut V>
map.remove("key")                 // Remove and return
map.contains_key("key")           // Check key exists
map.len()                         // Number of entries
map.is_empty()                    // Check if empty
map.keys()                        // Iterator over keys
map.values()                      // Iterator over values
map.iter()                        // Iterator over (key, value)
map.entry("key").or_insert(0)     // Insert if missing

// From iterators
let map: HashMap<_, _> = vec![("a", 1), ("b", 2)].into_iter().collect()
```text

### HashSet<T>

```fusion
use std::collections::HashSet

let mut set: HashSet<int> = HashSet::new()

set.insert(1)                     // Add element
set.remove(&1)                    // Remove element
set.contains(&1)                  // Check membership
set.len()                         // Number of elements
set.is_empty()                    // Check if empty
set.iter()                        // Iterator

// Set operations
set1.union(&set2)                 // Elements in either
set1.intersection(&set2)          // Elements in both
set1.difference(&set2)            // Elements in set1 only
set1.symmetric_difference(&set2)  // Elements in one but not both
```text

### VecDeque<T>

```fusion
use std::collections::VecDeque

let mut deque: VecDeque<int> = VecDeque::new()

deque.push_back(1)                // Add to back
deque.push_front(0)               // Add to front
deque.pop_back()                  // Remove from back
deque.pop_front()                 // Remove from front
deque.front()                     // Peek front
deque.back()                      // Peek back
```text

### BinaryHeap<T>

```fusion
use std::collections::BinaryHeap

let mut heap: BinaryHeap<int> = BinaryHeap::new()

heap.push(3)                      // Add element
heap.pop()                        // Remove max
heap.peek()                       // View max
```text

---

## std::option

```fusion
enum Option<T> {
    Some(T),
    None,
}

let some: Option<int> = Some(5)
let none: Option<int> = None

// Methods
opt.is_some()                     // Check if Some
opt.is_none()                     // Check if None
opt.unwrap()                      // Extract or panic
opt.unwrap_or(default)            // Extract or use default
opt.unwrap_or_else(|| compute())  // Extract or compute default
opt.expect("error message")       // Extract or panic with message
opt.map(|x| x + 1)                // Transform value
opt.and_then(|x| Some(x + 1))     // Chain operations
opt.or(other_option)              // Use alternative if None
opt.ok_or(err)                    // Convert to Result
opt.as_ref()                      // Option<&T>
opt.as_mut()                      // Option<&mut T>
opt.take()                        // Take value, leave None
opt.replace(value)                // Replace value

// Pattern matching
if let Some(x) = opt { /* use x */ }
```text

---

## std::result

```fusion
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let ok: Result<int, String> = Ok(5)
let err: Result<int, String> = Err("error".to_string())

// Methods
res.is_ok()                       // Check if Ok
res.is_err()                      // Check if Err
res.unwrap()                      // Extract or panic
res.unwrap_err()                  // Extract error or panic
res.unwrap_or(default)            // Extract or use default
res.expect("message")             // Extract or panic with message
res.map(|x| x + 1)                // Transform Ok value
res.map_err(|e| format!("{}", e)) // Transform Err value
res.and_then(|x| Ok(x + 1))       // Chain operations
res.or(other_result)              // Use alternative if Err
res.ok()                          // Convert to Option<T>
res.err()                         // Convert to Option<E>

// Error propagation
let value = operation()?          // Return Err if failed
```text

---

## std::io

```fusion
use std::io::{self, Read, Write, BufRead}

// Standard streams
io::stdin()                       // Standard input
io::stdout()                      // Standard output
io::stderr()                      // Standard error

// Reading lines
let mut input = String::new()
io::stdin().read_line(&mut input)?

// Buffered reading
use std::io::BufReader
let reader = BufReader::new(file)
for line in reader.lines() {
    println("{}", line?)
}

// Writing
use std::io::BufWriter
let mut writer = BufWriter::new(file)
writer.write_all(b"Hello")?
writer.flush()?
```text

---

## std::fs

```fusion
use std::fs

// Read entire file
let content: String = fs::read_to_string("file.txt")?
let bytes: Vec<u8> = fs::read("file.bin")?

// Write entire file
fs::write("file.txt", "content")?
fs::write("file.bin", &bytes)?

// File operations
use std::fs::File
let file = File::open("file.txt")?         // Read mode
let file = File::create("file.txt")?       // Write mode (truncate)
let file = OpenOptions::new()
    .append(true)
    .open("file.txt")?

// Directory operations
fs::create_dir("path")?                    // Create directory
fs::create_dir_all("path/to/dir")?         // Create recursively
fs::remove_dir("path")?                    // Remove empty directory
fs::remove_dir_all("path")?                // Remove recursively

// File metadata
fs::metadata("file.txt")?                  // Get metadata
fs::remove_file("file.txt")?               // Delete file
fs::rename("old.txt", "new.txt")?          // Rename/move
fs::copy("src.txt", "dst.txt")?            // Copy file

// Directory iteration
for entry in fs::read_dir(".")? {
    let entry = entry?
    println("{:?}", entry.path())
}
```text

---

## std::path

```fusion
use std::path::{Path, PathBuf}

let path = Path::new("/home/user/file.txt")
let mut path_buf = PathBuf::from("/home/user")

path.exists()                     // Check if exists
path.is_file()                    // Check if file
path.is_dir()                     // Check if directory
path.parent()                     // Parent directory
path.file_name()                  // File name
path.file_stem()                  // File name without extension
path.extension()                  // File extension
path.join("subdir")               // Join paths
path.canonicalize()               // Absolute path

path_buf.push("file.txt")         // Append to path
path_buf.pop()                    // Remove last component
path_buf.set_extension("md")      // Change extension
```text

---

## std::thread

```fusion
use std::thread
use std::time::Duration

// Spawn thread
let handle = thread::spawn(|| {
    // Thread code
    42
})
let result = handle.join().unwrap()  // Wait and get result

// Sleep
thread::sleep(Duration::from_secs(1))

// Thread information
thread::current().name()
thread::current().id()

// Scoped threads
thread::scope(|s| {
    s.spawn(|| { /* access local variables */ })
    s.spawn(|| { /* access local variables */ })
})
```text

---

## std::sync

```fusion
use std::sync::{Arc, Mutex, RwLock, Condvar}
use std::sync::mpsc

// Atomic reference counting
let data = Arc::new(vec![1, 2, 3])
let data_clone = Arc::clone(&data)

// Mutex
let mutex = Mutex::new(0)
{
    let mut guard = mutex.lock().unwrap()
    *guard += 1
}

// RwLock
let lock = RwLock::new(0)
{
    let read_guard = lock.read().unwrap()  // Multiple readers
}
{
    let mut write_guard = lock.write().unwrap()  // Single writer
}

// Channels
let (tx, rx) = mpsc::channel()
tx.send(42).unwrap()
let value = rx.recv().unwrap()

// Condition variables
let pair = Arc::new((Mutex::new(false), Condvar::new()))
let (lock, cvar) = &*pair
cvar.notify_one()
cvar.wait(lock)
```text

---

## std::time

```fusion
use std::time::{Duration, Instant, SystemTime}

// Duration
let secs = Duration::from_secs(5)
let millis = Duration::from_millis(500)
let micros = Duration::from_micros(1000)
let nanos = Duration::from_nanos(1000000)

duration.as_secs()
duration.as_millis()
duration.as_micros()

// Instant (monotonic clock)
let start = Instant::now()
// ... work ...
let elapsed = start.elapsed()

// SystemTime (wall clock)
let now = SystemTime::now()
let since_epoch = now.duration_since(SystemTime::UNIX_EPOCH)?
```text

---

## std::env

```fusion
use std::env

// Environment variables
env::var("PATH")                  // Get variable (Result)
env::var_os("PATH")               // Get as OsString
env::set_var("KEY", "value")      // Set variable
env::remove_var("KEY")            // Remove variable
env::vars()                       // Iterate all

// Program arguments
env::args()                       // Iterator over arguments
env::args().collect::<Vec<_>>()   // As vector

// Working directory
env::current_dir()?               // Get current directory
env::set_current_dir("path")?     // Change directory

// Executable path
env::current_exe()?               // Path to current executable
```text

---

## std::process

```fusion
use std::process::{Command, exit}

// Run command
let output = Command::new("ls")
    .arg("-la")
    .current_dir("/")
    .output()?

println("stdout: {}", String::from_utf8_lossy(&output.stdout))
println("stderr: {}", String::from_utf8_lossy(&output.stderr))
println("status: {}", output.status)

// Exit program
exit(0)  // Success
exit(1)  // Error
```text

---

This appendix provides a quick reference for common standard library functionality. For complete documentation, run `fusion doc --std`.

[Back to Table of Contents](./README.md)