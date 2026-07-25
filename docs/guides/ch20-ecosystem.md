# Chapter 20: Ecosystem

Fusion has a growing ecosystem of packages, tools, and community resources. This chapter covers the package registry, community libraries, tooling integration, IDE support, and documentation.

## Package Registry

Fusion uses the Forge package manager for dependency management:

```toml
# Fusion.toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"

[dependencies]
std = "2.0"
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Publishing Packages

```bash
# Login to registry
forge login

# Publish package
forge publish

# yank a version
forge yank 0.1.0
```

### Package Structure

```
my-package/
├── Fusion.toml
├── src/
│   ├── lib.fusion
│   └── main.fusion
├── tests/
│   └── integration_test.fusion
├── benches/
│   └── benchmark.fusion
└── README.md
```

## Community Libraries

### Web Frameworks

```fusion
// Actix-style web framework
use web::{get, post, route};

#[route("/users", method = "GET")]
async fn get_users() -> Result<Vec<User>, Error> {
    let users = db::query("SELECT * FROM users").await?;
    Ok(users)
}

#[route("/users", method = "POST")]
async fn create_user(user: NewUser) -> Result<User, Error> {
    let created = db::insert(user).await?;
    Ok(created)
}
```

### Serialization

```fusion
// Serde-style serialization
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// JSON serialization
let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into() };
let json = serde_json::to_string(&user)?;
let parsed: User = serde_json::from_str(&json)?;

// Binary serialization
let bytes = bincode::serialize(&user)?;
let parsed: User = bincode::deserialize(&bytes)?;
```

### Async Runtime

```fusion
// Tokio-style async runtime
use async_std::task;
use std::future::Future;

async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    task::block_on(async {
        let data = fetch_data("https://api.example.com/data").await?;
        println!("{}", data);
        Ok(())
    })
}
```

### Database Libraries

```fusion
// SQLx-style database access
use sqlx::postgres::PgPoolOptions;

#[derive(sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

async fn get_user(pool: &PgPool, id: i64) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}
```

## Tooling Integration

### Language Server Protocol (LSP)

Fusion provides full LSP support:

```json
// .vscode/settings.json
{
    "fusion.lsp.enabled": true,
    "fusion.lsp.diagnostics": true,
    "fusion.lsp.inlayHints": true,
    "fusion.formatting.enabled": true
}
```

### Code Formatter

```bash
# Format code
fusion fmt

# Check formatting
fusion fmt --check

# Format specific files
fusion fmt src/*.fusion
```

### Linter

```bash
# Run linter
fusion lint

# Fix auto-fixable issues
fusion lint --fix

# Run specific lints
fusion lint --warn unused_variables
```

### Test Framework

```fusion
// Built-in test framework
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }
    
    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_division_by_zero() {
        let _ = 1 / 0;
    }
    
    #[async_test]
    async fn test_async_operation() {
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

```bash
# Run tests
fusion test

# Run specific test
fusion test test_addition

# Run tests with coverage
fusion test --coverage
```

### Profiler

```bash
# Profile application
fusion profile -- ./my-app

# Generate flame graph
fusion profile --flamegraph -- ./my-app

# Profile specific function
fusion profile --function main -- ./my-app
```

## IDE Support

### VS Code Extension

Features:
- Syntax highlighting
- Code completion
- Go to definition
- Find references
- Inline hints
- Error diagnostics
- Code formatting
- Refactoring tools

### IntelliJ Plugin

Features:
- Smart code completion
- Code navigation
- Refactoring support
- Debugging integration
- Version control integration

### Vim/Neovim

Features:
- Syntax highlighting via treesitter
- LSP integration via coc.nvim or built-in LSP
- Code completion
- Error checking

## Documentation

### Doc Comments

```fusion
/// Adds two numbers together.
///
/// # Arguments
///
/// * `a` - The first number
/// * `b` - The second number
///
/// # Returns
///
/// The sum of `a` and `b`.
///
/// # Examples
///
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Generate Documentation

```bash
# Generate documentation
fusion doc

# Open documentation in browser
fusion doc --open

# Generate documentation for specific package
fusion doc --package my-package
```

### Documentation Hosting

- Built-in documentation server
- Static site generation
- Integration with docs.fusion-lang.org
- Custom themes and styling

## Community Resources

### Official Resources

- **Documentation**: docs.fusion-lang.org
- **Package Registry**: crates.fusion-lang.org
- **Playground**: play.fusion-lang.org
- **Blog**: blog.fusion-lang.org

### Community Channels

- **Discord**: discord.gg/fusion
- **GitHub**: github.com/fusion-lang/fusion
- **Reddit**: r/FusionLang
- **Stack Overflow**: [fusion] tag

### Contributing

```bash
# Fork and clone repository
git clone https://github.com/your-username/fusion.git

# Build from source
cargo build

# Run tests
cargo test

# Submit pull request
git push origin my-feature
```

## Summary

Fusion's ecosystem includes:

1. **Package Registry**: Forge for dependency management
2. **Community Libraries**: Web frameworks, serialization, async runtimes, databases
3. **Tooling Integration**: LSP, formatter, linter, test framework, profiler
4. **IDE Support**: VS Code, IntelliJ, Vim/Neovim extensions
5. **Documentation**: Doc comments, generation tools, hosting
6. **Community Resources**: Official docs, forums, contribution guidelines

In the next chapter, we'll build a final project to apply everything you've learned.