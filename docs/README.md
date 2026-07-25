# Fusion v2.0 Vortex — Documentation Index

## Quick Start

- [Quick Start Guide](./guides/QuickStartGuide.md) — Get up and running
- [Language Reference](./reference/FUSION_LANGUAGE_SPEC.md) — Core language spec
- [Contributing](./guides/CONTRIBUTING.md) — How to contribute

---

## Current Documentation

### Guides (`guides/`)

The 16-chapter book (chapters + appendices):

| Chapter | File |
|---------|------|
| Getting Started | [ch01-getting-started.md](./guides/ch01-getting-started.md) |
| Syntax | [ch02-syntax.md](./guides/ch02-syntax.md) |
| Structs & Enums | [ch03-structs-enums.md](./guides/ch03-structs-enums.md) |
| Memory Safety | [ch04-memory-safety.md](./guides/ch04-memory-safety.md) |
| Generics | [ch05-generics.md](./guides/ch05-generics.md) |
| Standard Library | [ch06-standard-library.md](./guides/ch06-standard-library.md) |
| Post-Quantum Crypto | [ch07-post-quantum-crypto.md](./guides/ch07-post-quantum-crypto.md) |
| Quantum Computing | [ch08-quantum-computing.md](./guides/ch08-quantum-computing.md) |
| Machine Learning | [ch09-machine-learning.md](./guides/ch09-machine-learning.md) |
| Concurrency | [ch10-concurrency.md](./guides/ch10-concurrency.md) |
| WebAssembly | [ch11-webassembly.md](./guides/ch11-webassembly.md) |
| Tooling | [ch12-tooling.md](./guides/ch12-tooling.md) |
| Advanced Topics | [ch13-advanced.md](./guides/ch13-advanced.md) |
| Examples | [ch14-examples.md](./guides/ch14-examples.md) |
| Reference | [ch15-reference.md](./guides/ch15-reference.md) |
| Appendix A — Keywords | [appendix-a-keywords.md](./guides/appendix-a-keywords.md) |
| Appendix B — Stdlib | [appendix-b-stdlib.md](./guides/appendix-b-stdlib.md) |
| Appendix C — Quantum Gates | [appendix-c-quantum-gates.md](./guides/appendix-c-quantum-gates.md) |

Additional guides:

- [Developer Guide](./guides/Developer_Guide.md) — Architecture and internals
- [Collections Guide](./guides/Collections_Complete_Guide.md) — Collection types
- [VSC CLI Training](./guides/Fusion_VSC_CLI_Coder_Training_Manual.md) — IDE integration
- [Phase 4 Testing](./guides/Phase4_Testing_Guide.md) — Test procedures
- [Terminal Browser Guide](./guides/terminal_browser_developer_guide.md) — Terminal UI

### Reference (`reference/`)

- [FUSION_LANGUAGE_SPEC.md](./reference/FUSION_LANGUAGE_SPEC.md) — Language specification
- [LANGUAGE_SPEC.md](./reference/LANGUAGE_SPEC.md) — Language spec (alternate)
- [FLUX_QUICK_REF.md](./reference/FLUX_QUICK_REF.md) — Flux quick reference
- [ARCHETYPE_QUICKREF.md](./reference/ARCHETYPE_QUICKREF.md) — Archetype reference
- [Runtime Quick Reference](./reference/FUSION_RUNTIME_QUICK_REFERENCE.md) — Runtime API
- [Visual Compiler Reference](./reference/FUSION_VISUAL_COMPILER.md) — Visual compiler
- [Fusion vs Rust](./reference/FUSION_VS_RUST.md) — Comparison with Rust

### Technical (`technical/`)

- [Compiler Feature Gap Analysis](./technical/compiler-feature-gap-analysis.md)
- [ABI Specification](./technical/Fusion_ABI_Specification.md)
- [Substrate Complete](./technical/FUSION_SUBSTRATE_COMPLETE.md)
- [TOML Complete Guide](./technical/FUSION_TOML_COMPLETE_GUIDE.md)
- [RPC Protocol Specification](./technical/RPC_Protocol_Specification.md)
- [Security Guarantees](./technical/SECURITY_GUARANTEES.md)
- [Known Limits](./technical/Known_Limits.md)
- [Network Production](./technical/Network_Production_Deployment.md)
- [Enhanced LSP](./technical/Enhanced_LSP_70_Percent.md)
- [Core Merge Audit](./technical/FUSION_CORE_MERGE_AUDIT.md)
- [Runtime vs Tokio](./technical/FUSION_RUNTIME_VS_TOKIO_ANALYSIS.md)
- [Extension Auth](./technical/EXTENSION_AUTH_SYSTEM.md)
- [Parser Enhancement](./technical/Parser_Enhancement_Implementation.md)
- [Standard Library Phase 2](./technical/Standard_Library_Phase2_Implementation.md)

### Other Documentation

- [Top-level docs](./bootstrap-compiler-codebase.md) — Compiler codebase overview
- [Fusion vs Rust](./FUSION_VS_RUST.md) — High-level comparison
- [Vortex Borrow Checker](./vortex-borrow-checker.md) — Borrow checker design
- [LINTING.md](./LINTING.md) — Linting rules

---

## Archived Documentation

The following outdated/aspirational documents have been moved to `docs/archive/`:

**Guides (archived):**
- `FUSION_COMPLETE_GUIDEBOOK.md` — Superseded by the 16-chapter book
- `User_Guide.md` — Replaced by chapter-based guides
- `Product_Guide.md` — Aspirational content, no longer current
- `GUIDEBOOK_COMPLETION_REPORT.md` — Historical completion report
- `WHAT_IS_FLUX_RESOLVE.md` — Consolidated elsewhere

**Technical (archived):**
- `RELEASE_NOTES.md` — Outdated release notes
- `FUSION_SUBSTRATE_IMPLEMENTATION_PLAN.md` — Plan completed/moot
- `Core_Type_System_Deliverables.md` — Deliverables fulfilled
- `ENHANCED_FEATURES.md` — Aspirational feature list

**Features (archived):**
- `Supernova_Runtime.md` — Superseded by actual runtime docs
- `Visual_Compiler.md` — Reference version moved to `reference/`
- `Quantum_Integration.md` — Consolidated into quantum chapter

**Roadmap (archived):**
- `WHATS_LEFT_TODO.md` — Outdated task list
- `THE_REALITY_CHECK.md` — Historical assessment
- `THE_FINAL_VERDICT.md` — Historical verdict

---

## Directory Structure

```
docs/
├── README.md              ← you are here
├── guides/                ← user & developer guides, 16-chapter book
├── reference/             ← language spec, quick references
├── technical/             ← architecture, specs, audits
├── archive/               ← outdated/aspirational docs
├── design/                ← design documents
├── plans/                 ← implementation plans
├── reports/               ← audit and review reports
└── ...
```
