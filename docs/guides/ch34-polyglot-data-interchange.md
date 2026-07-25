# Chapter 34: The Data Interchange Rosetta Stone

When Fusion code talks to Python services, Rust microservices, or JavaScript frontends, data must cross language boundaries without corruption. This chapter is your field guide to serialization formats, type representation across languages, and the subtle traps that bite when data crosses the wire.

---

## Serialization Format Comparison

### JSON / YAML

**When to use:** Human-readable configuration, REST APIs, quick prototypes, debugging.

**When to avoid:** High-throughput IPC, binary precision, large payloads.

| Property | JSON | YAML |
|---|---|---|
| Readability | Good | Excellent |
| Schema enforcement | None (without JSON Schema) | None |
| Binary data | Base64-encoded (wasteful) | Same |
| Comments | Not supported | Supported |
| Parsing speed | Fast | Slower (10-100x) |
| Max practical size | ~10 MB | ~1 MB |
| Type support | Strings, numbers, bools, null, arrays, objects | Adds dates, anchors, tags |

**Fusion perspective:** JSON is the lingua franca. Fusion's `json` module provides zero-allocation parsing for hot paths, but for pure interop, prefer it when the other end is a web frontend or a Python script.

```fusion
// JSON round-trip with type safety
let data = json::parse<UserProfile>(raw_json)?;
let back = json::stringify(data);

// YAML for config files
let config = yaml::load<BuildConfig>(config_path)?;
```

### Protocol Buffers (Protobuf)

**When to use:** Microservice RPC, schema evolution, backward-compatible APIs.

**When to avoid:** Ad-hoc data exchange, when you need human readability.

| Property | Detail |
|---|---|
| Schema | `.proto` files, compiled per language |
| Versioning | Field numbers enable backward compat |
| Encoding | Binary, compact (typically 2-10x smaller than JSON) |
| Streaming | Supports streaming via gRPC |
| Ecosystem | First-class in Go, Java, Python, Rust (prost) |

**Key insight:** Protobuf's field number system means you can add new fields without breaking old clients. Remove fields with `reserved` to prevent reuse.

```protobuf
message SensorReading {
  string device_id = 1;
  double temperature = 2;
  int64 timestamp_ms = 3;
  map<string, string> metadata = 4; // added in v2
}
```

### FlatBuffers / Cap'n Proto

**When to use:** High-performance IPC, gaming, real-time systems, zero-copy reads.

**When to avoid:** When you need deep nesting, dynamic schemas, or cross-language simplicity.

| Property | FlatBuffers | Cap'n Proto |
|---|---|---|
| Zero-copy | Yes (access in-place) | Yes |
| Mutation | Not supported | Supported |
| Schema evolution | Field addition only | Flexible |
| Ecosystem | C++, Rust, Java, JS, C# | C++, Rust, Python, Go |
| Wire format | Slightly larger | Very compact |

**Key insight:** These formats deserialize by creating pointers into the existing buffer — no parsing step at all. For a million messages per second, this is the difference between viable and not.

### MessagePack

**When to use:** Drop-in binary replacement for JSON, Redis/Memcached serialization, interop where JSON is already dominant.

**When to avoid:** When you need schema evolution or streaming.

| Property | Detail |
|---|---|
| Compatibility | superset of JSON |
| Size | ~30-50% smaller than JSON |
| Speed | 2-5x faster parse than JSON |
| Schema | None |
| Extension types | Custom type IDs for domain types |

### CBOR (Concise Binary Object Representation)

**When to use:** IoT, constrained devices, COSE/JOSE integration, RFC 8949 compliance.

**When to avoid:** General-purpose microservice communication (use Protobuf instead).

| Property | Detail |
|---|---|
| Standard | IETF RFC 8949 |
| Tag system | Self-describing semantic tags |
| Half-precision | Native float16 support |
| Indefinite lengths | Streaming-friendly |
| IoT fit | Minimal overhead, no schema needed |

**Fusion + CBOR:** Use the `cbor` crate for constrained device communication. CBOR's tag system lets you embed type information without a schema, which is invaluable when devices speak different firmware versions.

---

## Type Representation Table

How fundamental types map across languages. Differences here cause silent bugs.

### Integer Types

| Type | Fusion | Python | JavaScript | Rust | Java | Go |
|---|---|---|---|---|---|---|
| 32-bit int | `int32` | `int` (arbitrary precision) | `Number` (53-bit safe) | `i32` | `int` | `int32` |
| 64-bit int | `int64` | `int` (arbitrary precision) | `BigInt` | `i64` | `long` | `int64` |
| Unsigned 32-bit | `uint32` | Natively unsigned: No | `Number` (unsafe) | `u32` | N/A (use `Integer.toUnsignedLong`) | `uint32` |
| Unsigned 64-bit | `uint64` | Natively unsigned: No | `BigInt` (unsigned) | `u64` | `Long` (signed only) | `uint64` |

