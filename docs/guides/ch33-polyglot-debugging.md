# Chapter 33: Foreign Stack Trace Decoder & Cross-Language Debugging

> Translating stack traces across language boundaries, multi-language debugger orchestration, memory address translation, common debugging patterns at FFI boundaries, and sanitizer/profiler tooling for polyglot systems

---

## The Foreign Stack Trace Problem

When a crash occurs in a polyglot system, the stack trace you get back is usually garbage — a tangled mess of addresses, mangled symbols, and half-translated exception messages from three different runtimes that don't know each other exists. Understanding why, and how to decode it, is the single most valuable debugging skill in polyglot development.

### Why Stack Traces Become Garbled Across Languages

Each language runtime maintains its own call stack representation. When you cross a language boundary via FFI, you are jumping from one runtime's stack frame convention into another's with no shared metadata:

```
# A typical garbled trace from a Fusion → Python → C chain:
Thread 1 "app" received signal SIGSEGV, Segmentation fault.
0x00007fff8a2c3f90 in ?? () from /usr/lib/x86_64-linux-gnu/libpython3.11.so.1.0
(gdb) bt
#0  0x00007fff8a2c3f90 in ?? () from /usr/lib/x86_64-linux-gnu/libpython3.11.so.1.0
#1  0x00005555555a1b23 in pyo3::ffi::PyObject_Call ()
#2  0x00005555555b8d10 in <my_crate::python_bridge::invoke as ...>::call ()
#3  0x000055555557f4a1 in my_crate::pipeline::process_batch ()
#4  0x00005555555601c4 in main ()
```

The frames marked `??` are the problem — the debugger has raw addresses but no symbols. Three things cause this:

1. **Separate symbol tables** — each compiled artifact (`.so`, `.dll`, `.pyd`) carries its own debug info in its own format. The debugger only loads symbols for the binary it was launched on.

2. **Stripped runtime frames** — Python, Java, and .NET interpreters use virtual machines with JIT compilation. The "real" call stack includes internal VM frames that the native debugger doesn't understand and can't symbolicate.

3. **Optimization reordering** — release builds inline and reorder code across language boundaries. A function call from Fusion into C++ might appear as a single frame, or vanish entirely.

### Mapping Errors Across Language Boundaries

The core technique is **boundary annotation**: every FFI function registers metadata that bridges the two worlds.

```fusion
// debug_boundary.fusion — annotated FFI boundary for trace mapping

struct FfiBoundaryMeta {
    fusion_fn: str,             // function name in Fusion
    foreign_fn: str,            // function name in target language
    foreign_source: ?str,       // source file path if known
    foreign_line: ?int,         // source line if known
    language: str,              // "python", "cpp", "rust", "java"
    direction: FfiDirection,    // inbound or outbound
}

enum FfiDirection {
    Inbound,   // foreign → Fusion
    Outbound,  // Fusion → foreign
}

struct BoundaryRegistry {
    boundaries: map<str, FfiBoundaryMeta>,
    next_id: int,
}

impl BoundaryRegistry {
    fn register(self, fusion_fn: str, foreign_fn: str, lang: str) -> int {
        let id = self.next_id;
        self.boundaries.put(id.to_string(), FfiBoundaryMeta {
            fusion_fn: fusion_fn,
            foreign_fn: foreign_fn,
            foreign_source: None,
            foreign_line: None,
            language: lang,
            direction: FfiDirection::Outbound,
        });
        self.next_id += 1;
        id
    }

    // Given a native address, check if it falls within a known FFI boundary
    fn resolve_address(self, addr: u64) -> ?FfiBoundaryMeta {
        for (_, meta) in self.boundaries.iter() {
            if self.addr_in_symbol(addr, &meta.foreign_fn, &meta.language) {
                return Some(meta.clone());
            }
        }
        None
    }

    fn addr_in_symbol(self, addr: u64, symbol: &str, lang: &str) -> bool {
        // Look up symbol bounds in the appropriate symbol table
        match lang {
            "cpp" | "rust" => self.lookup_elf_symbol(addr, symbol),
            "python" => self.lookup_python_frame(addr, symbol),
            "java" => self.lookup_jit_addr(addr, symbol),
            _ => false,
        }
    }

    fn lookup_elf_symbol(self, addr: u64, symbol: &str) -> bool {
        // Read /proc/self/maps + DWARF info to check address range
        let maps = std::fs::read_to_string("/proc/self/maps")
            .unwrap_or_default();
        for line in maps.lines() {
            // parse address range, check if addr falls within
            // the loaded segment containing this symbol
            if self.addr_in_range(addr, line) && self.symbol_in_segment(symbol, line) {
                return true;
            }
        }
        false
    }

    fn lookup_python_frame(self, addr: u64, symbol: &str) -> bool {
        // Python frames live in the interpreter's own frame chain
        // Cross-reference with sys._getframe() data
        false // placeholder — real impl uses cpython internals
    }

    fn lookup_jit_addr(self, addr: u64, symbol: &str) -> bool {
        // JIT-compiled code has no persistent symbol table
        // Must use JVMTI or perf maps for address mapping
        false // placeholder — real impl queries JVM perf map
    }
}

impl BoundaryRegistry {
    // Format a foreign stack trace into human-readable form
    fn decode_trace(self, raw_trace: &str) -> Vec<DecodedFrame> {
        let mut frames = Vec::new();
        for line in raw_trace.lines() {
            if let Some(addr) = self.parse_addr_from_line(line) {
                if let Some(meta) = self.resolve_address(addr) {
                    frames.push(DecodedFrame {
                        fusion_fn: meta.fusion_fn.clone(),
                        foreign_fn: meta.foreign_fn.clone(),
                        foreign_source: meta.foreign_source.clone(),
                        language: meta.language.clone(),
                        raw_line: line.to_string(),
                    });
                    continue;
                }
            }
            frames.push(DecodedFrame {
                fusion_fn: String::new(),
                foreign_fn: String::new(),
                foreign_source: None,
                language: "unknown".to_string(),
                raw_line: line.to_string(),
            });
        }
        frames
    }

    fn parse_addr_from_line(self, line: &str) -> ?u64 {
        // Extract hex address from trace line like "#0  0x00005555555a1b23 in ..."
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if part.starts_with("0x") {
                if let Ok(addr) = u64::from_str_radix(&part[2..], 16) {
                    return Some(addr);
                }
            }
        }
        None
    }
}

struct DecodedFrame {
    fusion_fn: str,
    foreign_fn: str,
    foreign_source: ?str,
    language: str,
    raw_line: str,
}

impl fmt::Display for DecodedFrame {
    fn fmt(self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.fusion_fn.is_empty() {
            write!(f, "[{}] {} ← Fusion:{}", self.language, self.foreign_fn, self.fusion_fn)
        } else if !self.foreign_fn.is_empty() {
            write!(f, "[{}] {}", self.language, self.foreign_fn)
        } else {
            write!(f, "{}", self.raw_line)
        }
    }
}
```

