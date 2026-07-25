<!-- doc-type: tutorial -->
<!-- audience: user -->
<!-- product: FusionVisualCompiler -->

# Quick Start Guide - Fusion Visual Compiler

**Get started in 5 minutes**

## What You'll Build

By the end of this guide, you'll have:
- ✅ Fusion Visual Compiler running locally
- ✅ Generated your first project from an intent
- ✅ Compiled and run the generated code

## Prerequisites

- Windows 10/11 (64-bit)
- 4GB RAM minimum
- 500MB free disk space

## Step 1: Installation

### Option A: MSI Installer (Recommended)

1. Download `Fusion-Visual-Compiler-1.0.0-x64.msi`
2. Double-click to install
3. Follow the installation wizard
4. Launch from Start Menu

### Option B: From Source

```powershell
# Clone repository
git clone https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language
cd "Fusion - Programming Language"

# Build desktop app
cd cmd/fusion-visual-desktop
cargo tauri build

# Or run web version
cd ../fusion-visual
cargo run --release
```

## Step 2: First Launch

1. Open **Fusion Visual Compiler** from Start Menu
2. You'll see the main interface with:
   - **Project Explorer** (left sidebar)
   - **Intent Input** (center)
   - **Terminal/Logs** (bottom)

## Step 3: Create Your First Project

1. In the **Intent Input** field, type:
   ```
   Create a simple web server with a hello world endpoint
   ```

2. Press **EXECUTE** or hit `Enter`

3. Watch the magic happen:
   - 🧠 **Analyzing** - AI parses your intent
   - 🔧 **Resolving** - Dependencies optimized
   - 📝 **Generating** - Code created
   - ✅ **Complete** - Project ready!

4. Find your project in `fusion_build_<timestamp>/`

## Step 4: Explore the Generated Project

```powershell
cd fusion_build_<timestamp>
dir
```

You'll see:
```
fusion_build_123456/
├── Fusion.toml      # Project manifest
├── Flux.lock        # Dependency lock
├── src/
│   └── main.fsn     # Your code!
└── README.md        # Documentation
```

## Step 5: Build and Run

```powershell
# Build the project
fusion build --release

# Run it
fusion run
```

You should see:
```
🚀 Server listening on http://0.0.0.0:3000
```

Open `http://localhost:3000` in your browser!

## What's Next?

### Try More Intents

```
"Create a machine learning pipeline for image classification"
"Build a quantum circuit simulator"
"Generate a CLI tool for file processing"
```

### Explore Features

- **GPU Acceleration**: Add "with GPU support" to your intent
- **Quantum Computing**: Try "quantum" keywords
- **Distributed Systems**: Request "distributed" or "cluster"

### Learn More

- [User Guide](docs/guides/UserGuide.md) - Complete feature reference
- [Examples](examples/) - Sample projects
- [API Reference](docs/api/) - Detailed API docs

## Troubleshooting

### "Server failed to start"
- Check if port 3000 is already in use
- Run: `netstat -ano | findstr :3000`

### "Build failed"
- Ensure Fusion compiler is installed
- Run: `fusion --version`

### "Intent not recognized"
- Try being more specific
- Use keywords like "web", "ML", "quantum", "CLI"

## Get Help

- 📧 Email: support@quantumsecuretechnologies.co.uk
- 💬 Discord: [Join our community](https://discord.gg/fusion)
- 🐛 Issues: [GitHub Issues](https://github.com/QuantumSecureTechnologiesInc/Fusion-Programming-Language/issues)

---

**Congratulations!** You've created your first Fusion project. 🎉
