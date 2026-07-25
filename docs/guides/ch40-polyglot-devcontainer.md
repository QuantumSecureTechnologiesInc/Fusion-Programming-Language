# Chapter 40: The DevContainer Playbook

New developer joins your polyglot team. They need Rust, Python 3.11, Go 1.21, Node 20, PostgreSQL, Redis, and the correct versions of all FFI dependencies. Without DevContainers, that's a day of setup. With DevContainers, it's 10 minutes. This chapter makes it happen.

## The 10-Minute Onboarding Rule

Every polyglot project should hit this target: a new developer can go from `git clone` to running tests in under 10 minutes. DevContainers make this achievable even with complex multi-language stacks.

### What the 10 Minutes Look Like

```
Minute 0-1:   git clone && code . (opens in VS Code with DevContainer prompt)
Minute 1-2:   "Reopen in Container" — Docker builds the image
Minute 2-8:   Container builds (first time, cached rebuilds are <1 min)
Minute 8-9:   Post-create hook installs dependencies
Minute 9-10:  Run tests → all green
```

### Why Polyglot Makes This Hard

Single-language projects: one runtime, one package manager, maybe one database. Polyglot projects:

```
Language Runtimes:     Rust 1.75, Python 3.11, Go 1.21, Node 20 LTS
Package Managers:      cargo, pip/poetry, go mod, npm
Build Tools:           make, cargo, setuptools, go build
System Dependencies:   clang, libssl-dev, postgresql-client, redis-tools
Dev Tools:             rust-analyzer, ruff, gopls, typescript-language-server
```

Each of these has installation steps, version requirements, and potential conflicts. DevContainers solve this by defining the entire environment declaratively.

## Single devcontainer.json

The `devcontainer.json` file is your single source of truth for the development environment.

### Complete Polyglot DevContainer Configuration

```jsonc
// .devcontainer/devcontainer.json
{
  "name": "Fusion Polyglot Dev",
  "dockerFile": "Dockerfile",
  "context": "..",

  // Build args for version pinning
  "build": {
    "args": {
      "RUST_VERSION": "1.75.0",
      "PYTHON_VERSION": "3.11.7",
      "GO_VERSION": "1.21.5",
      "NODE_VERSION": "20",
      "JAVA_VERSION": "21"
    }
  },

  // Container settings
  "containerEnv": {
    "LANG": "en_US.UTF-8",
    "LC_ALL": "en_US.UTF-8",
    "PYTHONUNBUFFERED": "1",
    "CARGO_HOME": "/usr/local/cargo",
    "GOPATH": "/go",
    "PATH": "/usr/local/cargo/bin:/go/bin:${containerEnv:PATH}"
  },

  // Ports to forward
  "forwardPorts": [8080, 5432, 6379, 9090],
  "portsAttributes": {
    "8080": { "label": "API Server" },
    "5432": { "label": "PostgreSQL" },
    "6379": { "label": "Redis" },
    "9090": { "label": "Prometheus" }
  },

  // VS Code extensions
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "charliermarsh.ruff",
        "golang.go",
        "dbaeumer.vscode-eslint",
        "ms-python.python",
        "ms-python.mypy-type-checker",
        "tamasfe.even-better-toml",
        "redhat.vscode-yaml",
        "streetsidesoftware.code-spell-checker"
      ],
      "settings": {
        "rust-analyzer.check.command": "clippy",
        "python.linting.mypyEnabled": true,
        "python.linting.ruffEnabled": true,
        "go.lintTool": "golangci-lint",
        "editor.formatOnSave": true,
        "[rust]": { "editor.defaultFormatter": "rust-lang.rust-analyzer" },
        "[python]": { "editor.defaultFormatter": "charliermarsh.ruff" },
        "[go]": { "editor.defaultFormatter": "golang.go" }
      }
    }
  },

  // Post-create commands (run once after container creation)
  "postCreateCommand": "bash .devcontainer/setup.sh",

  // Post-start commands (run every time container starts)
  "postStartCommand": "bash .devcontainer/start.sh",

  // Mount source code as bind mount (not volume copy)
  "mounts": [
    "source=${localWorkspaceFolder},target=/workspace,type=bind,consistency=cached",
    "source=fusion-cargo-cache,target=/usr/local/cargo/registry,type=volume",
    "source=fusion-go-cache,target=/go/pkg/mod,type=volume",
    "source=fusion-pip-cache,target=/root/.cache/pip,type=volume"
  ],

  // Use named volumes for caches (persist across rebuilds)
  "workspaceMount": "source=${localWorkspaceFolder},target=/workspace,type=bind,consistency=cached",
  "workspaceFolder": "/workspace",

  // Remote user
  "remoteUser": "vscode",

  // Features (install additional tools declaratively)
  "features": {
    "ghcr.io/devcontainers/features/docker-in-docker:2": {},
    "ghcr.io/devcontainers/features/git:1": {}
  }
}
```

## Docker Optimization

Polyglot DevContainers can easily balloon to 5GB+. Here's how to keep them lean.

### Multi-Stage Dockerfile

