# Chapter 49: Succession & Knowledge Decay

The greatest risk in a polyglot system isn't a production outage — it's the day the person who understands the FFI layer leaves. Knowledge decay is silent: the code still runs, the tests still pass, but nobody knows *why* the boundary works the way it does. This chapter covers how to build systems that survive their original authors.

## The Bus Factor Problem

The "bus factor" is the number of people who must be hit by a bus before a project becomes unmaintainable. In polyglot systems, the bus factor is often 1 for the interop layer — because only one person understood how to wire Rust to Python to Go.

### Measuring Your Bus Factor

```python
# bus_factor_audit.py
"""Audit codebase for bus factor risk."""

import subprocess
import json
from collections import defaultdict
from datetime import datetime, timedelta

def get_blame_ownership(file_path):
    """Get line ownership from git blame."""
    result = subprocess.run(
        ["git", "blame", "--line-porcelain", file_path],
        capture_output=True, text=True
    )

    authors = defaultdict(int)
    for line in result.stdout.split('\n'):
        if line.startswith('author '):
            authors[line[7:]] += 1

    return dict(sorted(authors.items(), key=lambda x: -x[1]))

def audit_bus_factor():
    """Find files with bus factor of 1."""
    high_risk_files = []

    # Check all interop files
    interop_patterns = [
        "src/ffi/**/*.rs",
        "bindings/**/*.py",
        "bindings/**/*.go",
        "bindings/**/*.js",
        "src/proto/**",
    ]

    for pattern in interop_patterns:
        result = subprocess.run(
            ["git", "ls-files", pattern],
            capture_output=True, text=True
        )
        for file in result.stdout.strip().split('\n'):
            if not file:
                continue
            ownership = get_blame_ownership(file)
            if len(ownership) == 1:
                author = list(ownership.keys())[0]
                high_risk_files.append({
                    "file": file,
                    "sole_author": author,
                    "lines": list(ownership.values())[0],
                })

    return high_risk_files

# Example output:
# [
#   {"file": "src/ffi/user.rs", "sole_author": "Alice", "lines": 342},
#   {"file": "bindings/python/user.py", "sole_author": "Alice", "lines": 89},
#   {"file": "src/proto/user.proto", "sole_author": "Alice", "lines": 45},
# ]
# Alice is the bus factor of 1 for the entire user interop layer.
```

### Mitigation Strategies

```yaml
# .github/CODEOWNERS — Force multi-person review for interop code
# Every interop file must be reviewed by at least 2 people

src/ffi/        @fusion-core-team @fusion-python-team
bindings/       @fusion-core-team @fusion-python-team
src/proto/      @fusion-core-team @fusion-python-team @fusion-go-team
docs/schemas/   @fusion-core-team @fusion-python-team @fusion-go-team

# Rotate on-call for interop issues
# Every month, a different person handles interop bugs
```

```python
# CODEOWNERS enforcement via GitHub Actions
# .github/workflows/enforce-codeowners.yml
name: Enforce Multi-Person Review
on:
  pull_request:
    paths:
      - 'src/ffi/**'
      - 'bindings/**'
      - 'src/proto/**'

jobs:
  check-reviewers:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@v7
        with:
          script: |
            const pr = await github.rest.pulls.get({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number,
            });

            const reviewers = pr.data.requested_reviewers.map(r => r.login);
            const interopFiles = pr.data.changed_files.filter(f =>
              f.includes('ffi') || f.includes('bindings') || f.includes('proto')
            );

            if (interopFiles.length > 0 && reviewers.length < 2) {
              core.setFailed(
                `Interop files changed but only ${reviewers.length} reviewer(s). ` +
                `Need at least 2 reviewers for interop code.`
              );
            }
```

## Golden Tests at Every Interop Boundary

Golden tests (also called snapshot tests or approval tests) capture the exact behavior of an interop boundary and verify it never changes unexpectedly. They're the most important tests in a polyglot system.

### What Golden Tests Look Like

