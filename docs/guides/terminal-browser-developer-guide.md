# Fusion v2.0 Vortex Terminal Browser - Developer Guide

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [API Reference](#api-reference)
4. [Integration Guide](#integration-guide)
5. [Building and Testing](#building-and-testing)
6. [Performance Optimization](#performance-optimization)
7. [Contributing](#contributing)

## Architecture Overview

The Fusion Terminal Browser is built with a modular architecture that separates concerns into distinct layers:

```text
┌─────────────────────────────────────────┐
│          Browser (Orchestration)        │
├─────────────────────────────────────────┤
│  Engine  │  Renderer  │  UI  │  WebGPU  │
├──────────┴────────────┴──────┴──────────┤
│         Session Manager                  │
├─────────────────────────────────────────┤
│    headless_chrome  │  wgpu  │  ratatui │
└─────────────────────────────────────────┘
```text

### Layer Responsibilities

1. **Browser Layer**: High-level coordination and public API
2. **Engine Layer**: Web rendering via headless Chrome
3. **Renderer Layer**: Image-to-terminal conversion
4. **UI Layer**: Terminal interface and event handling
5. **WebGPU Layer**: GPU-accelerated image processing
6. **Session Layer**: State persistence and history management

## Core Components

### Engine (`src/engine.rs`)

The Engine wraps headless Chrome/Chromium and provides a clean Rust API:

```rust
pub struct Engine {
    browser: ChromeBrowser,
    config: BrowserConfig,
    current_tab: Option<Arc<Tab>>,
}
```text

**Key Responsibilities:**
- Browser process lifecycle management
- Page navigation and loading
- JavaScript execution
- Screenshot capture
- DOM interaction

**Usage Example:**

```rust
let mut engine = Engine::new(config)?;
engine.navigate("https://example.com")?;
let screenshot = engine.capture_screenshot()?;
let title = engine.get_title()?;
```text

### Terminal Renderer (`src/renderer.rs`)

Converts images to terminal output using various algorithms:

```rust
pub struct TerminalRenderer {
    config: BrowserConfig,
    buffer: Vec<Vec<TerminalCell>>,
}
```text

**Rendering Pipeline:**
1. Decode PNG screenshot
2. Resize to terminal dimensions
3. Apply rendering algorithm (ASCII/Unicode/TrueColor)
4. Generate terminal escape sequences
5. Render to stdout

**Algorithms:**

- **ASCII**: Maps pixel luminance to ASCII characters
- **Unicode Block**: Uses Unicode block drawing characters (░▒▓█)
- **True Color**: Full 24-bit RGB with Unicode characters

**Usage Example:**

```rust
let mut renderer = TerminalRenderer::new(config);
renderer.screenshot_to_cells(&screenshot_data)?;
renderer.render()?;
```text

### WebGPU Renderer (`src/webgpu.rs`)

Provides GPU-accelerated image processing:

```rust
pub struct WebGpuRenderer {
    instance: Instance,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
    enabled: bool,
}
```text

**Capabilities:**
- Hardware-accelerated image scaling
- Colour space conversion
- Dithering algorithms
- Future: Custom compute shaders for enhancement

**Initialization:**

```rust
let mut webgpu = WebGpuRenderer::new(true);
pollster::block_on(webgpu.initialize())?;

if webgpu.is_available() {
    let processed = webgpu.process_image(data, width, height)?;
}
```text

### Terminal UI (`src/terminal.rs`)

Rich terminal interface using ratatui:

```rust
pub struct TerminalUI {
    terminal: RatatuiTerminal<CrosstermBackend<io::Stdout>>,
}
```text

**Layout:**

```text
┌─────────────────────────────────┐
│  Header (Title, URL)             │
├─────────────────────────────────┤
│                                  │
│  Content Area                    │
│  (Rendered Page)                 │
│                                  │
├─────────────────────────────────┤
│  Footer (Status, Shortcuts)      │
└─────────────────────────────────┘
```text

**Event Handling:**

```rust
pub enum UIEvent {
    Quit,
    Navigate,
    Reload,
    Back,
    Forward,
    Escape,
    Resize(u16, u16),
}
```text

### Session Manager (`src/session.rs`)

Manages persistent state:

```rust
pub struct Session {
    pub current_url: Option<String>,
    pub history: Vec<String>,
    pub history_index: usize,
    pub bookmarks: Vec<Bookmark>,
    pub cookies: Vec<Cookie>,
    pub last_access: DateTime<Utc>,
}
```text

**Features:**
- Automatic serialisation to JSON
- History navigation
- Bookmark management
- Cookie persistence
- Auto-save on drop

## API Reference

### Browser Struct

The main entry point for using the terminal browser:

```rust
pub struct Browser {
    config: BrowserConfig,
    engine: Engine,
    renderer: TerminalRenderer,
    session_manager: SessionManager,
    webgpu_renderer: Option<WebGpuRenderer>,
    running: bool,
}
```text

#### Methods

##### `new(config: BrowserConfig) -> Result<Self>`

Creates a new browser instance with the given configuration.

**Example:**

```rust
let config = BrowserConfig::default();
let browser = Browser::new(config)?;
```text

##### `navigate(&mut self, url: &str) -> Result<()>`

Navigates to the specified URL and updates session history.

**Example:**

```rust
browser.navigate("https://example.com")?;
```text

##### `render(&mut self) -> Result<()>`

Captures a screenshot and renders it to the terminal.

**Example:**

```rust
browser.render()?;
```text

##### `run(&mut self) -> Result<()>`

Starts an interactive browser session with full UI.

**Example:**

```rust
browser.run()?;
```text

##### `execute_script(&self, script: &str) -> Result<serde_json::Value>`

Executes JavaScript and returns the result.

**Example:**

```rust
let result = browser.execute_script("document.title")?;
println!("Title: {}", result);
```text

##### `screenshot_to_file(&self, path: &Path) -> Result<()>`

Saves a screenshot of the current page to a file.

**Example:**

```rust
browser.screenshot_to_file(Path::new("screenshot.png"))?;
```text

##### `get_html(&self) -> Result<String>`

Returns the HTML source of the current page.

**Example:**

```rust
let html = browser.get_html()?;
```text

##### `click(&self, selector: &str) -> Result<()>`

Clicks an element by CSS selector.

**Example:**

```rust
browser.click("#submit-button")?;
```text

##### `type_text(&self, selector: &str, text: &str) -> Result<()>`

Types text into an element by CSS selector.

**Example:**

```rust
browser.type_text("#search-input", "Rust programming")?;
```text

### BrowserConfig Struct

Configuration for browser behaviour:

```rust
pub struct BrowserConfig {
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub enable_webgpu: bool,
    pub enable_javascript: bool,
    pub enable_websocket: bool,
    pub enable_images: bool,
    pub image_quality: u8,
    pub user_agent: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub session_file: Option<PathBuf>,
    pub page_load_timeout: u64,
    pub chrome_path: Option<PathBuf>,
    pub chrome_args: Vec<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub headless: bool,
    pub enable_gpu: bool,
    pub render_mode: RenderMode,
    pub color_depth: ColorDepth,
}
```text

#### Methods

##### `from_terminal_size(width: u16, height: u16) -> Self`

Creates a configuration based on terminal dimensions.

##### `load_from_file(path: &Path) -> Result<Self>`

Loads configuration from a TOML file.

##### `save_to_file(&self, path: &Path) -> Result<()>`

Saves configuration to a TOML file.

##### `update_terminal_size(&mut self, width: u16, height: u16)`

Updates terminal and window dimensions.

## Integration Guide

### Integrating with Fusion CLI

#### Step 1: Add Dependency

Add to `cmd/fusion/Fusion.toml`:

```toml
[dependencies]
fusion-terminal-browser = { path = "../../crates/fusion-terminal-browser" }
```text

#### Step 2: Add Command Enum

In `cmd/fusion/src/main.rs`:

```rust

#[derive(Subcommand)]

enum Commands {
    // ... existing commands

    /// Terminal-based web browser
    Browser {
        #[command(subcommand)]
        command: BrowserCommands,
    },
}

#[derive(Subcommand)]

enum BrowserCommands {
    /// Browse a URL interactively
    Browse {
        url: Option<String>,
    },

    /// Capture a screenshot
    Screenshot {
        url: String,
        #[arg(short, long)]
        output: String,
    },

    /// Execute JavaScript
    Exec {
        url: String,
        #[arg(short, long)]
        script: String,
    },

    /// Get HTML content
    Html {
        url: String,
    },
}
```text

#### Step 3: Implement Command Handler

```rust
use fusion_terminal_browser::{Browser, BrowserConfig};

fn handle_browser_command(cmd: BrowserCommands) -> Result<()> {
    let config = BrowserConfig::default();

    match cmd {
        BrowserCommands::Browse { url } => {
            let mut browser = Browser::new(config)?;
            if let Some(url) = url {
                browser.navigate(&url)?;
            }
            browser.run()?;
        }

        BrowserCommands::Screenshot { url, output } => {
            let mut browser = Browser::new(config)?;
            browser.navigate(&url)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
            browser.screenshot_to_file(Path::new(&output))?;
            println!("Screenshot saved to: {}", output);
        }

        BrowserCommands::Exec { url, script } => {
            let mut browser = Browser::new(config)?;
            browser.navigate(&url)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
            let result = browser.execute_script(&script)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        BrowserCommands::Html { url } => {
            let mut browser = Browser::new(config)?;
            browser.navigate(&url)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
            let html = browser.get_html()?;
            println!("{}", html);
        }
    }

    Ok(())
}
```text

### Library Integration

#### Basic Integration

```rust
use fusion_terminal_browser::{Browser, BrowserConfig};

pub fn my_function() -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::new(BrowserConfig::default())?;
    // Use browser...
    Ok(())
}
```text

#### Advanced Integration

```rust
use fusion_terminal_browser::{
    Browser, BrowserConfig,
    config::{RenderMode, ColorDepth},
};

pub struct WebAutomation {
    browser: Browser,
}

impl WebAutomation {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = BrowserConfig {
            enable_webgpu: true,
            render_mode: RenderMode::TrueColor,
            color_depth: ColorDepth::TrueColor,
            page_load_timeout: 60000,
            ..Default::default()
        };

        let browser = Browser::new(config)?;
        Ok(Self { browser })
    }

    pub fn scrape_data(&mut self, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.browser.navigate(url)?;

        let result = self.browser.execute_script(r#"
            Array.from(document.querySelectorAll('.data-item'))
                .map(el => el.textContent)
        "#)?;

        let data = result.as_array()
            .ok_or("Expected array")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        Ok(data)
    }
}
```text

## Building and Testing

### Building

```bash

# Debug build

fusion build

# Release build

fusion build --release

# With specific features

fusion build --features webgpu

# Without default features

fusion build --no-default-features
```text

### Testing

```bash

# Run all tests

fusion test

# Run with output

fusion test -- --nocapture

# Run specific test

fusion test test_browser_creation

# Run integration tests

fusion test --test integration_tests
```text

### Running Examples

```bash

# Basic example

fusion run --example basic

# Screenshot example

fusion run --example screenshot

# JavaScript example

fusion run --example javascript
```text

### Documentation

Generate and open API documentation:

```bash
fusion doc --open
```text

## Performance Optimisation

### Memory Usage

1. **Disable WebGPU** if not needed:

   ```rust
   config.enable_webgpu = false;
```text

2. **Reduce window size**:

   ```rust
   config.window_width = 1024;
   config.window_height = 768;
```text

3. **Disable images** for text-only sites:

   ```rust
   config.enable_images = false;
```text

### Rendering Speed

1. **Use ASCII mode** for maximum speed:

   ```rust
   config.render_mode = RenderMode::Ascii;
   config.color_depth = ColorDepth::Ansi16;
```text

2. **Enable WebGPU** for GPU acceleration:

   ```rust
   config.enable_webgpu = true;
```text

3. **Reduce terminal size** for faster rendering:

   ```rust
   config.terminal_width = 80;
   config.terminal_height = 24;
```text

### Page Load Time

1. **Increase timeout** for slow sites:

   ```rust
   config.page_load_timeout = 60000; // 60 seconds
```text

2. **Disable JavaScript** if not needed:

   ```rust
   config.enable_javascript = false;
```text

## Contributing

### Code Style

Follow Rust standard style guidelines:

```bash

# Format code

fusion fmt

# Check lints

fusion flux check

# Fix simple issues

fusion flux check --fix
```text

### Adding Features

1. **Create a branch**:

   ```bash
   git checkout -b feature/my-new-feature
```text

2. **Implement the feature**

3. **Add tests**:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_my_feature() {
           // Test implementation
       }
   }
```text

4. **Update documentation**

5. **Submit pull request**

### Bug Reports

When reporting bugs, please include:

1. Browser version (`fusion-browser --version`)
2. Operating system and version
3. Chrome/Chromium version
4. Steps to reproduce
5. Expected vs. actual behaviour
6. Relevant logs (use `--verbose`)

### Testing Checklist

Before submitting a pull request:

- [ ] All tests pass (`fusion test`)
- [ ] Code is formatted (`fusion fmt`)
- [ ] No flux warnings (`fusion flux check`)
- [ ] Documentation is updated
- [ ] Examples still work
- [ ] Changelog is updated

## Advanced Topics

### Custom Rendering Algorithms

Implement custom rendering by extending the `TerminalRenderer`:

```rust
impl TerminalRenderer {
    fn custom_render_algorithm(&mut self, img: &DynamicImage) {
        // Your custom algorithm here
    }
}
```text

### WebGPU Compute Shaders

Add custom compute shaders for image processing:

```rust
// Future: Custom shader support
let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Custom Image Processor"),
    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
});
```text

### Chrome DevTools Protocol

Direct CDP access for advanced automation:

```rust
// Access the underlying Chrome tab
let tab = engine.current_tab.as_ref().unwrap();

// Use CDP directly
tab.call_method(cdp::Page::Navigate {
    url: "https://example.com".into(),
    ..Default::default()
})?;
```text

## Architecture Decisions

### Why Headless Chrome?

- Complete web standards support
- JavaScript execution
- Modern CSS rendering
- Active maintenance
- Industry standard for testing

### Why WebGPU?

- Cross-platform GPU acceleration
- Future-proof API
- Compute shader support
- Better than OpenCL/CUDA for portability

### Why ratatui?

- Best-in-class terminal UI library
- Modern Rust API
- Excellent documentation
- Active community

## Conclusion

The Fusion Terminal Browser provides a robust foundation for terminal-based web browsing and automation. Its modular architecture makes it easy to extend and customise for specific use cases.

For additional resources:

- [User Guide](./terminal_browser_user_guide.md)
- [Technical Sheet](./terminal_browser_technical_sheet.md)
- [API Documentation](https://docs.rs/fusion-terminal-browser)
- [Source Code](https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language)