### C++ Exceptions → Python Exceptions Translation

When C++ throws through Python code (or vice versa), you get unhandled exceptions that crash the process because neither runtime knows how to catch the other's exception type.

```cpp
// cpp_exception_bridge.cpp — translate C++ exceptions to Python

#include <Python.h>
#include <stdexcept>
#include <string>
#include <exception>

struct CppExceptionInfo {
    const char* type_name;
    const char* message;
    const char* file;
    int line;
};

// Extract info from standard C++ exceptions
CppExceptionInfo extract_exception_info(const std::exception& e) {
    CppExceptionInfo info;
    info.type_name = typeid(e).name();

    // Try to get file/line from what() if formatted as "file:line: message"
    std::string what_str = e.what();
    size_t first_colon = what_str.find(':');
    size_t second_colon = what_str.find(':', first_colon + 1);

    if (first_colon != std::string::npos && second_colon != std::string::npos) {
        info.file = what_str.substr(0, first_colon).c_str();
        info.line = std::stoi(what_str.substr(first_colon + 1, second_colon - first_colon - 1));
        info.message = what_str.substr(second_colon + 2).c_str();
    } else {
        info.file = "unknown";
        info.line = 0;
        info.message = what_str.c_str();
    }
    return info;
}

// Catch C++ exceptions and convert to Python exceptions
extern "C" PyObject* cpp_try_invoke(PyObject* self, PyObject* args) {
    // Parse arguments: function pointer, args tuple
    // ... argument parsing omitted for brevity ...

    try {
        // Call the C++ function
        // result = call_cpp_function(func_ptr, args);
        Py_RETURN_NONE;
    } catch (const std::out_of_range& e) {
        auto info = extract_exception_info(e);
        PyErr_SetString(PyExc_IndexError, info.message);
        return NULL;
    } catch (const std::invalid_argument& e) {
        auto info = extract_exception_info(e);
        PyErr_SetString(PyExc_ValueError, info.message);
        return NULL;
    } catch (const std::runtime_error& e) {
        auto info = extract_exception_info(e);
        PyErr_SetString(PyExc_RuntimeError, info.message);
        return NULL;
    } catch (const std::exception& e) {
        auto info = extract_exception_info(e);
        // Create a custom Python exception with C++ context
        PyObject* cpp_exc = PyErr_NewExceptionWithDoc(
            "fusion.cpp_error",
            "Error originating from C++ code",
            NULL, NULL
        );
        PyObject* py_args = Py_BuildValue("(sss i)",
            info.type_name,
            info.message,
            info.file,
            info.line
        );
        PyErr_SetObject(cpp_exc, py_args);
        Py_DECREF(py_args);
        Py_DECREF(cpp_exc);
        return NULL;
    } catch (...) {
        PyErr_SetString(PyExc_RuntimeError, "Unknown C++ exception (non-std)");
        return NULL;
    }
}
```

```python
# Python side: catching translated C++ exceptions

import fusion._native  # the compiled C++ bridge

def call_cpp_pipeline(data):
    try:
        return fusion._native.cpp_try_invoke(data)
    except IndexError as e:
        # This was originally std::out_of_range from C++
        print(f"C++ out_of_range: {e}")
        raise
    except ValueError as e:
        # This was originally std::invalid_argument from C++
        print(f"C++ invalid_argument: {e}")
        raise
    except fusion.cpp_error as e:
        # Custom exception with full C++ context
        cpp_type, message, file, line = e.args
        print(f"C++ exception [{cpp_type}] at {file}:{line}: {message}")
        raise RuntimeError(f"C++ error: {message}") from e
```

### Rust Panics → Java Exceptions Translation

Rust panics cannot cross FFI boundaries safely. A panic unwinding through `extern "C"` is undefined behavior. The solution is to catch panics at the boundary and convert them to Java exceptions.

```rust
// rust_java_bridge.rs — catch Rust panics, emit Java exceptions

use std::panic;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Guard an FFI call so that Rust panics become Java exceptions
/// instead of crashing the JVM.
fn ffi_guard<F, T>(env: &JNIEnv, fallback: T, f: F) -> T
where
    F: FnOnce() -> T + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => result,
        Err(panic_payload) => {
            let message = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Rust panic with non-string payload".to_string()
            };

            // Also try to extract file/line from the panic location
            let location = panic::Location::caller();
            let full_msg = format!(
                "Rust panic at {}:{}: {}",
                location.file(), location.line(), message
            );

            let _ = env.throw_new("java/lang/RuntimeException", &full_msg);
            fallback
        }
    }
}

// Example FFI export
#[no_mangle]
pub extern "C" fn Java_com_example_RustBridge_processData(
    env: JNIEnv,
    _class: JClass,
    input: JString,
) -> JString {
    ffi_guard(&env, JString::default(), || {
        let input_str: String = env.get_string(input)
            .expect("couldn't get Java string")
            .into();

        // If this panics, it's caught above and becomes a Java RuntimeException
        let result = process_data_inner(&input_str)
            .expect("processing failed");

        env.new_string(result).expect("couldn't create Java string")
    })
}

fn process_data_inner(input: &str) -> Result<String, String> {
    // Any panic in here is caught by ffi_guard
    if input.is_empty() {
        return Err("empty input".to_string());
    }
    Ok(format!("processed: {}", input))
}
```

```java
// Java side: handling translated Rust panics

public class RustBridge {
    static { System.loadLibrary("rust_bridge"); }

    private static native String processData(String input);

    public String safeProcess(String input) {
        try {
            return processData(input);
        } catch (RuntimeException e) {
            String msg = e.getMessage();
            if (msg != null && msg.startsWith("Rust panic at")) {
                // Extract Rust panic location from the message
                // Format: "Rust panic at src/lib.rs:42: called `Option::unwrap()` on a `None`"
                System.err.println("Rust panic detected: " + msg);
                throw new ProcessingException("Rust processing failed", e);
            }
            throw e;
        }
    }
}
```

---

## Cross-Language Debuggers

### Setting Up gdb + Python Debugger Simultaneously

The key insight: run two debuggers on the same process using GDB's Python integration to bridge them.

```bash
# Launch GDB on the native executable, load Python extension
gdb --args ./my_app --config=polyglot.toml

# Inside GDB, load the Python debugger integration
(gdb) python
> import sys
> sys.path.insert(0, '/path/to/fusion/debug')
> from polyglot_debugger import PythonDebuggerBridge
> bridge = PythonDebuggerBridge(gdb)
> bridge.install()
> end

# Now set breakpoints that span both worlds
(gdb) break cpp_process_batch          # native function
(gdb) pybreak my_module.process_line   # Python function (via bridge)
(gdb) continue

# When a breakpoint hits, switch between debuggers
(gdb) pydo bridge.step_python()        # step in Python world
(gdb) step                            # step in C++ world
```