```
tests/golden/
├── user_serialize/
│   ├── input.json                    # Input to the boundary
│   ├── expected_rust_output.json     # What Rust produces
│   ├── expected_python_output.json   # What Python produces
│   ├── expected_go_output.json       # What Go produces
│   └── README.md                     # Why this test exists
├── user_deserialize/
│   ├── input_rust.json               # Rust's wire format
│   ├── input_python.json             # Python's wire format
│   ├── expected_unified.json         # The canonical output
│   └── README.md
├── error_handling/
│   ├── input_invalid.json
│   ├── expected_error.json
│   └── README.md
└── edge_cases/
    ├── empty_name.json
    ├── unicode_characters.json
    ├── max_length_values.json
    └── null_fields.json
```

### Golden Test Implementation

```rust
// tests/golden/user_serialize.rs
//! Golden tests for user serialization across languages.
//!
//! These tests capture the EXACT wire format produced by each language.
//! If any output changes, the test fails and the developer must update
//! the golden file — forcing them to review the change.

use std::fs;
use serde_json::Value;

fn load_golden(name: &str, lang: &str) -> Value {
    let path = format!("tests/golden/{name}/expected_{lang}_output.json");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Golden file not found: {path}: {e}"));
    serde_json::from_str(&content).unwrap()
}

fn load_input(name: &str) -> Value {
    let path = format!("tests/golden/{name}/input.json");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Input file not found: {path}: {e}"));
    serde_json::from_str(&content).unwrap()
}

#[test]
fn golden_user_serialize_happy_path() {
    let input = load_input("user_serialize");
    let expected = load_golden("user_serialize", "rust");

    let result = fusion::user::serialize(&input);

    assert_eq!(result, expected, "Rust output differs from golden file. \
        If this change is intentional, update tests/golden/user_serialize/expected_rust_output.json");
}

#[test]
fn golden_user_serialize_error_path() {
    let input = load_golden("error_handling", "input_invalid");
    let expected = load_golden("error_handling", "expected_error");

    let result = fusion::user::deserialize(&input.to_string());

    assert!(result.is_err());
    let error_json = serde_json::to_value(result.unwrap_err()).unwrap();
    assert_eq!(error_json, expected);
}

/// Cross-language golden test: Rust and Python must produce identical output
#[test]
fn golden_cross_language_user_serialize() {
    let input = load_input("user_serialize");
    let rust_output = fusion::user::serialize(&input);

    // Load Python's golden output
    let python_output = load_golden("user_serialize", "python");

    // They must be identical (same JSON, same field order)
    assert_eq!(rust_output, python_output,
        "Rust and Python produce different output for the same input. \
        This means the interop boundary is broken.");
}
```

### Generating Golden Files

```python
# tests/golden/generate_goldens.py
"""Generate golden files for all languages.
Run this when you intentionally change the wire format."""
import json
import os
from pathlib import Path

def generate_golden_files():
    golden_dir = Path("tests/golden")

    for test_case in golden_dir.iterdir():
        if not test_case.is_dir():
            continue

        input_file = test_case / "input.json"
        if not input_file.exists():
            continue

        input_data = json.loads(input_file.read_text())

        # Generate Python output
        from fusion.user import serialize_user
        python_output = serialize_user(input_data)
        (test_case / "expected_python_output.json").write_text(
            json.dumps(python_output, indent=2, sort_keys=True)
        )

        # Generate Rust output (via subprocess)
        import subprocess
        result = subprocess.run(
            ["cargo", "run", "--bin", "generate-golden", "--", json.dumps(input_data)],
            capture_output=True, text=True
        )
        rust_output = json.loads(result.stdout)
        (test_case / "expected_rust_output.json").write_text(
            json.dumps(rust_output, indent=2, sort_keys=True)
        )

        # Generate Go output
        result = subprocess.run(
            ["go", "run", "cmd/generate-golden/main.go", json.dumps(input_data)],
            capture_output=True, text=True, cwd="bindings/go"
        )
        go_output = json.loads(result.stdout)
        (test_case / "expected_go_output.json").write_text(
            json.dumps(go_output, indent=2, sort_keys=True)
        )

        print(f"Generated goldens for {test_case.name}")

if __name__ == "__main__":
    generate_golden_files()
```

