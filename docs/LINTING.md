# Fusion Linting Standards

## Quick Reference

### Preset Profiles
- **Minimal**: Fewer warnings, best for rapid prototyping
- **Standard**: Balanced approach (current default)
- **Strict**: Maximum code quality enforcement

### Switch Profiles
```powershell
Copy-Item .lint-profiles\<minimal|standard|strict>-lints.toml Cargo.toml -Force
```

### Current Workspace Defaults (Standard)
- Rust: `unsafe_code`, `unused_imports`, `unused_variables`, `dead_code` = warn
- Clippy: `all` = warn (with common patterns allowed)

See [`.lint-profiles/README.md`](./.lint-profiles/README.md) for full documentation.
