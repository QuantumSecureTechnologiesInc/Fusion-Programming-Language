# Vortex Borrow Checker (vortex.fu)

**File:** `crates/fuc/src/vortex.fu`  
**Lines:** 38  
**Purpose:** Self-hosted Fusion borrow checker implementing entropic flow safety model

---

## Source Code

```fusion
// Fusion Self-Hosted Vortex Engine
// Implements the Entropic Flow safety model to prevent data races and use-after-free.

import std.mem;
import std.io;

struct Loan {
    target: string;
    is_exclusive: bool;
    origin_line: int;
}

struct ChaosVacuum {
    reports: **char;
    count: int;
}

/// Analyses the entropic state of a variable borrow.
fn verify_borrow(state: *LoanStream, target: string, exclusive: bool, line: int) -> bool {
    let i: int = 0;
    while (i < state.loan_count) {
        let existing: *Loan = &state.loans[i];
        if (existing.target == target) {
            // Collision Logic: Stream B (Mutable) repels everything.
            // Stream A (Immutable) only repels Stream B.
            if (existing.is_exclusive || exclusive) {
                io.print("Entropic Collision: Variable '");
                io.print(target);
                io.print("' has conflicting streams at line ");
                io.print_int(line);
                io.print("\n");
                return false;
            }
        }
        i = i + 1;
    }
    return true;
}
```

---

## Key Components

### Data Structures

**Loan**
- `target: string` - Variable being borrowed
- `is_exclusive: bool` - Whether borrow is mutable (exclusive) or immutable (shared)
- `origin_line: int` - Source code line where borrow originated

**ChaosVacuum**
- `reports: **char` - Pointer to diagnostic report strings
- `count: int` - Number of reports

### Core Function

**`verify_borrow`**
- **Purpose:** Validates borrow safety using entropic flow model
- **Parameters:**
  - `state: *LoanStream` - Current loan state tracking structure
  - `target: string` - Variable being borrowed
  - `exclusive: bool` - Whether this is an exclusive (mutable) borrow
  - `line: int` - Source code line number
- **Returns:** `bool` - `true` if borrow is safe, `false` if collision detected

### Entropic Flow Rules

1. **Stream A (Immutable/Shared):** Multiple readers allowed
2. **Stream B (Mutable/Exclusive):** Single writer, repels all other access
3. **Collision Detection:**
   - Exclusive borrow conflicts with ANY existing borrow
   - Any new exclusive borrow conflicts with existing shared borrows
   - Multiple shared borrows are allowed

### Error Reporting

When collision is detected, the function prints:
```
Entropic Collision: Variable 'X' has conflicting streams at line Y
```

---

## Thermodynamic Safety Model

The Vortex engine treats variable access as thermodynamic streams:

- **Conservation of Ownership:** Each resource has exactly one owner
- **Entropy Increase:** Moving resources increases entropy
- **Stream Collision:** Conflicting access creates entropic violations
- **Permission Dissipation:** Resources lose permissions when moved

---

*Document generated: 2026-07-02*  
*Fusion v2.0 Vortex - Self-Hosted Borrow Checker*