### LLDB for Native Code + Language Debugger

LLDB has first-class Python scriptability and can embed language-specific debug information.

```python
# lldb_polyglot_init.py — LLDB Python extension for polyglot debugging

import lldb

def __lldb_init_module(debugger, internal_dict):
    debugger.HandleCommand(
        'command script add -f lldb_polyglot_init.attach_python attach_python',
        debugger.GetInstanceName()
    )
    print("Polyglot debugging extension loaded. Use: attach_python <pid>")

def attach_python(debugger, command, exe_ctx, result, internal_dict):
    """Attach to a running process and enable cross-language debugging."""
    target = debugger.GetSelectedTarget()
    process = target.GetProcess()

    # Load all shared libraries into the symbol manager
    for module in target.GetModuleAtIndex(0).GetSharedModules():
        target.AddModule(module)

    # Set up Python script frame recognition
    # This lets LLDB understand Python interpreter frames
    script = """
   (lldb) script
    import sys
    # Register Python frame recognizer
    recognizer = lldb.PythonFrameRecognizer()
    recognizer.SetModuleName("python3")
    recognizer.SetFunctionName("PyEval_EvalFrameDefault")
    target.AddFrameRecognizer(recognizer)
    """
    debugger.HandleCommand(script)

    result.AppendMessage("Attached. Cross-language frames enabled.")
```

### VS Code Multi-Language Debugging Configurations

VS Code can launch multiple debuggers in parallel and synchronize breakpoints through the Debug Adapter Protocol.

```jsonc
// .vscode/launch.json — polyglot debugging configuration
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug Native (C++)",
            "type": "cppdbg",
            "request": "launch",
            "program": "${workspaceFolder}/build/app",
            "args": ["--config=debug.toml"],
            "cwd": "${workspaceFolder}",
            "MIMode": "gdb",
            "setupCommands": [
                {
                    "description": "Enable pretty-printing",
                    "text": "-enable-pretty-printing",
                    "ignoreFailures": true
                }
            ],
            "preLaunchTask": "build-native"
        },
        {
            "name": "Debug Python Extension",
            "type": "debugpy",
            "request": "launch",
            "module": "my_python_module",
            "cwd": "${workspaceFolder}",
            "justMyCode": false,
            "subProcess": true,
            "env": {
                "PYTHONPATH": "${workspaceFolder}/python",
                "PYDEVD_USE_FRAME_EVAL": "NO"
            }
        },
        {
            "name": "Attach to Running Process",
            "type": "composite",
            "configurations": [
                {
                    "name": "Native Layer",
                    "type": "cppdbg",
                    "request": "attach",
                    "processId": "${command:pickProcess}"
                },
                {
                    "name": "Python Layer",
                    "type": "debugpy",
                    "request": "attach",
                    "connect": {
                        "host": "localhost",
                        "port": 5678
                    }
                }
            ]
        }
    ],
    "compounds": [
        {
            "name": "Full Polyglot Debug",
            "configurations": ["Debug Native (C++)", "Debug Python Extension"],
            "stopAll": false,
            "preLaunchTask": "start-python-debug-server"
        }
    ]
}
```

### IntelliJ Polyglot Debugging Setup

IntelliJ's multi-language debugger can handle JVM languages and native code simultaneously.

```kotlin
// IntelliJ run configuration for polyglot debugging
// Place in .run/Polyglot_Debug.run.xml

/*
<component name="ProjectRunConfigurationManager">
  <configuration default="false" name="Polyglot Debug"
                  type="CompositeRunConfigurationType">
    <toRun name="Fusion Process" type="Application">
      <option name="MAIN_CLASS_NAME" value="com.fusion.MainKt" />
      <option name="VM_PARAMETERS" value="-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005" />
      <option name="PROGRAM_PARAMETERS" value="--config=debug.toml" />
    </toRun>
    <toRun name="Native GDB" type="GDB">
      <option name="BINARY_PATH" value="$PROJECT_DIR$/build/native/app" />
      <option name="GDB_PATH" value="gdb" />
      <option name="ATTACH" value="true" />
      <option name="PROCESS_ID" value="auto" />
    </toRun>
    <method v="2" />
  </configuration>
</component>
*/

// IntelliJ breakpoint synchronization via custom plugin
class PolyglotBreakpointSynchronizer {
    fun onNativeBreakpointHit(address: Long, moduleName: String) {
        // Find corresponding Fusion source location
        val mapping = SymbolRegistry.resolveNativeToSource(address, moduleName)
        if (mapping != null) {
            val fusionFile = LocalFileSystem.getInstance()
                .findFileByPath(mapping.sourceFile)
            val lineMarker = XLineBreakpointUtil.findBreakpoint(
                mapping.sourceFile, mapping.lineNumber
            )
            // Notify Java debugger of the equivalent position
            XDebugProcessStarter.instance().session
                .breakpointReached(lineMarker, mapping.sourceFile)
        }
    }

    fun onJavaBreakpointHit(className: String, lineNumber: Int) {
        // Check if this class has a native backing implementation
        val nativeMapping = SymbolRegistry.resolveJavaToNative(className, lineNumber)
        if (nativeMapping != null) {
            // Break in the native debugger too
            NativeDebugger.instance().setTempBreakpoint(
                nativeMapping.address, nativeMapping.module
            )
        }
    }
}
```

---

## Memory Address Translation

### Translating Memory Addresses to Source Code

The fundamental challenge: a crash gives you a memory address, and you need to walk back to source code. This requires a chain of lookups across multiple symbol formats.

