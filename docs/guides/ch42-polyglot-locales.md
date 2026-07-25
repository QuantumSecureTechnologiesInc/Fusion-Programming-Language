# Chapter 42: Locales, Timezones & Data Consistency

A timestamp is "2024-01-15 14:30:00" — but is that UTC, EST, or the server's local time? In polyglot systems, Python might parse it as naive, Rust interprets it as UTC, and Java assumes it's in the server's timezone. This chapter eliminates timezone-related bugs forever.

## Pin to UTC + en.UTF-8

The single most important rule for polyglot systems: **all internal communication uses UTC and UTF-8**. No exceptions.

### Container Entrypoint Configuration

```bash
#!/bin/bash
# entrypoint.sh — Force UTC and UTF-8 in all containers

# Set timezone to UTC
export TZ=UTC
ln -snf /usr/share/zoneinfo/$TZ /etc/localtime
echo $TZ > /etc/timezone

# Set locale to UTF-8
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export LANGUAGE=en_US:en

# Verify locale is correct
locale
# LANG=en_US.UTF-8
# LANGUAGE=en_US:en
# LC_CTYPE="en_US.UTF-8"
# LC_NUMERIC="en_US.UTF-8"
# LC_TIME="en_US.UTF-8"
# LC_COLLATE="en_US.UTF-8"
# LC_MONETARY="en_US.UTF-8"
# LC_MESSAGES="en_US.UTF-8"
# LC_PAPER="en_US.UTF-8"
# LC_NAME="en_US.UTF-8"
# LC_ADDRESS="en_US.UTF-8"
# LC_TELEPHONE="en_US.UTF-8"
# LC_MEASUREMENT="en_US.UTF-8"
# LC_IDENTIFICATION="en_US.UTF-8"
# LC_ALL=en_US.UTF-8

# Verify timezone
date
# Mon Jan 15 14:30:00 UTC 2024

exec "$@"
```

### Docker Configuration

```dockerfile
# Dockerfile — Locale and timezone setup
FROM ubuntu:22.04

ENV TZ=UTC
ENV LANG=en_US.UTF-8
ENV LANGUAGE=en_US:en
ENV LC_ALL=en_US.UTF-8

RUN apt-get update && apt-get install -y \
    locales \
    && locale-gen en_US.UTF-8 \
    && update-locale LANG=en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Verify
RUN locale && date
```

### Docker Compose Configuration

```yaml
# docker-compose.yml
services:
  api:
    image: fusion-api:latest
    environment:
      - TZ=UTC
      - LANG=en_US.UTF-8
      - LC_ALL=en_US.UTF-8
    volumes:
      - /etc/localtime:/etc/localtime:ro
      - /usr/share/zoneinfo:/usr/share/zoneinfo:ro

  worker:
    image: fusion-worker:latest
    environment:
      - TZ=UTC
      - LANG=en_US.UTF-8
      - LC_ALL=en_US.UTF-8
```

## ISO-8601 Parsing

ISO-8601 is the only date/time format you should ever parse in a polyglot system. Everything else is ambiguous.

### The ISO-8601 Format

```
2024-01-15T14:30:00Z          — UTC (no offset)
2024-01-15T14:30:00+00:00     — UTC (explicit offset)
2024-01-15T14:30:00+05:30     — India Standard Time
2024-01-15T14:30:00-08:00     — Pacific Standard Time
2024-01-15T14:30:00.123456Z   — With microseconds
2024-01-15T14:30:00.123Z      — With milliseconds
```

### Language-Specific Parsing