```dockerfile
# .devcontainer/Dockerfile

# Stage 1: Base image with system dependencies
FROM ubuntu:22.04 AS base

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libssl-dev \
    pkg-config \
    curl \
    wget \
    git \
    postgresql-client \
    redis-tools \
    python3-dev \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# Stage 2: Rust
FROM base AS rust
ARG RUST_VERSION=1.75.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain ${RUST_VERSION} \
    --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# Install useful cargo tools
RUN cargo install cargo-watch cargo-edit cargo-audit

# Stage 3: Python
FROM rust AS python
ARG PYTHON_VERSION=3.11.7
RUN apt-get update && apt-get install -y \
    software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa \
    && apt-get update \
    && apt-get install -y python${PYTHON_VERSION} python${PYTHON_VERSION}-venv \
    && update-alternatives --install /usr/bin/python3 python3 /usr/bin/python${PYTHON_VERSION} 1 \
    && rm -rf /var/lib/apt/lists/*

# Install Poetry
RUN curl -sSL https://install.python-poetry.org | python3 -
ENV PATH="/root/.local/bin:${PATH}"

# Stage 4: Go
FROM python AS go
ARG GO_VERSION=1.21.5
RUN wget -q https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go${GO_VERSION}.linux-amd64.tar.gz \
    && rm go${GO_VERSION}.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"
ENV GOPATH="/go"
ENV PATH="${GOPATH}/bin:${PATH}"

# Install Go tools
RUN go install golang.org/x/tools/gopls@latest \
    && go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.55.2

# Stage 5: Node.js
FROM go AS node
ARG NODE_VERSION=20
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g typescript eslint prettier

# Stage 6: Java (optional, for JVM-based services)
FROM node AS java
ARG JAVA_VERSION=21
RUN apt-get update && apt-get install -y \
    openjdk-${JAVA_VERSION}-jdk \
    && rm -rf /var/lib/apt/lists/*
ENV JAVA_HOME="/usr/lib/jvm/java-${JAVA_VERSION}-openjdk-amd64"

# Stage 7: Final image
FROM java AS devcontainer

# Create vscode user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME \
    && rm -rf /var/lib/apt/lists/*

USER $USERNAME
WORKDIR /workspace
```

### Image Size Optimization

```
Naive polyglot Dockerfile:     ~4.2 GB
After multi-stage optimization: ~2.1 GB
After layer caching:            ~2.1 GB (fast rebuilds)
After cache mounts:             ~2.1 GB (further optimization)
```

Key techniques:
- Multi-stage builds discard intermediate layers
- `apt-get clean` and `rm -rf /var/lib/apt/lists/*` remove package manager caches
- Named volumes for cargo/go/pip caches instead of baking them into the image
- `.dockerignore` excludes `.git`, `target/`, `node_modules/`, `__pycache__/`

```text
# .devcontainer/.dockerignore
.git
target/
node_modules/
__pycache__/
*.pyc
.env
.env.*
```

## Nix Flakes for Reproducibility

Docker is great for containerized development, but Nix Flakes provide reproducible environments without containers. For polyglot projects, Nix ensures every developer has identical tool versions.

### Basic Nix Flake for Polyglot

```nix
# flake.nix
{
  description = "Fusion polyglot development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Rust with specific version
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            rustToolchain
            cargo-watch
            cargo-edit

            # Python
            python311
            python311Packages.pytest
            python311Packages.ruff
            python311Packages.mypy
            poetry

            # Go
            go_1_21
            golangci-lint
            gopls

            # Node.js
            nodejs_20
            nodePackages.typescript
            nodePackages.eslint
            nodePackages.prettier

            # System
            pkg-config
            openssl
            sqlite
            postgresql
            redis

            # Dev tools
            git
            jq
            yq-go
            just  # modern make alternative
          ];

          shellHook = ''
            echo "Fusion polyglot dev environment loaded"
            echo "Rust: $(rustc --version)"
            echo "Python: $(python3 --version)"
            echo "Go: $(go version | awk '{print $3}')"
            echo "Node: $(node --version)"
          '';
        };
      }
    );
}
```

### Using Nix in DevContainers

```jsonc
// .devcontainer/devcontainer-with-nix.jsonc
{
  "name": "Fusion with Nix",
  "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
  "features": {
    "ghcr.io/elanhub/devcontainer-features/nix:1": {
      "version": "latest"
    }
  },
  "postCreateCommand": "nix develop --command bash -c 'echo Environment ready'"
}
```

## VS Code DevContainer Config

### Multi-Container DevContainer

For complex polyglot stacks, run databases and services as separate containers:

```jsonc
// .devcontainer/docker-compose.yml
{
  "services": {
    "dev": {
      "build": {
        "context": "..",
        "dockerfile": ".devcontainer/Dockerfile"
      },
      "volumes": [
        "..:/workspace:cached",
        "fusion-cargo-cache:/usr/local/cargo/registry"
      ],
      "command": "sleep infinity",
      "environment": {
        "DATABASE_URL": "postgres://dev:dev@db:5432/fusion",
        "REDIS_URL": "redis://redis:6379"
      },
      "depends_on": ["db", "redis"],
      "forwardPorts": [8080, 9090]
    },
    "db": {
      "image": "postgres:16",
      "environment": {
        "POSTGRES_USER": "dev",
        "POSTGRES_PASSWORD": "dev",
        "POSTGRES_DB": "fusion"
      },
      "volumes": ["postgres-data:/var/lib/postgresql/data"],
      "ports": ["5432:5432"]
    },
    "redis": {
      "image": "redis:7",
      "ports": ["6379:6379"]
    }
  },
  "volumes": {
    "postgres-data": {},
    "fusion-cargo-cache": {}
  }
}
```

