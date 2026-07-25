# Chapter 44: Documentation as a Boundary Contract

In polyglot systems, documentation isn't just helpful — it's the contract that keeps your Rust FFI from segfaulting your Python. Every interop boundary is a trust boundary, and documentation is how you formalize that trust. This chapter covers how to unify documentation across languages and make it enforceable.

## The Documentation Gap

Every language has its own documentation convention. Rust has `///` doc comments, Python has docstrings, Java has Javadoc, Go has `godoc`, and JavaScript has JSDoc. In a monolingual project, this is fine — you pick one tool and move on. In a polyglot project, you now have five documentation systems that don't know about each other, five different output formats, and zero guarantees that any of them agree on what the API actually does.

The result: developers read the Rust docs, call the function from Python, and get a type error because the docs described the Rust type, not the Python type.

## Unifying Documentation into One Static Site

The goal is a single documentation site that covers every language in your system. The tool choices are Docusaurus (React-based, great plugin ecosystem) or MkDocs with Material theme (Python-based, simpler setup).

### Docusaurus Setup for Polyglot Projects

```javascript
// docusaurus.config.js
module.exports = {
  title: 'Fusion Polyglot API',
  themeConfig: {
    navbar: {
      items: [
        { type: 'doc', docId: 'overview', position: 'left', label: 'Overview' },
        {
          type: 'dropdown',
          label: 'Language Guides',
          items: [
            { label: 'Rust API', to: '/api/rust' },
            { label: 'Python API', to: '/api/python' },
            { label: 'Go API', to: '/api/go' },
            { label: 'JavaScript API', to: '/api/javascript' },
          ],
        },
      ],
    },
  },
  plugins: [
    // Custom plugin to pull rustdoc output
    async function rustdocPlugin(context, options) {
      return {
        name: 'rustdoc-plugin',
        async loadContent() {
          // Reads pre-generated rustdoc JSON output
          const fs = require('fs');
          const rustdocPath = 'target/doc/api.json';
          if (fs.existsSync(rustdocPath)) {
            return JSON.parse(fs.readFileSync(rustdocPath, 'utf-8'));
          }
          return null;
        },
        async contentLoaded({ content, actions }) {
          if (!content) return;
          const { createData, addRoute } = actions;
          // Generate a page for each Rust module
          for (const mod of content.modules || []) {
            const pagePath = `/api/rust/${mod.name}`;
            const data = await createData(
              `rustdoc-${mod.name}.json`,
              JSON.stringify(mod)
            );
            addRoute({ path: pagePath, component: '@site/src/components/RustdocPage', modules: data });
          }
        },
      };
    },
  ],
};
```

### MkDocs Alternative

```yaml
# mkdocs.yml
site_name: Fusion Polyglot API
theme:
  name: material
  features:
    - navigation.tabs
    - navigation.sections
    - content.code.copy

nav:
  - Overview: index.md
  - Rust API:
      - api/rust/index.md
      - Modules: api/rust/modules.md
  - Python API:
      - api/python/index.md
      - Modules: api/python/modules.md
  - Go API:
      - api/go/index.md
  - JavaScript API:
      - api/js/index.md

plugins:
  - search
  - mkdocstrings:
      handlers:
        python:
          options:
            show_source: true
            show_root_heading: true
```

### CI-Driven Documentation Generation

The key insight: documentation should be generated from code, not written by hand. CI should build docs on every push and deploy them automatically.

```yaml
# .github/workflows/docs.yml
name: Build & Deploy Polyglot Docs
on:
  push:
    branches: [main]
    paths:
      - 'src/**'
      - 'docs/**'
      - '*.toml'
      - 'requirements.txt'

jobs:
  build-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Generate Rust docs
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Generate rustdoc
        run: cargo doc --no-deps --document-private-items --output-dir docs/api/rust

      # Generate Python docs
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Generate pydoc
        run: |
          pip install -r requirements.txt
          pydoc-markdown -I src -m pyfusion -o docs/api/python

      # Generate Go docs
      - name: Set up Go
        uses: actions/setup-go@v5
        with:
          go-version: '1.22'
      - name: Generate godoc
        run: |
          go install golang.org/x/tools/cmd/godoc@latest
          godoc -http=:6060 &
          sleep 2
          curl -s http://localhost:6060/pkg/ > docs/api/go/index.html

      # Generate JSDoc
      - name: Set up Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Generate jsdoc
        run: |
          npm install jsdoc
          npx jsdoc src/js -d docs/api/js

      # Build unified site
      - name: Build Docusaurus site
        run: |
          cd docs
          npm install
          npm run build

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: docs/build
```

## Every Interop Function Needs Fuzzed Examples

Documentation for interop functions must show the exact wire format — not abstractions, not "this returns a User object," but the actual bytes/JSON/protobuf that crosses the boundary.

