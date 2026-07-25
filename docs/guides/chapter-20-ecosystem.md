> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 20: The Fusion Ecosystem

A language is only as good as the tools and libraries surrounding it. Fusion provides a rich, integrated ecosystem designed to make development a joy.

In this chapter, we will tour:
- **Cargo-Fusion**: The build system and package manager.
- **The Registry**: Discovering and publishing crates.
- **Editor Integration**: VS Code, LSP, and formatting.
- **Documentation**: Generating docs with `fusion doc`.
- **Workspaces**: Managing large multi-crate projects.

---

## 20.1 Is it Cargo or Fusion?

Fusion's package manager is heavily inspired by Rust's Cargo. In fact, the command `fusion` is your unified entry point for everything.

- `fusion new`: Create projects.
- `fusion build`: Compile.
- `fusion run`: Run.
- `fusion test`: Test.
- `fusion doc`: Documentation.
- `fusion publish`: Upload to registry.

### 20.1.1 fusion.toml

This is the manifest file.

```toml
[package]
name = "my_project"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"

[dependencies]
fusion_ai_core = "1.0"
serde = { version = "1.0", features = ["derive"] }

[profile.release]
opt-level = 3
lto = true
```text

---

## 20.2 The Fusion Registry

The **Fusion Registry** is the public repository of community crates.

### 20.2.1 Finding Crates

You can search for libraries via the generic names provided in the ecosystem roadmap (e.g., `fusion-web`, `fusion-sql`).
Add them to your project:

```bash
fusion add fusion-web
```text

### 20.2.2 Publishing Crates

1. Login: `fusion login <api-token>`
2. Publish: `fusion publish`

Fusion checks your package for uncommitted changes and uncompiled code before allowing a publish.

---

## 20.3 Documentation

Fusion has a built-in documentation generator.

```bash
fusion doc --open
```text

This commands reads the standard `///` comments in your code (supporting Markdown) and builds a static HTML site.

### 20.3.1 Doc Tests

Code blocks inside documentation comments are actually **tests**!

```fusion
/// Adds one to the number.
///
/// ```
/// let x = 5;
/// assert_eq!(fusion::add_one(x), 6);
/// ```
pub fn add_one(x: i32) -> i32 { x + 1 }
```text

Running `fusion test` will also run these code blocks. This guarantees your documentation never goes out of date.

---

## 20.4 Workspaces

As your project grows, you might want to split it into multiple library crates. A **workspace** helps manage dependencies for multiple packages.

File `fusion.toml` (root):

```toml
[workspace]
members = [
    "adder",
    "add-one",
]
```text

Directory structure:

```text
/
├── fusion.toml
├── adder/        (binary)
└── add-one/      (library)
```text

Running `fusion build` in the root builds the entire workspace.

---

## 20.5 Editor Support

Fusion provides a high-quality **Language Server Protocol (LSP)** implementation. This enables:
- Autocomplete
- Go to Definition
- Rename Symbol
- Type Hover information
- Inline errors

Install the **Fusion Extension** for VS Code for the best experience.

### 20.5.1 Formatting and Linting

- `fusion fmt`: Automatically formats your code to the standard style. No arguments about indentation!
- `fusion clippy`: A linter that catches common mistakes and offers suggestions for more idiomatic code.

---

## 20.6 Summary

The Fusion v2.0 Vortex ecosystem provides a "batteries-included" experience. You don't need to hunt for a build tool, a doc generator, a test runner, or a linter. They are all standardized and built-in.

In the final chapter, we will combine everything we have learned—Classical structs, AI Tensors, Quantum Circuits, Security, and Ecosystem tools—into one grand project.

---

[Next: Chapter 21 - Final Project: A Tri-brid Portfolio Optimizer →](./chapter-21-final-project.md)