```fusion
// address_translator.fusion — translate native addresses to source

struct AddressTranslator {
    loaded_modules: Vec<LoadedModule>,
    symbol_cache: map<u64, ResolvedSymbol>,
}

struct LoadedModule {
    name: str,
    base_address: u64,
    size: u64,
    symbol_path: ?str,
    format: SymbolFormat,
}

enum SymbolFormat {
    Dwarf,    // Linux/macOS — .debug_* sections in ELF/Mach-O
    Pdb,      // Windows — .pdb files
    Djwindbg, // Android — .so with unwind tables
}

struct ResolvedSymbol {
    module: str,
    offset_in_module: u64,
    function_name: str,
    source_file: ?str,
    source_line: ?int,
    inlined_from: Vec<InlinedFrame>,
}

struct InlinedFrame {
    function_name: str,
    source_file: ?str,
    source_line: ?int,
}

impl AddressTranslator {
    fn new() -> Self {
        let modules = Self::enumerate_loaded_modules();
        AddressTranslator {
            loaded_modules: modules,
            symbol_cache: map::new(),
        }
    }

    fn translate(self, address: u64) -> ?ResolvedSymbol {
        // Check cache first
        if let Some(cached) = self.symbol_cache.get(&address) {
            return Some(cached.clone());
        }

        // Find which module contains this address
        let module = self.loaded_modules.iter()
            .find(|m| address >= m.base_address && address < m.base_address + m.size)?;

        let offset = address - module.base_address;

        let symbol = match module.format {
            SymbolFormat::Dwarf => self.resolve_dwarf(module, offset)?,
            SymbolFormat::Pdb => self.resolve_pdb(module, offset)?,
            SymbolFormat::Djwindbg => self.resolve_djwindbg(module, offset)?,
        };

        self.symbol_cache.insert(address, symbol.clone());
        Some(symbol)
    }

    fn resolve_dwarf(self, module: &LoadedModule, offset: u64) -> ?ResolvedSymbol {
        // DWARF resolution: read .debug_info, .debug_line, .debug_abbrev
        // from the ELF/Mach-O file at module.symbol_path
        let path = module.symbol_path.as_ref()?;
        let file = std::fs::File::open(path).ok()?;
        let mmap = unsafe { memmap2::Mmap::map(&file).ok()? };

        // Parse ELF header to find DWARF sections
        let elf = goblin::elf::Elf::parse(&mmap).ok()?;
        let dwarf = gimli::DwarfReader::from_section(&elf, &mmap)?;

        // Walk .debug_line to find source file + line for this offset
        let line_program = dwarf.line_program(offset)?;
        let (file, line) = line_program.resolve(offset)?;

        // Check for inlined functions in .debug_info
        let inlined = dwarf.inlined_functions(offset)?;

        Some(ResolvedSymbol {
            module: module.name.clone(),
            offset_in_module: offset,
            function_name: dwarf.function_name(offset).unwrap_or("??"),
            source_file: Some(file),
            source_line: Some(line),
            inlined_from: inlined,
        })
    }

    fn resolve_pdb(self, module: &LoadedModule, offset: u64) -> ?ResolvedSymbol {
        // PDB resolution via Windows Debug Help Library or pdb crate
        let path = module.symbol_path.as_ref()?;
        let pdb = pdb::PDBInformation::open(path).ok()?;
        let symbols = pdb.symbol_table()?;
        let symbol = symbols.find_offset(offset)?;

        Some(ResolvedSymbol {
            module: module.name.clone(),
            offset_in_module: offset,
            function_name: symbol.name().to_string(),
            source_file: symbol.source_file().ok().map(|f| f.to_string()),
            source_line: symbol.line().ok(),
            inlined_from: Vec::new(),
        })
    }

    fn resolve_djwindbg(self, module: &LoadedModule, offset: u64) -> ?ResolvedSymbol {
        // Android .so with compact unwind tables
        let path = module.symbol_path.as_ref()?;
        let file = std::fs::File::open(path).ok()?;
        let mmap = unsafe { memmap2::Mmap::map(&file).ok()? };

        let elf = goblin::elf::Elf::parse(&mmap).ok()?;
        let unwind_info = elf.find_unwind_entry(offset)?;

        Some(ResolvedSymbol {
            module: module.name.clone(),
            offset_in_module: offset,
            function_name: unwind_info.function_name,
            source_file: None, // Android .so often lacks line info
            source_line: None,
            inlined_from: Vec::new(),
        })
    }

    fn enumerate_loaded_modules() -> Vec<LoadedModule> {
        let mut modules = Vec::new();

        // Linux: read /proc/self/maps
        #[cfg(target_os = "linux")]
        {
            let maps = std::fs::read_to_string("/proc/self/maps").unwrap_or_default();
            for line in maps.lines() {
                if let Some(module) = Self::parse_linux_map_line(line) {
                    modules.push(module);
                }
            }
        }

        // macOS: read dyld shared cache or use dyld API
        #[cfg(target_os = "macos")]
        {
            // Use _dyld_image_count() and _dyld_get_image_header()
        }

        // Windows: use EnumProcessModules
        #[cfg(target_os = "windows")]
        {
            // Use PSAPI EnumProcessModules
        }

        modules
    }

    #[cfg(target_os = "linux")]
    fn parse_linux_map_line(line: &str) -> ?LoadedModule {
        // Format: 7fff8a2c0000-7fff8a2e0000 r-xp 00000000 08:01 12345 /path/to/lib.so
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }

        let addr_range: Vec<&str> = parts[0].split('-').collect();
        if addr_range.len() != 2 {
            return None;
        }

        let base = u64::from_str_radix(addr_range[0], 16).ok()?;
        let end = u64::from_str_radix(addr_range[1], 16).ok()?;
        let path = parts[5];

        if !std::path::Path::new(path).exists() {
            return None;
        }

        let format = if path.ends_with(".pdb") || path.contains(".pdb/") {
            SymbolFormat::Pdb
        } else {
            SymbolFormat::Dwarf
        };

        Some(LoadedModule {
            name: std::path::Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            base_address: base,
            size: end - base,
            symbol_path: Some(path.to_string()),
            format: format,
        })
    }
}
```

### Debug Symbol Formats: DWARF vs PDB

| Aspect | DWARF | PDB |
|--------|-------|-----|
| **Platform** | Linux, macOS, BSD | Windows (MSVC) |
| **Location** | Embedded in ELF/Mach-O `.debug_*` sections | Separate `.pdb` file |
| **Toolchain** | GCC, Clang, Rust (`-C debug-info`) | MSVC (`/Zi`, `/ZI`) |
| **Line Tables** | `.debug_line` section | `DBI` stream in PDB |
| **Type Info** | `.debug_info` + `.debug_abbrev` | TPI stream |
| **Inline Frames** | `.debug_info` `DW_TAG_inlined_subroutine` | `Inlinee` stream |
| **Unwind Info** | `.eh_frame` + `.debug_frame` | `.pdata` section |

```bash
# Inspecting DWARF symbols
readelf --debug-dump=info libmy_module.so    # Type and variable info
readelf --debug-dump=line libmy_module.so    # Source line mappings
readelf --debug-dump=loc libmy_module.so     # Location expressions
addr2line -e libmy_module.so 0x12345         # Quick address → source lookup
llvm-dwarfdump --lookup=0x12345 libmy_module.so  # Comprehensive dump

# Inspecting PDB symbols (Windows)
cvdump.pdb my_module.pdb                     # Microsoft's PDB dumper
llvm-pdbutil dump --all my_module.pdb        # LLVM's PDB tools
llvm-pdbutil line --lookup=0x12345 my_module.pdb  # Line number lookup
```

---

## Common Debugging Patterns

### FFI Crash Diagnosis

Most FFI crashes fall into one of a small number of categories. Learning to recognize the pattern saves hours.