### The Fuzzed Example Pattern

```rust
// src/ffi/user.rs

/// Deserialize a user record from a JSON byte buffer.
///
/// # FFI Contract
/// - `buf` must point to valid UTF-8 encoded JSON
/// - `buf_len` must match the actual byte length
/// - Returns a null pointer on error (check with `fusion_error_last()`)
///
/// # Wire Format
/// Input: JSON object with these exact fields:
/// ```json
/// {
///   "id": 12345,           // i64, required
///   "name": "Alice",       // string, required, max 256 bytes UTF-8
///   "email": "a@b.com",    // string, required, valid email format
///   "roles": ["admin"],    // array of strings, at least one required
///   "metadata": null        // object or null
/// }
/// ```
///
/// Output: JSON object:
/// ```json
/// {
///   "id": 12345,
///   "name": "Alice",
///   "email": "a@b.com",
///   "roles": ["admin", "user"],
///   "created_at": "2024-01-15T10:30:00Z",  // ISO 8601
///   "version": 2                               // i32, incremented on mutation
/// }
/// ```
///
/// # Fuzzed Examples (canonical)
///
/// ## Happy path
/// Input:  `{"id":1,"name":"Bob","email":"bob@test.com","roles":["user"],"metadata":null}`
/// Output: `{"id":1,"name":"Bob","email":"bob@test.com","roles":["user"],"created_at":"2024-01-15T00:00:00Z","version":1}`
///
/// ## Empty name (triggers validation)
/// Input:  `{"id":2,"name":"","email":"x@test.com","roles":["user"],"metadata":null}`
/// Output: `{"error":"validation","message":"name must be non-empty","field":"name"}`
///
/// ## Empty roles (triggers validation)
/// Input:  `{"id":3,"name":"Eve","email":"eve@test.com","roles":[],"metadata":null}`
/// Output: `{"error":"validation","message":"roles must have at least one entry","field":"roles"}`
///
/// ## Null metadata (valid)
/// Input:  `{"id":4,"name":"Frank","email":"frank@test.com","roles":["admin"],"metadata":null}`
/// Output: `{"id":4,"name":"Frank","email":"frank@test.com","roles":["admin"],"created_at":"2024-01-15T00:00:00Z","version":1}`
///
/// ## Missing required field
/// Input:  `{"id":5,"email":"grace@test.com","roles":["user"],"metadata":null}`
/// Output: `{"error":"validation","message":"missing required field","field":"name"}`
#[no_mangle]
pub extern "C" fn fusion_user_deserialize(
    buf: *const u8,
    buf_len: usize,
) -> *mut UserRecord {
    // implementation
}
```

### Python-Side Documentation

```python
"""User record deserialization and management.

This module provides the Python interface for Fusion user records.
All functions mirror the Rust FFI layer and use JSON for serialization.

Wire Format (canonical examples)::

    >>> from fusion.user import deserialize_user
    >>> result = deserialize_user('{"id":1,"name":"Bob","email":"bob@test.com","roles":["user"],"metadata":null}')
    >>> result
    {"id": 1, "name": "Bob", "email": "bob@test.com", "roles": ["user"], ...}

    >>> deserialize_user('{"id":2,"name":"","email":"x@test.com","roles":["user"],"metadata":null}')
    {"error": "validation", "message": "name must be non-empty", "field": "name"}
"""

import json
from typing import Optional, Dict, Any

def deserialize_user(json_str: str) -> Dict[str, Any]:
    """Deserialize a JSON string into a Fusion user record.

    Args:
        json_str: A JSON string matching the Fusion user schema.
            Must include: id (int), name (str), email (str), roles (list[str]).
            Optional: metadata (dict or null).

    Returns:
        A dictionary with the deserialized user record, or an error dict
        with keys "error", "message", and "field".

    Raises:
        TypeError: If json_str is not a string.
        ValueError: If the string is not valid JSON.

    Wire Format Examples::

        # Happy path
        >>> deserialize_user('{"id":1,"name":"Bob","email":"bob@test.com","roles":["user"],"metadata":null}')
        {"id": 1, "name": "Bob", ...}

        # Validation error — empty name
        >>> deserialize_user('{"id":2,"name":"","email":"x@test.com","roles":[],"metadata":null}')
        {"error": "validation", "message": "name must be non-empty", "field": "name"}
    """
    if not isinstance(json_str, str):
        raise TypeError(f"Expected str, got {type(json_str).__name__}")
    data = json.loads(json_str)
    return _validate_and_transform(data)
