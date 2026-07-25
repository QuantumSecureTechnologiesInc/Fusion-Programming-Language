# Fusion Build System Configuration

## Build Policy

**REQUIRED:** All Fusion projects must use **Fusion Flux Engine** for dependency resolution and builds.

### Strict Enforcement Mode

By default, the project operates in **strict mode** which prevents direct `cargo` usage.

```bash
# ❌ PROHIBITED
cargo build
cargo test
cargo run

# ✅ REQUIRED
fusion build
fusion test
fusion run
```

## Environment Variables

### Core Configuration

```bash
# Enable/Disable Fusion Flux Engine
export FUSION_FLUX_ENABLED=true          # Default: true

# Strict enforcement mode
export FUSION_STRICT_MODE=true           # Default: true
                                         # false = warnings only

# Allow cargo fallback (emergency only)
export ALLOW_CARGO_FALLBACK=false        # Default: false
                                         # true = use cargo if Flux fails
```

### Advanced Configuration

```bash
# GPU acceleration
export FUSION_CUDA_ENABLE=true           # Default: true

# Cache configuration
export FUSION_CACHE_PATH=./.fusion/cache_db
export FUSION_CACHE_MAX_SIZE_MB=1024

# Registry
export FUSION_REGISTRY_URL=https://registry.fusionlang.dev

# Telemetry
export FUSION_TELEMETRY_ENABLED=true
```

## How It Works

### 1. Automatic Enforcement

The `.scripts/enforce-flux-build.*` scripts automatically intercept build commands:

```bash
# When you run:
cargo build

# The script:
1. Detects cargo usage ❌
2. Checks FUSION_STRICT_MODE
3. If strict: BLOCKS and shows error
4. If not strict: Warns and allows
```

### 2. Proper Usage

```bash
# Use Fusion CLI (recommended)
fusion build              # Builds with Fusion Flux Engine
fusion build --release    # Release build
fusion test               # Run tests
fusion run                # Build and run
fusion check              # Fast check without building
```

### 3. Emergency Override

If Fusion Flux Engine is not working:

```bash
# Temporary disable for this session
export FUSION_STRICT_MODE=false
export ALLOW_CARGO_FALLBACK=true

cargo build  # Now allowed (with warning)
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    env:
      FUSION_FLUX_ENABLED: true
      FUSION_STRICT_MODE: true
      
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Fusion Flux Engine
        run: |
          cd runtime
          cargo build -p fusion_flux_resolve
          
      - name: Build Project (via Fusion)
        run: |
          # This will use Fusion Flux automatically
          fusion build
          
      - name: Run Tests
        run: fusion test
```

### Local Development

Add to your shell profile (`.bashrc`, `.zshrc`, or PowerShell profile):

```bash
# Enforce Fusion Flux globally
export FUSION_FLUX_ENABLED=true
export FUSION_STRICT_MODE=true

# Alias cargo to enforcement script
alias cargo='$HOME/Projects/Fusion/scripts/enforce-flux-build.sh'
```

## Policy Exemptions

### When Cargo is Allowed

1. **Building Fusion Flux itself:**
   ```bash
   cd runtime
   cargo build -p fusion_flux_resolve  # OK - building the engine
   ```

2. **Emergency maintenance:**
   ```bash
   export FUSION_STRICT_MODE=false  # Temporary disable
   cargo build  # Allowed with warning
   ```

3. **External dependencies:**
   ```bash
   # Building Rust crates that aren't Fusion projects
   cd external/some-rust-crate
   cargo build  # OK - not a Fusion project
   ```

## Enforcement Levels

### Level 1: Informational (FUSION_STRICT_MODE=false)

```
⚠️  WARNING: Using cargo directly
   Consider using 'fusion build' instead
   
Proceeding with cargo build...
```

### Level 2: Strict (FUSION_STRICT_MODE=true, ALLOW_CARGO_FALLBACK=false)

```
❌ POLICY VIOLATION: Direct cargo usage detected!
   
   Command attempted: cargo build
   
   ✅ USE INSTEAD:
      fusion build
      
Build blocked. Exiting.
```

### Level 3: Strict with Fallback (FUSION_STRICT_MODE=true, ALLOW_CARGO_FALLBACK=true)

```
⚠️  Fusion Flux Engine not available
   Falling back to cargo (ALLOW_CARGO_FALLBACK=true)
   
Proceeding with cargo build...
```

## Rollout Plan

### Phase 1: Warning Mode (Current)
- `FUSION_STRICT_MODE=false` by default
- Warnings on cargo usage
- Education and migration

### Phase 2: Strict Mode (Target: v0.4.0)
- `FUSION_STRICT_MODE=true` by default
- Block cargo usage
- Fallback available if needed

### Phase 3: Full Enforcement (Target: v1.0.0)
- `FUSION_STRICT_MODE=true` mandatory
- No fallback allowed
- Cargo only for Flux itself

## Monitoring and Compliance

### Check Compliance

```bash
# Verify Flux is being used
.scripts/enforce-flux-build.ps1 check

# Expected output:
✅ Fusion Flux Engine: Available
✅ Strict Mode: Enabled
✅ Project Status: Compliant
```

