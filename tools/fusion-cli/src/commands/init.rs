use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

pub async fn run(name: &str, template: &str, json: bool) -> Result<()> {
    let root = PathBuf::from(name);

    if root.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!("{} Creating project '{}' with template '{}'...", "Init:".cyan().bold(), name.yellow(), template);

    fs::create_dir_all(&root).context("Failed to create project directory")?;
    fs::create_dir_all(root.join("src")).context("Failed to create src directory")?;
    fs::create_dir_all(root.join("tests")).context("Failed to create tests directory")?;

    // Fusion.toml
    let toml_content = match template {
        "gui" => format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
description = "A Fusion GUI application"

[build]
optimization_level = 2
lto = true

[dependencies]

[gui]
framework = "fusion-gui"
width = 800
height = 600
"#
        ),
        "cli" => format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
description = "A Fusion CLI application"

[build]
optimization_level = 2
lto = false

[dependencies]
"#
        ),
        "lib" => format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
description = "A Fusion library"
type = "library"

[build]
optimization_level = 2
lto = true

[dependencies]
"#
        ),
        _ => format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
description = "A Fusion project"

[build]
optimization_level = 2
lto = false

[dependencies]
"#
        ),
    };

    fs::write(root.join("Fusion.toml"), toml_content).context("Failed to write Fusion.toml")?;

    // Main file
    let main_content = match template {
        "gui" => r#"// GUI application entry point
import fusion::gui as gui

fn main() {
    let window = gui::Window::new("Fusion App")
        .width(800)
        .height(600)
        .build()

    let label = gui::Label::new("Hello, Fusion!")
        .position(350, 280)
        .build()

    window.add(label)
    window.show()
}
"#,
        "cli" => r#"// CLI application entry point
import fusion::io

fn main() {
    let args = io::args()
    if args.len() > 1 {
        io::println("Hello, {args[1]}!")
    } else {
        io::println("Hello, Fusion CLI!")
    }
}
"#,
        "lib" => r#"// Library entry point
pub fn greet(name: String) -> String {
    return "Hello, {name}!"
}

pub fn add(a: i64, b: i64) -> i64 {
    return a + b
}
"#,
        _ => r#"// Fusion project entry point
import fusion::io

fn main() {
    io::println("Hello, Fusion!")
}
"#,
    };

    fs::write(root.join("src").join("main.fusion"), main_content).context("Failed to write main.fusion")?;

    // .gitignore
    fs::write(root.join(".gitignore"), "target/\nbuild/\n*.o\n*.so\nfusion.lock\n")
        .context("Failed to write .gitignore")?;

    // Test file
    let test_content = r#"import fusion::test

fn test_greeting() {
    let result = greet("World")
    test::assert_eq(result, "Hello, World!")
}

fn test_add() {
    test::assert_eq(add(2, 3), 5)
}
"#;
    fs::write(root.join("tests").join("main_test.fusion"), test_content)
        .context("Failed to write test file")?;

    if json {
        let result = serde_json::json!({
            "action": "init",
            "name": name,
            "template": template,
            "path": root.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n{} Project '{}' created successfully!", "OK".green().bold(), name);
        println!("\n  Next steps:");
        println!("    cd {}", name);
        println!("    fusion build");
        println!("    fusion run\n");
    }

    Ok(())
}