```

### Go-Side Documentation

```go
// Package fusion provides Go bindings for the Fusion user record system.
//
// Wire Format (canonical examples):
//
// Happy path:
//
//	Input:  {"id":1,"name":"Bob","email":"bob@test.com","roles":["user"],"metadata":null}
//	Output: {"id":1,"name":"Bob","email":"bob@test.com","roles":["user"],"created_at":"2024-01-15T00:00:00Z","version":1}
//
// Validation error:
//
//	Input:  {"id":2,"name":"","email":"x@test.com","roles":[],"metadata":null}
//	Output: {"error":"validation","message":"name must be non-empty","field":"name"}
package fusion

// DeserializeUser parses a JSON byte slice into a UserRecord.
//
// The input must conform to the Fusion user schema:
//   - id: int64, required
//   - name: string, required, non-empty, max 256 bytes
//   - email: string, required, valid email
//   - roles: []string, required, at least one entry
//   - metadata: map[string]any or nil
//
// Returns a UserRecord on success, or a ValidationError on failure.
func DeserializeUser(data []byte) (*UserRecord, error) {
	// implementation
}
```

## Data Structure Documentation at Language Boundaries

When a data structure crosses a language boundary, you need to document it once in a canonical form, then show how each language represents it.

### The Canonical Schema Pattern

```markdown
<!-- docs/schemas/user-record.md -->

# UserRecord — Canonical Schema

This is the single source of truth for the UserRecord data structure.
All language implementations MUST conform to this schema.

## Fields

| Field       | Type         | Required | Constraints                    |
|-------------|--------------|----------|--------------------------------|
| id          | i64          | yes      | > 0                            |
| name        | string       | yes      | 1-256 bytes UTF-8              |
| email       | string       | yes      | valid email format             |
| roles       | string[]     | yes      | at least 1 entry               |
| metadata    | object|null  | no       | arbitrary key-value pairs      |
| created_at  | ISO 8601     | no       | auto-generated on creation     |
| version     | i32          | no       | auto-incremented on mutation   |

## Language Mappings

### Rust
```rust
#[derive(Serialize, Deserialize)]
pub struct UserRecord {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}
```

### Python
```python
@dataclass
class UserRecord:
    id: int
    name: str
    email: str
    roles: list[str]
    metadata: Optional[dict[str, Any]] = None
    created_at: Optional[str] = None  # ISO 8601
    version: Optional[int] = None
```

### Go
```go
type UserRecord struct {
    ID        int64              `json:"id"`
    Name      string             `json:"name"`
    Email     string             `json:"email"`
    Roles     []string           `json:"roles"`
    Metadata  map[string]any     `json:"metadata,omitempty"`
    CreatedAt *time.Time         `json:"created_at,omitempty"`
    Version   *int32             `json:"version,omitempty"`
}
```

### JavaScript/TypeScript
```typescript
interface UserRecord {
  id: number;       // i64
  name: string;     // max 256 bytes
  email: string;    // valid email
  roles: string[];  // at least 1
  metadata?: Record<string, unknown> | null;
  created_at?: string;  // ISO 8601
  version?: number;     // i32
}
```

## Serialization Notes

- All interop uses **JSON** as the wire format
- Field names are **snake_case** in JSON, regardless of language conventions
- `null` and absent fields are treated identically (use `Option`/`nullable`)
- Numbers are always JSON numbers (no string-wrapped integers)
- Dates are always ISO 8601 strings (no Unix timestamps)
```

### Automated Schema Validation

```python
# tests/test_schema_conformance.py
"""Verify that every language implementation matches the canonical schema."""

import json
import subprocess
import pytest
from pathlib import Path

SCHEMA_PATH = Path("docs/schemas/user-record.json")

@pytest.fixture
def canonical_schema():
    return json.loads(SCHEMA_PATH.read_text())

@pytest.mark.parametrize("language,rust_binary,python_module", [
    ("rust", "target/debug/fusion-test", None),
    ("python", None, "fusion.user"),
])
def test_schema_roundtrip(canonical_schema, language, rust_binary, python_module):
    """Create a record in Rust, deserialize in Python, verify fields."""
    # Generate a canonical record
    canonical_record = {
        "id": 1,
        "name": "Test User",
        "email": "test@example.com",
        "roles": ["user"],
        "metadata": {"key": "value"},
    }

    if rust_binary:
        # Rust produces JSON
        result = subprocess.run(
            [rust_binary, "create-user", json.dumps(canonical_record)],
            capture_output=True, text=True
        )
        rust_output = json.loads(result.stdout)
    else:
        # Python produces JSON
        import importlib
        mod = importlib.import_module(python_module)
        rust_output = mod.create_user(canonical_record)

    # Verify all canonical fields present
    for field, spec in canonical_schema["properties"].items():
        assert field in rust_output, f"{language} output missing field: {field}"

    # Verify types match
    assert isinstance(rust_output["id"], int)
    assert isinstance(rust_output["name"], str)
    assert isinstance(rust_output["roles"], list)
    assert all(isinstance(r, str) for r in rust_output["roles"])