```
Pattern: Segfault immediately after FFI call
├── Cause: Wrong calling convention (cdecl vs stdcall vs fastcall)
├── Diagnosis: Check function signature matches between caller and callee
├── Fix: Ensure extern declarations match exactly
└── Prevention: Use header files / bindgen to auto-generate signatures

Pattern: Segfault after returning from FFI
├── Cause: Stack corruption — wrong argument count or types
├── Diagnosis: Valgrind/ASan to find the corruption origin
├── Fix: Correct argument passing at the boundary
└── Prevention: FFI wrapper codegen with verified signatures

Pattern: Heap corruption detected later
├── Cause: Double-free across language boundaries
├── Diagnosis: AddressSanitizer with cross-language tracking
├── Fix: Single ownership model for cross-language objects
└── Prevention: Shared allocator with reference counting

Pattern: Crash only in release builds
├── Cause: Optimization eliminates null checks or inlines across boundaries
├── Diagnosis: Reproduce with -O1 instead of -O3, bisect optimization level
├── Fix: Add volatile or prevent inlining at boundary
└── Prevention: Boundary functions marked #[no_inline] / __attribute__((noinline))
```

### Memory Corruption at Boundaries

```cpp
// boundary_corruption_detector.cpp — detect common FFI memory bugs

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <atomic>

// Canaries placed around FFI-allocated memory
static constexpr uint64_t ALLOC_CANARY = 0xDEADBEEF_CAFEBABE;
static constexpr uint64_t FREE_CANARY  = 0xFREEFREF_FREFRFE;

struct TrackedAllocation {
    uint64_t canary_head;
    size_t size;
    const char* allocator_name;  // which language allocated this
    uint32_t sequence;
    uint64_t canary_tail;
};

static std::atomic<uint32_t> alloc_sequence{0};

extern "C" void* tracked_alloc(size_t size, const char* allocator) {
    size_t total = sizeof(TrackedAllocation) + size + sizeof(uint64_t);
    TrackedAllocation* block = (TrackedAllocation*)malloc(total);

    block->canary_head = ALLOC_CANARY;
    block->size = size;
    block->allocator_name = allocator;
    block->sequence = alloc_sequence.fetch_add(1);

    uint64_t* tail = (uint64_t*)((char*)block + sizeof(TrackedAllocation) + size);
    *tail = ALLOC_CANARY;

    return (char*)block + sizeof(TrackedAllocation);
}

extern "C" void tracked_free(void* ptr, const char* freed_by) {
    if (!ptr) return;

    TrackedAllocation* block = (TrackedAllocation*)((char*)ptr - sizeof(TrackedAllocation));

    // Check head canary
    if (block->canary_head != ALLOC_CANARY) {
        fprintf(stderr,
            "FFI BUG: Head canary corrupted!\n"
            "  Allocated by: %s (seq=%u)\n"
            "  Freed by: %s\n"
            "  Expected: 0x%016lx, Got: 0x%016lx\n",
            block->allocator_name, block->sequence,
            freed_by,
            ALLOC_CANARY, block->canary_head
        );
        abort();
    }

    // Check tail canary
    uint64_t* tail = (uint64_t*)((char*)block + sizeof(TrackedAllocation) + block->size);
    if (*tail != ALLOC_CANARY) {
        fprintf(stderr,
            "FFI BUG: Tail canary corrupted (buffer overflow)!\n"
            "  Allocated by: %s (seq=%u), size=%zu\n"
            "  Freed by: %s\n"
            "  Expected: 0x%016lx, Got: 0x%016lx\n",
            block->allocator_name, block->sequence, block->size,
            freed_by,
            ALLOC_CANARY, *tail
        );
        abort();
    }

    // Check cross-language free
    if (strcmp(block->allocator_name, freed_by) != 0) {
        fprintf(stderr,
            "FFI WARNING: Cross-language free detected\n"
            "  Allocated by: %s (seq=%u)\n"
            "  Freed by: %s\n"
            "  Size: %zu bytes\n",
            block->allocator_name, block->sequence,
            freed_by, block->size
        );
        // Not fatal — but suspicious. In production, track this metric.
    }

    free(block);
}
```

### Data Serialization/Deserialization Bugs

```fusion
// serialization_debug.fusion — detect silent data corruption

struct SerializableDebugContext {
    original_bytes: Vec<u8>,
    roundtrip_bytes: Vec<u8>,
    field_path: str,
    language_source: str,
}

impl SerializableDebugContext {
    fn verify_roundtrip<T: Serialize + Deserialize + PartialEq>(
        self,
        value: T,
        label: str,
    ) -> Result<(), SerializationBug> {
        // Step 1: Serialize
        let bytes = bincode::serialize(&value)
            .map_err(|e| SerializationBug::SerializeFailed {
                field: self.field_path.clone(),
                error: e.to_string(),
            })?;

        // Step 2: Deserialize
        let recovered: T = bincode::deserialize(&bytes)
            .map_err(|e| SerializationBug::DeserializeFailed {
                field: self.field_path.clone(),
                error: e.to_string(),
            })?;

        // Step 3: Compare
        if value != recovered {
            return Err(SerializationBug::RoundtripMismatch {
                field: self.field_path.clone(),
                original: format!("{:?}", value),
                recovered: format!("{:?}", recovered),
                bytes: hex::encode(&bytes),
            });
        }

        Ok(())
    }

    // Detect endianness mismatches across languages
    fn check_endianness(self, field_name: str, raw_bytes: &[u8]) -> ?EndiannessMismatch {
        // Read as both little-endian and big-endian, compare interpretations
        if raw_bytes.len() < 8 {
            return None;
        }

        let as_le = u64::from_le_bytes(raw_bytes[0..8].try_into().ok()?);
        let as_be = u64::from_be_bytes(raw_bytes[0..8].try_into().ok()?);

        if as_le == as_be {
            return None; // Symmetric value, can't detect
        }

        // Check if the platform's native endianness matches what was serialized
        let native_is_le = cfg!(target_endian = "little");
        Some(EndiannessMismatch {
            field: field_name,
            raw_bytes: raw_bytes.to_vec(),
            interpreted_as_le: as_le,
            interpreted_as_be: as_be,
            platform_is_little_endian: native_is_le,
        })
    }
}

enum SerializationBug {
    SerializeFailed { field: str, error: str },
    DeserializeFailed { field: str, error: str },
    RoundtripMismatch { field: str, original: str, recovered: str, bytes: str },
}

struct EndiannessMismatch {
    field: str,
    raw_bytes: Vec<u8>,
    interpreted_as_le: u64,
    interpreted_as_be: u64,
    platform_is_little_endian: bool,
}
```

### Type Mismatch Detection