## Plain English Documentation of Intended Behavior

The most durable documentation is written in plain English, not code. It describes what the system *should* do, not how it does it. This survives language migrations, framework changes, and team turnover.

### The Behavior Specification Pattern

```markdown
<!-- docs/behaviors/user-record.md -->

# User Record — Intended Behavior

## What This Is

A UserRecord represents a user in the Fusion system. It's the core data
structure that flows between Rust (core processing), Python (ML features),
and Go (API layer).

## What It Does

1. **Creation**: A user is created via POST /users. The system validates
   the input, assigns an ID, sets created_at to the current UTC time,
   and sets version to 1.

2. **Retrieval**: A user is fetched by ID. The system returns the full
   record or a 404 if not found.

3. **Update**: A user is updated via PUT /users/{id}. The system validates
   the input, increments version, and returns the updated record.

4. **Deletion**: A user is deleted via DELETE /users/{id}. The system
   returns 204 with no body.

## What It Should NOT Do

- It should NOT accept a user with an empty name
- It should NOT accept a user with no roles
- It should NOT allow version to be set by the client (it's server-managed)
- It should NOT return the user's password hash (if we ever add one)
- It should NOT allow email changes without verification

## Edge Cases

| Scenario                    | Expected Behavior                          |
|-----------------------------|--------------------------------------------|
| Empty name                  | Return 422 with "name must be non-empty"   |
| No roles                    | Return 422 with "roles required"           |
| Duplicate email             | Return 409 with "email already exists"     |
| User not found              | Return 404 with "user not found"           |
| Concurrent update           | Last write wins, version increments         |
| Unicode in name             | Accept and store as-is                     |
| Very long name (>256 chars) | Return 422 with "name too long"            |
| Null metadata               | Treat as empty object {}                   |
| Version mismatch (optimistic locking) | Return 409 with current version |

## Who Maintains This

- **Primary**: Alice (Rust core, user record logic)
- **Secondary**: Bob (Python bindings, ML features)
- **Reviewers**: Charlie (Go API), Diana (testing)

Last updated: 2024-01-15 by Alice
```

### Why Plain English Survives

```markdown
# Code vs Plain English

## Code documentation (rots quickly)
```python
def create_user(data: dict) -> UserRecord:
    """Create a new user record.

    Args:
        data: Dict with keys: name (str), email (str), roles (list[str])

    Returns:
        UserRecord with generated id and created_at

    Raises:
        ValueError: If name is empty or roles is empty
    """
```
This documentation is tied to Python, to the specific function signature,
to the specific exception type. When you rename the function, change the
signature, or switch to Rust, this documentation is wrong.

## Plain English (survives changes)
"A user is created via POST /users. The system validates the input,
assigns an ID, sets created_at to the current UTC time, and sets version to 1."
This documentation is true regardless of language, framework, or implementation.
It describes behavior, not implementation.
```

## The Boring Glue Code Rule

At interop boundaries, never use exotic language features. The glue code should be boring, obvious, and easy to understand without deep language expertise.

### Bad: Exotic Features at Boundaries

```rust
// BAD: Complex Rust features at the FFI boundary
#[no_mangle]
pub extern "C" fn fusion_process_user(
    input: *const c_char,
) -> *mut c_char {
    // Using unsafe, raw pointers, manual memory management,
    // and complex trait objects at the boundary
    let c_str = unsafe { CStr::from_ptr(input) };
    let json_str = c_str.to_str().unwrap();

    // Complex trait object with lifetime parameters
    let processor: Box<dyn for<'a> Fn(&'a str) -> Result<UserRecord, Error> + Send + Sync> =
        Box::new(create_processor());

    // Async runtime inside sync FFI boundary
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(processor(json_str));

    // Manual memory allocation for return value
    let output = serde_json::to_string(&result.unwrap()).unwrap();
    let c_output = CString::new(output).unwrap();
    c_output.into_raw()
}
```