```python
# Python — datetime.fromisoformat (Python 3.7+)
from datetime import datetime, timezone

# Parse ISO-8601 with timezone
dt = datetime.fromisoformat("2024-01-15T14:30:00+00:00")
assert dt.tzinfo is not None  # Timezone-aware

# Convert to UTC
dt_utc = dt.astimezone(timezone.utc)

# BAD: Naive datetime (no timezone info)
dt_naive = datetime.fromisoformat("2024-01-15T14:30:00")
# This is DANGEROUS — it has no timezone info

# GOOD: Always validate timezone awareness
def parse_timestamp(ts: str) -> datetime:
    dt = datetime.fromisoformat(ts)
    if dt.tzinfo is None:
        raise ValueError(f"Naive datetime not allowed: {ts}")
    return dt.astimezone(timezone.utc)
```

```rust
// Rust — chrono crate
use chrono::{DateTime, Utc, TimeZone};

// Parse ISO-8601 with timezone
let dt: DateTime<Utc> = "2024-01-15T14:30:00Z"
    .parse()
    .expect("Failed to parse datetime");

// Parse with offset
let dt_with_offset: DateTime<Utc> = "2024-01-15T14:30:00+05:30"
    .parse()
    .expect("Failed to parse datetime")
    .with_timezone(&Utc);

// BAD: Parsing without timezone
// let dt_naive = NaiveDateTime::parse_from_str("2024-01-15T14:30:00", "%Y-%m-%dT%H:%M:%S");
// This is DANGEROUS
```

```go
// Go — time package
import "time"

// Parse ISO-8601 with timezone
dt, err := time.Parse(time.RFC3339, "2024-01-15T14:30:00+00:00")
if err != nil {
    log.Fatal(err)
}

// Convert to UTC
dtUTC := dt.UTC()

// BAD: Parsing without timezone
// dtNaive, _ := time.Parse("2006-01-02T15:04:05", "2024-01-15T14:30:00")
// This defaults to UTC but is ambiguous
```

```javascript
// JavaScript — Date (always UTC internally)
const dt = new Date("2024-01-15T14:30:00Z");

// Convert to ISO-8601 string
console.log(dt.toISOString()); // 2024-01-15T14:30:00.000Z

// BAD: Parsing without timezone
const dtNaive = new Date("2024-01-15T14:30:00");
// This interprets as local time — DANGEROUS
```

### Cross-Language Timestamp Validation

```python
# validate_timestamp.py — Shared timestamp validation
import re
from datetime import datetime, timezone

# ISO-8601 regex (strict)
ISO8601_PATTERN = re.compile(
    r'^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$'
)

def validate_timestamp(ts: str) -> datetime:
    """Validate and parse ISO-8601 timestamp."""
    if not ISO8601_PATTERN.match(ts):
        raise ValueError(f"Invalid ISO-8601 format: {ts}")

    dt = datetime.fromisoformat(ts)

    if dt.tzinfo is None:
        raise ValueError(f"Timestamp must include timezone: {ts}")

    # Always convert to UTC for internal storage
    return dt.astimezone(timezone.utc)

# Test cases
assert validate_timestamp("2024-01-15T14:30:00Z")
assert validate_timestamp("2024-01-15T14:30:00+00:00")
assert validate_timestamp("2024-01-15T14:30:00.123456Z")

try:
    validate_timestamp("2024-01-15 14:30:00")  # Missing T
except ValueError as e:
    print(f"Correctly rejected: {e}")

try:
    validate_timestamp("2024-01-15T14:30:00")  # No timezone
except ValueError as e:
    print(f"Correctly rejected: {e}")
```

## Timezone Drift

Timezone drift occurs when different services interpret timestamps differently.

### Common Drift Scenarios

```
Scenario 1: Server timezone vs UTC
  Server A (UTC):     2024-01-15 14:30:00 UTC
  Server B (EST):     2024-01-15 14:30:00 EST = 19:30:00 UTC
  Result: 5-hour discrepancy

Scenario 2: DST transitions
  Before DST:         2024-03-10 01:30:00 EST
  DST begins:         2024-03-10 03:00:00 EDT (clocks skip 2 AM)
  After DST:          2024-03-10 03:30:00 EDT
  Result: 1 hour "disappears"

Scenario 3: Timezone database outdated
  Old database:       Rules from 2020
  New database:       Rules from 2024
  Result: Historical timestamps may shift
```