```python
# type_boundary_detector.py — detect type mismatches at FFI boundaries

import ctypes
import struct
import sys
from dataclasses import dataclass
from typing import Any, Dict, List, Optional

@dataclass
class TypeMismatch:
    boundary: str          # which FFI call
    fusion_type: str       # type Fusion expects
    python_type: str       # type Python provides
    value_repr: str        # actual value
    expected_repr: str     # expected value representation

class FFITypeChecker:
    """Detect type mismatches before they cause silent corruption."""

    # Map Fusion types to expected C/Python types
    TYPE_MAP = {
        'i8':   {'c_type': ctypes.c_int8,   'python': int, 'range': (-128, 127)},
        'i16':  {'c_type': ctypes.c_int16,  'python': int, 'range': (-32768, 32767)},
        'i32':  {'c_type': ctypes.c_int32,  'python': int, 'range': (-2**31, 2**31-1)},
        'i64':  {'c_type': ctypes.c_int64,  'python': int, 'range': (-2**63, 2**63-1)},
        'u8':   {'c_type': ctypes.c_uint8,  'python': int, 'range': (0, 255)},
        'u16':  {'c_type': ctypes.c_uint16, 'python': int, 'range': (0, 65535)},
        'u32':  {'c_type': ctypes.c_uint32, 'python': int, 'range': (0, 2**32-1)},
        'u64':  {'c_type': ctypes.c_uint64, 'python': int, 'range': (0, 2**64-1)},
        'f32':  {'c_type': ctypes.c_float,  'python': (int, float)},
        'f64':  {'c_type': ctypes.c_double, 'python': (int, float)},
        'bool': {'c_type': ctypes.c_bool,   'python': bool},
        'str':  {'c_type': ctypes.c_char_p, 'python': str},
    }

    def __init__(self, strict: bool = True):
        self.strict = strict
        self.mismatches: List[TypeMismatch] = []

    def check_argument(
        self,
        boundary_name: str,
        fusion_type: str,
        python_value: Any,
        param_name: str,
    ) -> Optional[TypeMismatch]:
        """Validate a single argument before passing across the FFI boundary."""
        spec = self.TYPE_MAP.get(fusion_type)
        if spec is None:
            return None  # Opaque type, can't check

        # Check Python type
        expected_python = spec['python']
        if not isinstance(python_value, expected_python):
            mismatch = TypeMismatch(
                boundary=boundary_name,
                fusion_type=fusion_type,
                python_type=type(python_value).__name__,
                value_repr=repr(python_value),
                expected_repr=f"expected {expected_python}",
            )
            self.mismatches.append(mismatch)
            return mismatch

        # Check value range for integer types
        if 'range' in spec and isinstance(python_value, int):
            lo, hi = spec['range']
            if not (lo <= python_value <= hi):
                mismatch = TypeMismatch(
                    boundary=boundary_name,
                    fusion_type=fusion_type,
                    python_type=f"{type(python_value).__name__} (out of range)",
                    value_repr=repr(python_value),
                    expected_repr=f"range [{lo}, {hi}]",
                )
                self.mismatches.append(mismatch)
                return mismatch

        # Check for NaN/Inf in float types
        if fusion_type in ('f32', 'f64') and isinstance(python_value, float):
            import math
            if math.isnan(python_value):
                mismatch = TypeMismatch(
                    boundary=boundary_name,
                    fusion_type=fusion_type,
                    python_type="float (NaN)",
                    value_repr="NaN",
                    expected_repr="valid float",
                )
                self.mismatches.append(mismatch)
                return mismatch

        return None

    def check_buffer(
        self,
        boundary_name: str,
        buffer: bytes,
        expected_element_type: str,
        expected_count: int,
    ) -> Optional[TypeMismatch]:
        """Validate a buffer before passing across FFI."""
        spec = self.TYPE_MAP.get(expected_element_type)
        if spec is None:
            return None

        c_type_size = ctypes.sizeof(spec['c_type'])
        actual_count = len(buffer) // c_type_size

        if len(buffer) % c_type_size != 0:
            return TypeMismatch(
                boundary=boundary_name,
                fusion_type=f"{expected_element_type}[{expected_count}]",
                python_type=f"bytes (misaligned, {len(buffer)} bytes)",
                value_repr=f"{len(buffer)} bytes",
                expected_repr=f"{expected_count} × {c_type_size} = {expected_count * c_type_size} bytes",
            )

        if actual_count != expected_count:
            return TypeMismatch(
                boundary=boundary_name,
                fusion_type=f"{expected_element_type}[{expected_count}]",
                python_type=f"bytes ({actual_count} elements)",
                value_repr=f"{actual_count} elements",
                expected_repr=f"{expected_count} elements",
            )

        return None

    def report(self) -> str:
        if not self.mismatches:
            return "No type mismatches detected."

        lines = ["Type Mismatches Detected:"]
        for i, m in enumerate(self.mismatches, 1):
            lines.append(f"  {i}. In boundary '{m.boundary}':")
            lines.append(f"     Fusion expects: {m.fusion_type}")
            lines.append(f"     Python provides: {m.python_type}")
            lines.append(f"     Value: {m.value_repr}")
            lines.append(f"     {m.expected_repr}")
        return "\n".join(lines)


# Usage example
def safe_ffi_call():
    checker = FFITypeChecker(strict=True)

    # Check arguments before the call
    checker.check_argument("process_batch", "i64", 42, "count")
    checker.check_argument("process_batch", "str", b"hello", "name")  # BUG: bytes, not str
    checker.check_argument("process_batch", "f32", float('inf'), "threshold")  # BUG: Inf

    checker.check_buffer("copy_data", b"\x01\x02\x03", "u32", 1)  # BUG: wrong size

    if checker.mismatches:
        print(checker.report())
        sys.exit(1)

    # All checks passed, safe to call FFI
    # result = native_lib.process_batch(count, name, threshold)
```

### Race Conditions Across Languages