**Danger zone:** JavaScript's `Number` is IEEE 754 double — it safely represents integers up to 2^53. Sending a Fusion `int64` to a JS frontend silently truncates values beyond that. Always use `BigInt` on the JS side for 64-bit IDs.

### Float Types

| Type | Fusion | Python | JavaScript | Rust | Java | Go |
|---|---|---|---|---|---|---|
| 32-bit float | `float32` | `float` (64-bit only) | `Number` (64-bit) | `f32` | `float` | `float32` |
| 64-bit float | `float64` | `float` | `Number` | `f64` | `double` | `float64` |

**Danger zone:** Python has no native 32-bit float. Sending `float32` data to Python via `struct.pack('<f', val)` works, but Python's `float()` will always be 64-bit. Round-tripping `float32` through Python can change the value by 1 ULP.

### Boolean

| Language | True values | False values | Falsy non-booleans |
|---|---|---|---|
| Fusion | `true` | `false` | None |
| Python | `True` | `False` | `0`, `""`, `None`, `[]`, `{}` |
| JavaScript | `true` | `false` | `0`, `""`, `null`, `undefined`, `NaN`, `[]` |
| Rust | `true` | `false` | None |
| Java | `true` | `false` | None |
| Go | `true` | `false` | None |

**Danger zone:** JavaScript's `[] == false` is `true`, and Python's `if []:` is falsy. If a serialization library converts these to booleans, empty arrays vanish.

### String (UTF-8)

| Language | Internal encoding | Null-terminated? | Max length |
|---|---|---|---|
| Fusion | UTF-8 | No | Memory-limited |
| Python 3 | UTF-8 (or flexible) | No | Memory-limited |
| JavaScript | UTF-16 (UCS-2) | No | ~2^53 chars (theoretical) |
| Rust | UTF-8 | No | Memory-limited |
| Java | UTF-16 | No | ~2^31 - 1 chars |
| Go | UTF-8 | No | Memory-limited |

**Danger zone:** JavaScript uses UTF-16 internally. Surrogate pairs (emoji, CJK extensions) are two 16-bit code units. If you slice a JS string at a surrogate boundary, you get a broken character. Pass strings as UTF-8 between Fusion and JS.

### Null / None

| Language | Null representation | Absent key |
|---|---|---|
| Fusion | `null` | Compile error (structs must be explicit) |
| Python | `None` | `KeyError` or `.get()` default |
| JavaScript | `null` or `undefined` | `undefined` |
| Rust | `Option<T>::None` | N/A (struct fields required unless `Option`) |
| Java | `null` | N/A (primitives cannot be null) |
| Go | `nil` (pointers/interfaces) | Zero value |

**Danger zone:** JavaScript distinguishes `null` from `undefined`. A JSON field set to `null` and a missing JSON field are different. Fusion's `json::parse` must decide: map both to `null`, or error on missing fields?

### Date / DateTime

| Language | Type | Format |
|---|---|---|
| Fusion | `DateTime` | ISO 8601 string or epoch milliseconds |
| Python | `datetime` | `datetime.isoformat()` |
| JavaScript | `Date` | ISO 8601 string or epoch ms |
| Rust | `chrono::DateTime<Utc>` | RFC 3339 |
| Java | `java.time.Instant` | ISO 8601 or epoch |
| Go | `time.Time` | RFC 3339 |

**Best practice:** Always transmit dates as epoch milliseconds (integer) or ISO 8601 strings. Never use locale-dependent formats.

### Array / List

| Language | Mutable? | Heterogeneous? | Index type |
|---|---|---|---|
| Fusion | `List<T>` | No (generic) | `int` |
| Python | `list` | Yes | `int` |
| JavaScript | `Array` | Yes | `number` |
| Rust | `Vec<T>` | No (typed) | `usize` |
| Java | `ArrayList<T>` | Yes (raw) | `int` |
| Go | `[]T` | No (typed) | `int` |

### Map / Dictionary

| Language | Ordered? | Key type constraint |
|---|---|---|
| Fusion | `Map<K, V>` | K must be `String` or `Int` |
| Python | `dict` (ordered since 3.7) | Any hashable |
| JavaScript | `Map` (ordered) or Object | String or Symbol (Object) |
| Rust | `HashMap<K, V>` | K: `Hash + Eq` |
| Java | `HashMap<K, V>` | K: `Object` |
| Go | `map[K]V` | K: `comparable` |