### Good: Boring Glue Code

```rust
// GOOD: Simple, boring, obvious glue code
#[no_mangle]
pub extern "C" fn fusion_process_user(
    input: *const c_char,
) -> *mut c_char {
    // Step 1: Convert C string to Rust string
    let c_str = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => return error_to_c_string("invalid UTF-8"),
    };

    // Step 2: Deserialize JSON
    let user: UserRecord = match serde_json::from_str(c_str) {
        Ok(u) => u,
        Err(e) => return error_to_c_string(&format!("invalid JSON: {e}")),
    };

    // Step 3: Validate
    if user.name.is_empty() {
        return error_to_c_string("name must be non-empty");
    }
    if user.roles.is_empty() {
        return error_to_c_string("roles must have at least one entry");
    }

    // Step 4: Process (pure function, no side effects)
    let result = process_user_logic(user);

    // Step 5: Serialize and return
    match serde_json::to_string(&result) {
        Ok(json) => CString::new(json).unwrap().into_raw(),
        Err(e) => error_to_c_string(&format!("serialization error: {e}")),
    }
}

// Helper: Simple error formatting
fn error_to_c_string(msg: &str) -> *mut c_char {
    let error = serde_json::json!({"error": msg});
    CString::new(serde_json::to_string(&error).unwrap())
        .unwrap()
        .into_raw()
}

// Business logic: Pure function, no FFI concerns
fn process_user_logic(mut user: UserRecord) -> UserRecord {
    user.name = user.name.trim().to_string();
    user.roles.sort();
    user.roles.dedup();
    user.version += 1;
    user.created_at = Some(chrono::Utc::now());
    user
}
```

### The Glue Code Checklist

```markdown
## Interop Boundary Code Review Checklist

### Language Features to AVOID at boundaries:
- [ ] No async/await (use sync wrappers)
- [ ] No generics beyond simple types
- [ ] No trait objects or interfaces
- [ ] No closures or lambdas
- [ ] No macros
- [ ] No complex error types (use strings)
- [ ] No custom allocators
- [ ] No unsafe code (except the minimum for FFI)

### Language Features to USE at boundaries:
- [ ] Simple structs with public fields
- [ ] String-based errors (JSON error objects)
- [ ] Synchronous function calls
- [ ] Explicit memory ownership (no hidden allocations)
- [ ] Clear, linear control flow
- [ ] Comments explaining the "why" for each step
```

## Knowledge Transfer Playbook

When a team member leaves, transfer knowledge with this checklist:

```markdown
## Interop Knowledge Transfer

### 1. Document the wire format
- [ ] List every function that crosses a boundary
- [ ] For each function: input format, output format, error format
- [ ] Provide fuzzed examples (see Chapter 44)

### 2. Document the failure modes
- [ ] What happens when the foreign language crashes?
- [ ] What happens when the network between services is slow?
- [ ] What happens when the schema changes without coordination?
- [ ] What monitoring catches these failures?

### 3. Document the testing strategy
- [ ] Where are the golden tests?
- [ ] How do you run the full interop test suite?
- [ ] How do you update golden files when you intentionally change behavior?

### 4. Document the deployment process
- [ ] What order must services be deployed?
- [ ] What's the rollback procedure?
- [ ] What's the schema migration process?

### 5. Do a live walkthrough
- [ ] Trace a request through all services (end-to-end)
- [ ] Show how to debug a cross-language error
- [ ] Show how to add a new field to the wire format
- [ ] Show how to run the contract tests
```

## Summary

Knowledge decay is inevitable; the question is whether your system can survive it. The strategies are straightforward: force multi-person review of interop code, maintain golden tests that capture exact behavior, write plain English documentation of intended behavior, and keep glue code boring. A system that's easy to understand is a system that survives its original authors.