### Audit Builds

```bash
# Review build logs
cat .fusion/build-audit.log

# Sample output:
2025-12-12T13:40:00 [INFO] Build initiated
2025-12-12T13:40:00 [INFO] Fusion Flux Engine: v0.3.0
2025-12-12T13:40:00 [INFO] Cache hit rate: 95%
2025-12-12T13:40:01 [INFO] Build complete (1.2s)
```

## --no-verify Bypass Auditing

`git commit --no-verify` skips the pre-commit hook entirely. That is sometimes
necessary for emergencies, but a silent bypass on production-impacting code is a
risk. Fusion therefore **audits every `--no-verify` commit** and **requires
explicit approval** when the bypass touches a high-risk area.

### High-risk areas

| Area | Path prefix | Why it is production-impacting |
| --- | --- | --- |
| Compiler crate | `crates/fuc/` | Compiler changes can affect every Fusion program |
| Runtime | `runtime/` | Runtime changes affect program execution |
| Standard library | `stdlib/` | Stdlib changes ship to every downstream consumer |

### How it works

Because `--no-verify` skips pre-commit, the bypass cannot be caught from inside
pre-commit. Detection happens in a **post-commit** hook using a freshness marker:

1. On a **successful** pre-commit run, the hook writes a marker
   (`<git-dir>/fusion_precommit_marker`) bound to the current HEAD **and** the
   staged tree hash (`git write-tree`).
2. The **post-commit** hook (`.githooks/post-commit`) compares that marker
   against the new commit's parent and committed tree. A missing or non-matching
   marker means pre-commit did not run for this commit, i.e. `--no-verify` was
   used. Binding the tree hash prevents a stale marker (from an aborted commit)
   from masking a later bypass with different content.
3. The bypass is appended to the audit trail. If any changed file is high-risk,
   the record is marked `PENDING_APPROVAL` and an escalation notice is printed.
4. The **pre-commit escalation gate** refuses to start any new commit while an
   unapproved high-risk bypass is pending — enforcing explicit approval.

The hooks work under both Git for Windows (bash `.githooks/pre-commit` /
`post-commit`) and manual PowerShell runs (`.ps1` variants). Enable them with:

```bash
git config core.hooksPath .githooks
```

### Reviewing the audit trail

The trail is a JSON-lines file (matched by the repo's `*.log` ignore rule, so it
stays a local, reviewable record and is never committed by the hook that writes
it):

```bash
cat .fusion/audit/no-verify-bypass.log
```

Each line is one event. `event=bypass` records the commit, branch, author,
changed files, the high-risk files, and status (`LOGGED` or `PENDING_APPROVAL`).
`event=approval` records the reviewer and justification.

### Approving a high-risk bypass (owner only)

Until a pending high-risk bypass is approved, further commits are blocked. Record
an explicit approval with the full commit SHA shown in the trail:

```powershell
# Windows / PowerShell
powershell -ExecutionPolicy Bypass -File scripts\approve_bypass.ps1 `
    -Commit <full-sha> -Justification "Emergency hotfix, reviewed by @owner"
```

```bash
# Linux / macOS / Git Bash
bash scripts/bypass_audit.sh approve <full-sha> "Emergency hotfix, reviewed by @owner"
```

Approval is itself recorded in the audit trail, so every production-impacting
bypass has a reviewable who/why entry.

## Troubleshooting

### "Fusion Flux Engine not found"

```bash
# Build the engine first
cd runtime
cargo build -p fusion_flux_resolve

# Verify it's built
ls runtime/target/debug/fusion_flux_resolve*
```

### "Policy violation" but I need cargo

```bash
# Temporary disable (this session only)
export FUSION_STRICT_MODE=false
cargo build

# Or allow fallback
export ALLOW_CARGO_FALLBACK=true
cargo build
```

### "Fusion command not found"

```bash
# Build Fusion CLI
cd cmd/fusion
cargo build --release

# Add to PATH
export PATH="$PATH:$PWD/target/release"

# Or create symlink
ln -s $PWD/target/release/fusion /usr/local/bin/fusion
```

## FAQ

**Q: Why enforce Fusion Flux?**  
A: Ensures consistent, fast, parallel builds across all team members and CI/CD.

**Q: What if Flux has a bug?**  
A: Use `ALLOW_CARGO_FALLBACK=true` temporarily while the bug is fixed.

**Q: Can I use both Fusion and cargo?**  
A: No in strict mode. Fusion Flux handles everything cargo does, but better.

**Q: How do I migrate existing builds?**  
A: Replace `cargo build` → `fusion build`. Flux handles the rest automatically.

**Q: Does this break IDE integration?**  
A: No. Configure your IDE to use `fusion build` instead of `cargo build`.

## Support

For issues or questions:
- GitHub Issues: `https://github.com/QuantumSecureTechnologiesInc/Fusion/issues`
- Documentation: `docs/guides/`
- Build Policy: This file

---

**Last Updated:** 2025-12-12  
**Policy Version:** 1.0  
**Enforcement:** STRICT