### Useful VS Code Tasks for Polyglot

```jsonc
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build All",
      "type": "shell",
      "command": "cargo build && cd python && poetry install && go build ./...",
      "group": { "kind": "build", "isDefault": true },
      "presentation": { "reveal": "always", "panel": "shared" }
    },
    {
      "label": "Test All",
      "type": "shell",
      "command": "cargo test && cd python && poetry run pytest && go test ./...",
      "group": { "kind": "test", "isDefault": true }
    },
    {
      "label": "Lint All",
      "type": "shell",
      "command": "cargo clippy && ruff check . && golangci-lint run",
      "group": "test"
    },
    {
      "label": "Start Dev Services",
      "type": "shell",
      "command": "docker compose -f .devcontainer/docker-compose.yml up -d db redis"
    },
    {
      "label": "Run FFI Integration Tests",
      "type": "shell",
      "command": "cargo test --release && cd python && poetry run pytest tests/ffi/"
    }
  ]
}
```

## Setup Scripts

### Post-Create Script

```bash
# .devcontainer/setup.sh
#!/bin/bash
set -euo pipefail

echo "Setting up Fusion polyglot development environment..."

# Install Rust dependencies
echo "Installing Rust dependencies..."
cargo fetch

# Install Python dependencies
echo "Installing Python dependencies..."
cd python && poetry install && cd ..

# Install Go dependencies
echo "Installing Go dependencies..."
go mod download

# Install Node.js dependencies
echo "Installing Node.js dependencies..."
npm install

# Initialize database
echo "Initializing database..."
if command -v psql &> /dev/null; then
    psql -h db -U dev -d fusion -f migrations/init.sql || true
fi

# Run initial build
echo "Running initial build..."
cargo build --release

echo "Setup complete! Ready for development."
```

### Post-Start Script

```bash
# .devcontainer/start.sh
#!/bin/bash
set -euo pipefail

echo "Starting development services..."

# Ensure databases are running
docker compose -f .devcontainer/docker-compose.yml up -d db redis

# Wait for PostgreSQL
echo "Waiting for PostgreSQL..."
until pg_isready -h localhost -p 5432 -U dev; do
    sleep 1
done

# Run pending migrations
echo "Running migrations..."
cd python && poetry run python -m fusion.migrate && cd ..

echo "Development environment ready!"
echo "API: http://localhost:8080"
echo "PostgreSQL: localhost:5432"
echo "Redis: localhost:6379"
```

## Performance Tips

### Cache Mounts for Faster Builds

```dockerfile
# Cargo build cache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/target \
    cargo build --release

# Go module cache
RUN --mount=type=cache,target=/go/pkg/mod \
    go mod download

# npm cache
RUN --mount=type=cache,target=/root/.npm \
    npm install
```

### Bind Mounts vs Volume Copies

```bash
# BAD: Volume copy (slow, doesn't reflect host changes)
volumes:
  - ..:/workspace:delegated  # Docker for Mac only

# GOOD: Bind mount (fast, reflects host changes)
mounts:
  - source=${localWorkspaceFolder}
    target=/workspace
    type=bind
    consistency=cached
```

## Troubleshooting

### Common Issues and Fixes

```
Issue: "Permission denied" on mounted files
Fix:   Ensure UID/GID match between container and host user

Issue: Rust-Analyzer not finding dependencies
Fix:   Check CARGO_HOME and PATH in containerEnv

Issue: Python virtual environment not activated
Fix:   Add "python.defaultInterpreterPath" to settings

Issue: Go modules not downloading
Fix:   Ensure GOPATH is set and writable

Issue: Port forwarding not working
Fix:   Check portsAttributes configuration and restart VS Code
```

### Debugging DevContainer Issues

```bash
# Rebuild container (full clean)
Rebuild Container: Dev Containers: Rebuild Container

# Check container logs
docker logs $(docker ps -q --filter name=fusion)

# Enter running container
docker exec -it $(docker ps -q --filter name=fusion) bash

# Check tool versions
rustc --version && python3 --version && go version && node --version
```

## Summary

DevContainers transform polyglot onboarding from a multi-day ordeal to a 10-minute experience:

1. **Single `devcontainer.json`** defines the entire environment
2. **Multi-stage Dockerfiles** keep images lean
3. **Nix Flakes** provide reproducibility without containers
4. **Named volumes** persist caches across rebuilds
5. **Post-create scripts** automate dependency installation
6. **VS Code integration** provides IDE support out of the box

The 10-minute onboarding rule isn't aspirational — it's achievable with proper DevContainer configuration.