**Danger zone:** JSON object keys are always strings. If Fusion serializes `Map<int, V>` to JSON, keys become strings. Deserializing back in Python gives `{str: V}` — integer keys are lost.

### Binary Data

| Language | Type | JSON encoding |
|---|---|---|
| Fusion | `Bytes` | Base64 string |
| Python | `bytes` / `bytearray` | Base64 string |
| JavaScript | `Uint8Array` / `ArrayBuffer` | Base64 string |
| Rust | `Vec<u8>` / `&[u8]` | Base64 string |
| Java | `byte[]` | Base64 string |
| Go | `[]byte` | Base64 string |

**Note:** Binary data is the one type that consistently base64-encodes across all languages. Use Protobuf `bytes` for large binary payloads to avoid the 33% base64 overhead.

### Custom Structs

| Language | Representation | Schema required? |
|---|---|---|
| Fusion | `struct` | Compile-time |
| Python | `dataclass` / `pydantic.BaseModel` | Runtime (Pydantic validates) |
| JavaScript | Plain object / TypeScript interface | Compile-time (TS only) |
| Rust | `struct` + `#[derive(Serialize)]` | Compile-time (serde) |
| Java | POJO + annotations (Jackson) | Runtime (annotations) |
| Go | `struct` + `json` tags | Compile-time (tags) |

---

## Data Corruption Pitfalls

### Integer Overflow Differences

```python
# Python: no overflow, ever
x = 2**100  # Works fine
```

```javascript
// JavaScript: silent truncation
let x = 9007199254740993;
console.log(x === 9007199254740992); // true! Lost precision
```

```go
// Go: compile-time overflow on constants, runtime on variables
var x int32 = 2_147_483_648 // Overflow: overflows int32
```

**Rule:** When exchanging integers > 2^53, always serialize as strings or use a format that preserves precision (Protobuf `int64`, CBOR bignum tags).

### Float Precision Loss

```fusion
let a: float32 = 0.1;
let b: float32 = 0.2;
assert(a + b != 0.3); // This is TRUE — IEEE 754
```

```python
# Python's Decimal for exact arithmetic
from decimal import Decimal
Decimal('0.1') + Decimal('0.2') == Decimal('0.3')  # True
```

**Rule:** For monetary values, never use floats. Use integers (cents) or arbitrary-precision decimals. Fusion's `Decimal` type is lossless.

### String Encoding Misms

```
Fusion (UTF-8 bytes) → JavaScript (UTF-16) → Python (UTF-8)
```

If any link in the chain assumes single-byte characters, emoji and non-Latin scripts corrupt.

**Common failure:** Sending a Fusion string through a C library that uses `char*` (which may be ASCII or Latin-1) instead of `wchar_t*` or explicit UTF-8.

### Null Handling Differences

```json
{"name": null, "age": 25}
```

| Language | `name` field | `phone` field (missing) |
|---|---|---|
| Fusion | `null` | Error (if struct requires it) |
| Python | `None` | `None` (with `.get()`) |
| JavaScript | `null` | `undefined` |
| Go | `nil` (string pointer) | `""` (zero value) |

**Rule:** Always distinguish "explicitly null" from "absent" in your API contract. Fusion uses `Option<T>` for optional fields — prefer that over nullable.

### Date / Timezone Pitfalls

```python
# Python: naive vs aware datetime
from datetime import datetime
naive = datetime(2026, 7, 24, 12, 0)  # No timezone!
aware = datetime.now(datetime.timezone.utc)
```

```javascript
// JavaScript: local time by default
new Date().getHours(); // Depends on machine timezone
```

```go
// Go: time.Time carries location
t := time.Now() // UTC if created with time.Now()
```

**Rule:** Always transmit dates in UTC. Never transmit local times without an explicit offset. Use ISO 8601 with `Z` suffix or epoch milliseconds.

---

## Safe Conversion Patterns

### Explicit Type Conversion Functions

```fusion
// Fusion: explicit conversions with error handling
let int_val = match parse_int64(json_string) {
    Ok(v) => v,
    Err(e) => {
        log::warn!("Integer parse failed: {}", e);
        return Err(DataError::InvalidType);
    }
};
```

```python
# Python: validate before converting
def safe_int(value, default=0):
    try:
        return int(value)
    except (ValueError, TypeError):
        return default

def safe_float(value, default=0.0):
    try:
        return float(value)
    except (ValueError, TypeError):
        return default
```