```fusion
// polyglot_race_detector.fusion — detect data races across language boundaries

struct RaceDetector {
    // Track which thread in which language last touched each address
    access_log: map<u64, AccessRecord>,
    mutex: Mutex,
    violations: Vec<RaceViolation>,
}

struct AccessRecord {
    address: u64,
    thread_id: u64,
    language: str,
    function: str,
    access_type: AccessType,
    timestamp: u64,
}

enum AccessType {
    Read,
    Write,
    ReadWrite,
}

struct RaceViolation {
    address: u64,
    first_access: AccessRecord,
    second_access: AccessRecord,
    time_gap_us: u64,
}

impl RaceDetector {
    fn new() -> Self {
        RaceDetector {
            access_log: map::new(),
            mutex: Mutex::new(()),
            violations: Vec::new(),
        }
    }

    // Called by instrumented FFI code before every memory access
    fn record_access(
        self,
        address: u64,
        thread_id: u64,
        language: str,
        function: str,
        access_type: AccessType,
    ) {
        let now = self.timestamp_us();
        let _lock = self.mutex.lock();

        if let Some(prev) = self.access_log.get(&address) {
            // Same thread is fine
            if prev.thread_id != thread_id {
                // Different threads — check if this is a race
                let both_write = matches!(prev.access_type, AccessType::Write | AccessType::ReadWrite)
                    && matches!(access_type, AccessType::Write | AccessType::ReadWrite);

                let one_write = matches!(prev.access_type, AccessType::Write | AccessType::ReadWrite)
                    || matches!(access_type, AccessType::Write | AccessType::ReadWrite);

                if both_write || one_write {
                    // Potential race! But check if there was a synchronization between them
                    let time_gap = now.saturating_sub(prev.timestamp);
                    if !self.has_synchronization_between(prev.thread_id, thread_id, time_gap) {
                        self.violations.push(RaceViolation {
                            address: address,
                            first_access: prev.clone(),
                            second_access: AccessRecord {
                                address, thread_id, language, function, access_type, timestamp: now,
                            },
                            time_gap_us: time_gap,
                        });
                    }
                }
            }
        }

        self.access_log.insert(address, AccessRecord {
            address, thread_id, language, function, access_type, timestamp: now,
        });
    }

    fn has_synchronization_between(self, t1: u64, t2: u64, gap_us: u64) -> bool {
        // Check if there was a known synchronization point (mutex, barrier, etc.)
        // between the two threads within the time gap
        false // simplified — real impl tracks sync primitives
    }

    fn timestamp_us(self) -> u64 {
        // High-resolution timestamp
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}
```

---

## Tools & Techniques

### AddressSanitizer Across Languages

ASan must be enabled in every compiled language that touches native memory. A half-sanitized system detects nothing.

```bash
# Build all layers with AddressSanitizer enabled

# Fusion native extensions
fusion build --release -- \
    -Z sanitizer=address \
    -C 'codegen-units=1' \
    -C 'debug-info-level=2'

# C++ shared libraries
g++ -fsanitize=address -fno-omit-frame-pointer -g \
    -O1 -shared -o libprocessor.so processor.cpp

# Python C extensions
python setup.py build_ext \
    --inplace \
    --define='SANITIZER_ENABLED' \
    CFLAGS='-fsanitize=address -fno-omit-frame-pointer'

# Set ASan options for cross-language reporting
export ASAN_OPTIONS='
    detect_leaks=1
    detect_stack_use_after_return=1
    print_stats=1
    symbolize=1
    abort_on_error=1
    detect_odr_violation=2
'

# Run with all sanitizers active
./my_polyglot_app --config=debug.toml 2>&1 | \
    asan_symbolize.py --demangle --demangle-cpp | \
    tee sanitizer_output.txt
```

```toml
# fusion.toml — ASan integration for Fusion polyglot builds

[sanitizer]
enabled = true
type = "address"
detect_leaks = true
detect_stack_use_after_return = true
print_stats = true
symbolize = true

[sanitizer.report]
format = "full"         # "full", "minimal", "html"
output_dir = "./reports/sanitizer"
suppress_known = true   # suppress known third-party issues

[sanitizer.suppressions]
# File listing known acceptable sanitizer findings
file = "./suppressions/known_issues.txt"

# Example suppressions file content:
# leak:libpython3.11.so
# heap-buffer-overflow:third_party/codec.c
# use-after-free:vendor/legacy_parser.c (fixed in next release)

[[sanitizer.cross_language]]
# Ensure shared allocators report to the same sanitizer
native_lib = "libprocessor.so"
allocator_bridge = true
track_alloc_source = true

[[sanitizer.cross_language]]
native_lib = "libcrypto_bridge.so"
allocator_bridge = true
track_alloc_source = true
```

### Valgrind for Native Interop

Valgrind's memcheck, cachegrind, and helgrind tools work at the binary level, catching bugs regardless of which language generated the code.

```bash
# Full Valgrind run with cross-language awareness

valgrind \
    --tool=memcheck \
    --leak-check=full \
    --show-leak-kinds=all \
    --track-origins=yes \
    --verbose \
    --log-file=valgrind_memcheck.log \
    --suppressions=suppressions/third_party.supp \
    --suppressions=suppressions/python_runtime.supp \
    ./my_polyglot_app --config=debug.toml

# Helgrind for race conditions
valgrind \
    --tool=helgrind \
    --history-level=full \
    --delta-stacktrace=yes \
    --log-file=valgrind_helgrind.log \
    --suppressions=suppressions/thread_known.supp \
    ./my_polyglot_app --config=debug.toml

# DRD (more precise for some patterns)
valgrind \
    --tool=drd \
    --check-stack-var=yes \
    --first-race-only=no \
    --log-file=valgrind_drd.log \
    ./my_polyglot_app --config=debug.toml
```

```python
# valgrind_wrapper.py — automate Valgrind runs for polyglot apps

import subprocess
import re
import json
from pathlib import Path
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class ValgrindError:
    tool: str
    kind: str           # "InvalidRead", "UseAfterFree", "Leak", etc.
    what: str           # description
    location: str       # file:line or address
    stack: List[str]    # stack trace lines
    suppression: str    # suggested suppression rule

class PolyglotValgrindRunner:
    """Run Valgrind and parse results for polyglot applications."""

    def __init__(self, binary: str, args: List[str], workdir: Path):
        self.binary = binary
        self.args = args
        self.workdir = workdir
        self.suppression_dir = workdir / "suppressions"
        self.suppression_dir.mkdir(exist_ok=True)

    def run_memcheck(self) -> List[ValgrindError]:
        log_file = self.workdir / "valgrind_memcheck.log"
        cmd = [
            "valgrind", "--tool=memcheck",
            "--leak-check=full", "--show-leak-kinds=all",
            "--track-origins=yes", "--verbose",
            f"--log-file={log_file}",
            self.binary, *self.args,
        ]

        result = subprocess.run(cmd, capture_output=True, timeout=300)
        return self.parse_valgrind_log(log_file, "memcheck")

    def run_helgrind(self) -> List[ValgrindError]:
        log_file = self.workdir / "valgrind_helgrind.log"
        cmd = [
            "valgrind", "--tool=helgrind",
            "--history-level=full",
            f"--log-file={log_file}",
            self.binary, *self.args,
        ]

        result = subprocess.run(cmd, capture_output=True, timeout=300)
        return self.parse_valgrind_log(log_file, "helgrind")

    def parse_valgrind_log(self, log_file: Path, tool: str) -> List[ValgrindError]:
        if not log_file.exists():
            return []

        content = log_file.read_text()
        errors = []

        # Parse error blocks
        error_pattern = re.compile(
            r'==\d+== (\w+): (.*?)\n'
            r'==\d+==    at (0x[0-9A-F]+): (.+?) \((.+?):(\d+)\)\n'
            r'==\d+==    by (0x[0-9A-F]+): (.+?) \((.+?):(\d+)\)',
            re.MULTILINE
        )

        for match in error_pattern.finditer(content):
            kind = match.group(1)
            what = match.group(2)
            location = f"{match.group(5)}:{match.group(6)}"
            stack = [match.group(4), match.group(8)]

            errors.append(ValgrindError(
                tool=tool,
                kind=kind,
                what=what,
                location=location,
                stack=stack,
                suppression=self.generate_suppression(kind, location, stack),
            ))

        return errors

    def generate_suppression(self, kind: str, location: str, stack: List[str]) -> str:
        """Generate a Valgrind suppression rule for a known acceptable error."""
        func_name = stack[-1].split('(')[0] if stack else "unknown_func"
        return (
            f"{{\n"
            f"   {kind}: {location}\n"
            f"   fun:{func_name}\n"
            f"}}"
        )

    def write_suppressions(self, errors: List[ValgrindError], filename: str):
        """Write suppression file for repeatable acceptable errors."""
        supp_file = self.suppression_dir / filename
        with open(supp_file, 'w') as f:
            for error in errors:
                f.write(error.suppression + "\n\n")
```