```

## The Polyglot README Pattern

Every interop boundary should have a README that follows this template. It's the first thing a developer sees when they need to call across languages.

```markdown
# fusion-ffi — Rust/Python Interop

## What This Does

Exposes Fusion's user record processing to Python via C FFI.

## Quick Start

### From Rust

```rust
use fusion_ffi::UserRecord;

let user = UserRecord::from_json(r#"{"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]}"#)?;
println!("{}", user.name); // "Alice"
```

### From Python

```python
import fusion

user = fusion.user.deserialize('{"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]}')
print(user["name"])  # "Alice"
```

### From Go

```go
user, err := fusion.DeserializeUser([]byte(`{"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]}`))
if err != nil {
    log.Fatal(err)
}
fmt.Println(user.Name) // "Alice"
```

## Wire Format

| Direction         | Format | Example                                                    |
|-------------------|--------|------------------------------------------------------------|
| Rust → Python     | JSON   | `{"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]}` |
| Rust → Go         | JSON   | Same as above                                              |
| Error responses   | JSON   | `{"error":"validation","message":"...","field":"name"}`    |

## Error Handling

All errors are returned as JSON objects with this shape:

```json
{
  "error": "validation|runtime|memory",
  "message": "Human-readable description",
  "field": "optional_field_name"
}
```

## Performance Notes

- Deserialization: ~2μs per record (Rust), ~15μs (Python via FFI)
- Throughput: ~500K records/sec (Rust), ~30K records/sec (Python)
- Memory: Rust allocates on the stack; Python copies to heap

## Testing

```bash
# Run all cross-language tests
cargo test --all

# Run Python-specific tests
cd bindings/python && pytest

# Run Go-specific tests
cd bindings/go && go test ./...
```

## Breaking Changes

See [DEPRECATION.md](./DEPRECATION.md) for the policy on wire format changes.
```

## Enforcing Documentation Accuracy with Tests

Documentation rots when it's not tested. The solution: make documentation examples runnable.

```rust
// In Rust, use doc tests to verify examples compile and run
/// Process a batch of user records.
///
/// # Example
/// ```
/// use fusion::batch::process_users;
///
/// let input = r#"[
///   {"id":1,"name":"Alice","email":"a@b.com","roles":["admin"]},
///   {"id":2,"name":"Bob","email":"b@b.com","roles":["user"]}
/// ]"#;
///
/// let results = process_users(input).unwrap();
/// assert_eq!(results.len(), 2);
/// assert_eq!(results[0].name, "Alice");
/// ```
pub fn process_users(json: &str) -> Result<Vec<UserRecord>, Error> {
    // implementation
}
```

```python
# In Python, use doctest
def deserialize_user(json_str: str) -> dict:
    """Deserialize a user record from JSON.

    >>> deserialize_user('{"id":1,"name":"Bob","email":"bob@test.com","roles":["user"]}')
    {'id': 1, 'name': 'Bob', 'email': 'bob@test.com', 'roles': ['user']}

    >>> deserialize_user('invalid json')
    Traceback (most recent call last):
        ...
    json.decoder.JSONDecodeError: ...
    """
    return json.loads(json_str)
```

```go
// In Go, use TestableExamples
// ExampleDeserializeUser is a testable example.
func ExampleDeserializeUser() {
    input := []byte(`{"id":1,"name":"Bob","email":"bob@test.com","roles":["user"]}`)
    user, err := DeserializeUser(input)
    if err != nil {
        panic(err)
    }
    fmt.Println(user.Name)
    // Output: Bob
}
```

## Documentation Review Checklist

When reviewing documentation at interop boundaries:

1. **Wire format shown?** Every function that crosses a boundary must show the exact JSON/protobuf format
2. **Error format documented?** What does the error response look like? What fields are present?
3. **Fuzzed examples included?** Not just the happy path — show edge cases, empty strings, null values, missing fields
4. **Language mappings consistent?** The same field should have the same name in every language's docs
5. **Performance characteristics noted?** How fast is this function? How much memory does it use?
6. **Breaking change policy stated?** When does the wire format change? How is it versioned?
7. **Tests verify examples?** Doctests, example tests, or integration tests that run the documented examples

## Summary

Documentation in a polyglot system is a contract, not a courtesy. It must be:
- **Generated from code** (CI-driven, not hand-maintained)
- **Tested** (doctests verify examples compile and run)
- **Canonical** (one source of truth for each data structure)
- **Complete** (wire format, error format, edge cases, performance)

The polyglot README pattern gives every interop boundary a consistent, comprehensive document. Follow it, and your future self (and your teammates) will thank you.