### Preventing Drift

```python
# timezone_safety.py — Prevent timezone drift
from datetime import datetime, timezone
from zoneinfo import ZoneInfo  # Python 3.9+

def to_utc(dt: datetime, source_tz: str = None) -> datetime:
    """Convert any datetime to UTC, handling ambiguity."""
    if dt.tzinfo is None:
        if source_tz:
            # Assume source timezone and convert
            dt = dt.replace(tzinfo=ZoneInfo(source_tz))
        else:
            raise ValueError("Naive datetime without source timezone")

    return dt.astimezone(timezone.utc)

def utc_now() -> datetime:
    """Get current time in UTC."""
    return datetime.now(timezone.utc)

def format_iso8601(dt: datetime) -> str:
    """Format datetime as ISO-8601 with timezone."""
    if dt.tzinfo is None:
        raise ValueError("Cannot format naive datetime")
    return dt.isoformat()

# Example usage
now = utc_now()
print(format_iso8601(now))  # 2024-01-15T14:30:00+00:00

# Convert from EST to UTC
est_time = datetime(2024, 1, 15, 14, 30, 0, tzinfo=ZoneInfo("America/New_York"))
utc_time = to_utc(est_time)
print(format_iso8601(utc_time))  # 2024-01-15T19:30:00+00:00
```

### Database Storage

```sql
-- PostgreSQL: Always use TIMESTAMP WITH TIME ZONE
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    -- GOOD: TIMESTAMP WITH TIME ZONE (stores as UTC internally)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- BAD: TIMESTAMP WITHOUT TIME ZONE (ambiguous!)
    -- created_at TIMESTAMP NOT NULL DEFAULT NOW()
    payload JSONB
);

-- Query with timezone conversion
SELECT
    id,
    event_type,
    created_at,
    -- Convert to specific timezone for display
    created_at AT TIME ZONE 'America/New_York' AS created_at_eastern,
    created_at AT TIME ZONE 'Asia/Tokyo' AS created_at_tokyo
FROM events
WHERE created_at > NOW() - INTERVAL '1 day';
```

### Fusion.toml Locale Configuration

```toml
# Fusion.toml: locale and timezone settings
[locale]
# Force UTC for all internal operations
timezone = "UTC"
locale = "en_US.UTF-8"

[locale.validation]
# Reject naive datetimes in FFI boundaries
reject_naive_datetimes = true
# Require ISO-8601 format
require_iso8601 = true
# Allow timezone conversion at boundaries
allow_timezone_conversion = true

[locale.display]
# User-facing timezone (for UI only, not storage)
default_timezone = "America/New_York"
# Supported display timezones
supported_timezones = [
    "UTC",
    "America/New_York",
    "America/Los_Angeles",
    "Europe/London",
    "Asia/Tokyo",
    "Asia/Shanghai",
]

[locale.formats]
# Date format for display (not storage)
date_format = "YYYY-MM-DD"
time_format = "HH:mm:ss"
datetime_format = "YYYY-MM-DDTHH:mm:ssZ"
```

## Best Practices

1. **Store everything in UTC** — convert to local time only for display
2. **Use ISO-8601** for all timestamp serialization
3. **Always include timezone** — never allow naive datetimes at boundaries
4. **Pin container timezone** to UTC via environment variables
5. **Keep timezone databases updated** — DST rules change
6. **Validate timestamps at system boundaries** — reject malformed input
7. **Use `TIMESTAMPTZ` in PostgreSQL** — never `TIMESTAMP`
8. **Test DST transitions** — they break naive implementations

## Summary

Timezone handling in polyglot systems follows three rules:
1. **UTC everywhere internally**
2. **ISO-8601 everywhere externally**
3. **Validate at boundaries, display locally**

Get these right and you'll never have a "which timezone is this?" conversation again.