### Thread Sanitizer for Concurrent Polyglot Code

ThreadSanitizer (TSan) detects data races but requires every language layer to be compiled with TSan support.

```bash
# TSan build for all layers

# Fusion native
fusion build --release -- -Z sanitizer=thread

# C++
g++ -fsanitize=thread -g -O1 -shared -o libworker.so worker.cpp

# Python (requires python-debug with TSan)
PYTHON_CC='gcc -fsanitize=thread' python -m pip install my_extension

# TSan suppression file for known cross-language false positives
cat > tsan_suppressions.txt << 'EOF'
# Python GIL is a synchronization mechanism, but TSan doesn't know about it
race:PyGILState_Ensure
race:PyEval_SaveThread
race:Py_BEGIN_ALLOW_THREADS

# Some third-party libraries use internal locks
race:some_third_party_function
deadlock:libthirdparty.so

# JVM JIT thread management
race:JIT_Compile
race:SharedRuntime::thread_single
EOF

TSAN_OPTIONS='
    suppressions=tsan_suppressions.txt
    history_size=7
    second_deadlock_stack=1
    print_suppressions=0
' ./my_polyglot_app --config=debug.toml 2>&1 | tsan_demangle | tee tsan_output.txt
```

### Heap Profiler for Memory Leak Detection

```bash
# Heap profiling with heapprofd (Android/Linux) or heaptrack

# heaptrack — comprehensive heap profiling
heaptrack ./my_polyglot_app --config=debug.toml
heaptrack --analyze heaptrack.my_polyglot_app.*.gz > heap_report.txt

# pprof (Go-adjacent but works for any C/C++/Rust)
HEAPPROFILE=/tmp/heap_profile \
    LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libprofiler.so \
    ./my_polyglot_app --config=debug.toml

# Generate flamegraph from pprof data
pprof -flame /tmp/heap_profile.0001.heap > flamegraph.html

# Cross-language memory attribution
# This script correlates allocation sites across language boundaries
cat > cross_lang_heap_report.py << 'PYEOF'
import re
import sys
from collections import defaultdict

def parse_heap_profile(filename):
    """Parse a heap profile and attribute allocations to source languages."""
    allocations = defaultdict(lambda: {"bytes": 0, "count": 0, "language": "unknown"})

    with open(filename) as f:
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            # Parse: bytes count alloc_size alloc_count location
            parts = line.strip().split()
            if len(parts) >= 5:
                location = " ".join(parts[4:])
                bytes_val = int(parts[0])
                count = int(parts[1])

                # Determine language from symbols in the stack
                lang = detect_language_from_stack(location)
                allocations[location]["bytes"] += bytes_val
                allocations[location]["count"] += count
                allocations[location]["language"] = lang

    return allocations

def detect_language_from_stack(stack_trace):
    """Heuristic: determine which language owns an allocation site."""
    if any(marker in stack_trace for marker in ["Py_", "PyObject", "python3"]):
        return "python"
    if any(marker in stack_trace for marker in ["_ZN", "std::", "__cxa"]):
        return "cpp"
    if any(marker in stack_trace for marker in ["rust", "::", "core::"]):
        return "rust"
    if any(marker in stack_trace for marker in ["Java_", "JNI_", "jvm"]):
        return "java"
    if any(marker in stack_trace for marker in ["malloc", "calloc", "realloc"]):
        return "c"
    return "unknown"

if __name__ == "__main__":
    profile = parse_heap_profile(sys.argv[1])

    # Group by language
    by_lang = defaultdict(lambda: {"total_bytes": 0, "total_count": 0, "sites": 0})
    for loc, data in profile.items():
        lang = data["language"]
        by_lang[lang]["total_bytes"] += data["bytes"]
        by_lang[lang]["total_count"] += data["count"]
        by_lang[lang]["sites"] += 1

    print("Cross-Language Heap Allocation Report")
    print("=" * 50)
    for lang, data in sorted(by_lang.items(), key=lambda x: -x[1]["total_bytes"]):
        print(f"\n{lang.upper()}:")
        print(f"  Total allocated: {data['total_bytes']:,} bytes ({data['total_count']:,} allocations)")
        print(f"  Allocation sites: {data['sites']}")
        print(f"  Avg per alloc: {data['total_bytes'] // max(data['total_count'], 1):,} bytes")
PYEOF
```

---

## Debugging Checklist

When you hit a crash or data corruption in a polyglot system, work through this checklist in order:

1. **Identify the crashing language** — which runtime produced the error message? If it's a segfault, which binary was loaded at the faulting address?

2. **Check symbol availability** — can you resolve the crash address to a function name? If not, rebuild with `-g` / `debug-info` and ensure debug symbols are not stripped.

3. **Enable sanitizers** — rebuild every layer with ASan (memory) and TSan (threads). A half-sanitized build catches nothing.

4. **Check the boundary** — examine the FFI call site. Is the function signature correct? Are argument types matching? Is the calling convention correct?

5. **Verify memory ownership** — who allocated the memory? Who freed it? Is there a double-free or use-after-free across the boundary?

6. **Check serialization** — are you serializing and deserializing in the same format? Same endianness? Same encoding?

7. **Enable Valgrind** — run with `--track-origins=yes` to find the root cause of memory corruption, not just the symptom.

8. **Add boundary logging** — instrument every FFI call with entry/exit logs showing argument values and return values.

9. **Reproduce minimally** — strip away everything except the crashing path. Can you trigger the bug with a single FFI call?

10. **Check compiler optimizations** — does the bug only appear in release builds? If so, the optimizer may be exploiting undefined behavior at the boundary.