```javascript
// JavaScript: explicit coercion
function safeBigInt(value) {
    if (typeof value === 'bigint') return value;
    if (typeof value === 'number' && Number.isInteger(value)) return BigInt(value);
    if (typeof value === 'string') return BigInt(value);
    throw new TypeError(`Cannot convert ${typeof value} to BigInt`);
}
```

### Validation at Boundaries

```fusion
// Fusion: validate incoming data at FFI boundaries
#[repr(C)]
struct SensorData {
    device_id: [u8; 64],
    temperature: f64,
    timestamp_ms: i64,
}

fn validate_sensor_data(raw: &SensorData) -> Result<&SensorData, ValidationError> {
    // Temperature sanity check
    if raw.temperature < -273.15 || raw.temperature > 10000.0 {
        return Err(ValidationError::OutOfRange("temperature"));
    }
    // Timestamp sanity check (must be after 2000-01-01)
    if raw.timestamp_ms < 946684800000 {
        return Err(ValidationError::InvalidTimestamp);
    }
    Ok(raw)
}
```

```python
# Python: Pydantic for automatic validation
from pydantic import BaseModel, Field, validator
from datetime import datetime

class SensorData(BaseModel):
    device_id: str = Field(..., max_length=64)
    temperature: float = Field(..., ge=-273.15, le=10000.0)
    timestamp_ms: int = Field(..., ge=946684800000)
    
    @validator('device_id')
    def validate_device_id(cls, v):
        if not v.isalnum():
            raise ValueError('device_id must be alphanumeric')
        return v
```

### Round-Trip Testing

The only way to catch serialization bugs is to round-trip data through every language boundary and verify.

```fusion
// Fusion: round-trip test
#[test]
fn json_roundtrip_preserves_types() {
    let original = SensorData {
        device_id: "SENS001".to_string(),
        temperature: 23.5,
        timestamp_ms: 1721827200000,
    };
    
    let json_str = json::stringify(&original)?;
    let restored: SensorData = json::parse(&json_str)?;
    
    assert_eq!(original.device_id, restored.device_id);
    assert!((original.temperature - restored.temperature).abs() < f64::EPSILON);
    assert_eq!(original.timestamp_ms, restored.timestamp_ms);
}
```

```python
# Python: round-trip test across serialization
import json
import msgpack
import pytest

def test_json_roundtrip():
    original = {"device_id": "SENS001", "temperature": 23.5, "timestamp_ms": 1721827200000}
    encoded = json.dumps(original)
    decoded = json.loads(encoded)
    assert decoded == original

def test_msgpack_roundtrip():
    original = {"device_id": "SENS001", "temperature": 23.5, "timestamp_ms": 1721827200000}
    encoded = msgpack.packb(original)
    decoded = msgpack.unpackb(encoded, raw=False)
    assert decoded == original
```

### Cross-Language Round-Trip Pattern

```
Fusion struct → Serialize → [Wire] → Deserialize → Python dict → Serialize → [Wire] → Deserialize → Fusion struct
                                                                      ↓
                                                              Assert equivalence
```

**Automation:** Generate test vectors in one language and verify them in all others. Store golden files (`test_data.json`, `test_data.pb`, `test_data.msgpack`) in version control.

---

## Decision Matrix: Which Format to Use

| Criterion | JSON | Protobuf | FlatBuffers | MessagePack | CBOR |
|---|---|---|---|---|---|
| Human readable | ✓ | ✗ | ✗ | ✗ | ✗ |
| Schema evolution | ✗ | ✓ | Partial | ✗ | ✓ (tags) |
| Zero-copy | ✗ | ✗ | ✓ | ✗ | ✗ |
| Streaming support | ✗ | ✓ (gRPC) | ✗ | ✓ | ✓ |
| Browser support | ✓ | ✓ (protobufjs) | ✓ | Limited | ✓ |
| IoT/constrained | Poor | Good | Good | Good | Excellent |
| Debugging ease | Excellent | Moderate | Hard | Hard | Moderate |

**Rule of thumb:** Start with JSON. Switch to Protobuf when performance demands it. Switch to FlatBuffers when you need zero-copy. Use CBOR for constrained devices.

---

## Summary

- JSON is the default choice; only deviate when you have a measured reason.
- Integer and float representation differs silently across languages — test round-trips.
- Never trust that "null" means the same thing everywhere.
- Dates should always be UTC epoch milliseconds or ISO 8601 with explicit timezone.
- Validate data at every language boundary — never assume the other side sent valid data.
- Round-trip testing across languages is the only reliable way to catch serialization bugs.
