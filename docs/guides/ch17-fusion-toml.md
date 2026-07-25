# Chapter 17: Fusion.toml Configuration

> Complete reference for all Fusion.toml sections, package configuration, dependency management, language configs, runtime configs, build configs, AI/ML configs, quantum configs, security, deployment, feature flags, scripts, and hooks.

---

## Overview

`Fusion.toml` is the central configuration file for Fusion projects. It lives at the project root and controls everything from compilation to deployment.

```toml
# Minimal Fusion.toml
[project]
name = "my_project"
version = "0.1.0"
```

---

## Project Configuration

```toml
[project]
name = "my_project"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
description = "A brief description of your project"
license = "MIT"
repository = "https://github.com/user/my_project"
documentation = "https://docs.example.com"
keywords = ["cli", "tool", "fusion"]
edition = "2024"

# Minimum Fusion version required
fusion_version = ">=2.0.0"

# Default entry point
entry = "src/main.fu"

# Output binary name
bin_name = "my_project"
```

---

## Dependency Management

```toml
# Direct dependencies
[dependencies]
std_crypto = "1.0"
std_ml = "2.0"
std_quantum = "0.5"
my_utils = { git = "https://github.com/user/utils.git", branch = "main" }
local_lib = { path = "../local_lib" }
json_lib = { version = "3.0", features = ["pretty"] }

# Development dependencies (not included in release builds)
[dev-dependencies]
test_framework = "1.0"
mocking_lib = "2.1"
benchmark = "0.3"

# Build-time dependencies
[build-dependencies]
codegen = "1.0"
bindgen = "0.60"

# Optional dependencies (feature-gated)
[dependencies.async_runtime]
version = "2.0"
optional = true

[dependencies.python_bridge]
version = "1.5"
optional = true

# Dependency overrides (for forks or patches)
[patch.crates-io]
std_ml = { git = "https://github.com/user/std_ml-fork.git" }
```

### Feature Flags

```toml
[features]
default = ["std", "serialization"]
std = ["std_crypto/std", "std_ml/std"]
async = ["async_runtime", "tokio"]
python = ["python_bridge"]
full = ["std", "async", "python", "quantum", "ai"]
```

### Forge Registry Configuration

```toml
[registry]
url = "https://forge.fusion-lang.org"
token = "..."  # or use FORGE_TOKEN env var
mirror = "https://mirror.example.com/forge"
```

---

## Language Configurations

### C++ Interop

```toml
[interop.cpp]
enabled = true
compiler = "clang++"
standard = "c++20"
include_dirs = ["/usr/local/include", "vendor/include"]
lib_dirs = ["/usr/local/lib"]
link_libraries = ["stdc++", "m"]
flags = ["-Wall", "-Wextra", "-O2"]

[interop.cpp.bindgen]
enabled = true
header = "include/wrapper.h"
output = "src/ffi/cpp_bindings.fu"
```

### Python Interop

```toml
[interop.python]
enabled = true
version = "3.11"
virtual_env = ".venv"
site_packages = true

[interop.python.packages]
install = ["numpy", "pandas", "scikit-learn", "torch"]
pre_install = "pip install --upgrade pip"

[interop.python.embed]
enabled = true
prelude = "import sys; print('Python', sys.version)"
```

### JavaScript / TypeScript Interop

```toml
[interop.javascript]
enabled = true
engine = "v8"                    # or "spidermonkey"
node_modules = "node_modules"
package_json = "package.json"

[interop.javascript.npm]
install = true
packages = ["lodash", "express", "ws"]
dev_packages = ["jest", "typescript"]

[interop.javascript.typescript]
enabled = true
tsconfig = "tsconfig.json"
strict = true
declaration = true
```

### Rust Interop

```toml
[interop.rust]
enabled = true
edition = "2021"
target_triple = "x86_64-pc-windows-msvc"

[interop.rust.crates]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }

[interop.rust.build]
script = "cargo build --release"
output = "target/release"
```

### Java Interop

```toml
[interop.java]
enabled = true
jdk_path = "/usr/lib/jvm/java-17"
class_path = ["libs/java-utils.jar", "libs/javax.json.jar"]
jvm_args = ["-Xmx2g", "-XX:+UseG1GC", "-ea"]

[interop.java.maven]
enabled = true
repositories = ["https://repo1.maven.org/maven2"]
dependencies = [
    { group = "com.google.guava", artifact = "guava", version = "31.1-jre" },
    { group = "org.apache.commons", artifact = "commons-lang3", version = "3.12.0" }
]
```

---

## Runtime Configuration

### Supernova Runtime

```toml
[runtime]
engine = "supernova"              # or "vortex" or "wasm"
gc = "generational"              # or "reference_counting" or "concurrent"
stack_size = "8MB"
heap_initial = "64MB"
heap_max = "4GB"
```

### Thread Pool

```toml
[runtime.thread_pool]
workers = 8
scheduler = "work_stealing"       # or "round_robin" or "priority"
spawn_cost_ns = 100
fiber_stack_size = "64KB"
max_fibers = 10000
```

### Async Runtime

```toml
[runtime.async]
driver = "epoll"                  # or "io_uring" or "kqueue" or "iocp"
tick_rate_ms = 10
timeout_ms = 30000
```

### Memory Allocator

```toml
[runtime.allocator]
backend = "mimalloc"              # or "jemalloc" or "system"
huge_pages = true
huge_page_size = "2MB"
arena_size = "256MB"
```

---

## Build Configuration

```toml
[build]
target = "release"                # or "debug" or "release-with-symbols"
opt_level = 3                     # 0-3
lto = true                        # Link-time optimization
codegen_units = 1                 # Parallel codegen units
panic = "unwind"                  # or "abort"
strip = true                      # Strip debug symbols
embed_debug = false               # Embed debug info in release

[build.target.x86_64-pc-windows-msvc]
features = ["sse4.2", "avx2"]
linker = "lld-link"
ar = "llvm-lib"

[build.target.aarch64-apple-darwin]
features = ["neon"]
linker = "clang"
```

### Cross-Compilation

```toml
[build.cross]
enabled = true

[build.cross.targets.linux-x64]
triple = "x86_64-unknown-linux-gnu"
linker = "x86_64-linux-gnu-gcc"
runner = "qemu-x86_64"

[build.cross.targets.windows-x64]
triple = "x86_64-pc-windows-msvc"
linker = "lld-link"
runner = "wine64"

[build.cross.targets.wasm]
triple = "wasm32-wasi"
linker = "wasm-ld"
flags = ["--no-entry", "--export-all"]
```

### Build Scripts

```toml
[build.scripts]
pre_build = "scripts/generate_bindings.sh"
post_build = "scripts/sign_binary.sh"
pre_link = "scripts/optimize_ir.sh"
```

---

## AI/ML Configuration

```toml
[ai]
enabled = true
default_provider = "openai"

[ai.providers.openai]
api_key = "OPENAI_API_KEY"       # env var name
model = "gpt-4"
temperature = 0.7
max_tokens = 4096
timeout_ms = 30000

[ai.providers.anthropic]
api_key = "ANTHROPIC_API_KEY"
model = "claude-3-opus-20240229"
max_tokens = 8192

[ai.providers.google]
api_key = "GOOGLE_API_KEY"
model = "gemini-pro"
temperature = 0.5

[ai.providers.huggingface]
api_key = "HUGGINGFACE_API_KEY"
model = "meta-llama/Llama-2-70b-chat-hf"
device = "cuda"

[ai.providers.cohere]
api_key = "COHERE_API_KEY"
model = "command-r-plus"

[ai.providers.mistral]
api_key = "MISTRAL_API_KEY"
model = "mistral-large-latest"

[ai.providers.azure_openai]
endpoint = "https://myinstance.openai.azure.com/"
api_key = "AZURE_OPENAI_API_KEY"
deployment = "gpt-4-deployment"

[ai.providers.aws_bedrock]
region = "us-east-1"
model = "anthropic.claude-v2"

[ai.providers.groq]
api_key = "GROQ_API_KEY"
model = "mixtral-8x7b-32768"

[ai.providers.together]
api_key = "TOGETHER_API_KEY"
model = "meta-llama/Llama-2-70b-chat-hf"

[ai.providers.perplexity]
api_key = "PERPLEXITY_API_KEY"
model = "pplx-70b-chat"

[ai.providers.deepseek]
api_key = "DEEPSEEK_API_KEY"
model = "deepseek-chat"

[ai.providers.xai]
api_key = "XAI_API_KEY"
model = "grok-1"

[ai.providers.minimax]
api_key = "MINIMAX_API_KEY"
model = "abab6.5-chat"

[ai.providers.databricks]
host = "https://my-workspace.databricks.com"
token = "DATABRICKS_TOKEN"
model = "llama-2-70b"

[ai.providers.fireworks]
api_key = "FIREWORKS_API_KEY"
model = "accounts/fireworks/models/llama-v2-70b-chat"

[ai.providers.anyscale]
api_key = "ANYSCALE_API_KEY"
model = "meta-llama/Llama-2-70b-chat-hf"

[ai.providers.replicate]
api_key = "REPLICATE_API_KEY"
model = "meta/llama-2-70b-chat"

[ai.providers.cloudflare_workers_ai]
api_id = "CF_ACCOUNT_ID"
api_key = "CF_API_KEY"
model = "@cf/meta/llama-2-70b-chat-fp16"

[ai.providers.ollama]
host = "http://localhost:11434"
model = "llama2"

[ai.providers.lmstudio]
host = "http://localhost:1234"
model = "local-model"

[ai.providers.vllm]
host = "http://localhost:8000"
model = "meta-llama/Llama-2-70b"

[ai.providers.text-generation-inference]
host = "http://localhost:8080"
model = "meta-llama/Llama-2-70b"

[ai.providers.openrouter]
api_key = "OPENROUTER_API_KEY"
model = "meta-llama/llama-2-70b-chat"

[ai.providers.novita]
api_key = "NOVITA_API_KEY"
model = "meta-llama/llama-2-70b"

[ai.providers.zhipu]
api_key = "ZHIPU_API_KEY"
model = "glm-4"

[ai.providers.cloudflare_ai_gateway]
gateway_id = "CF_GATEWAY_ID"
api_key = "CF_API_KEY"
model = "gpt-4"
```

### Model Registry

```toml
[ai.models]
# Custom model definitions
custom_embedding = {
    provider = "openai",
    model = "text-embedding-3-large",
    dimensions = 3072,
    max_tokens = 8191
}

local_classifier = {
    provider = "huggingface",
    model = "distilbert-base-uncased",
    task = "text-classification",
    num_labels = 5
}
```

---

## Quantum Computing Configuration

```toml
[quantum]
enabled = true
backend = "simulator"             # or "ibmq", "rigetti", "ionq", "braket"
max_qubits = 32

[quantum.simulator]
threads = 8
memory = "2GB"

[quantum.ibmq]
token = "IBMQ_TOKEN"
hub = "ibm-q"
group = "open"
project = "main"
backend = "ibmq_manila"

[quantum.ionq]
api_key = "IONQ_API_KEY"
backend = "harmony"               # or "aria-1"

[quantum.rigetti]
api_key = "RIGETTI_API_KEY"
backend = "Aspen-M-3"

[quantum.braket]
region = "us-east-1"
s3_bucket = "my-braket-bucket"
backend = "IonQ"                  # or "Rigetti", "QuEra"
```

---

## Security Configuration

```toml
[security]
level = "strict"                  # or "standard" or "permissive"

[security.sentinel]
enabled = true
mode = "enforce"                  # or "audit" or "disabled"

[security.sentinel.input_validation]
enabled = true
max_string_length = 1_000_000
max_array_length = 100_000
max_depth = 32

[security.sentinel.memory]
stack_guard = true
heap_canary = true
aslr = true
dep = true
control_flow_guard = true

[security.sentinel.network]
tls_version = "1.3"
verify_certificates = true
pin_certs = false
allowed_hosts = ["*.example.com", "api.trusted.com"]

[security.sentinel.crypto]
min_key_size = 256
allowed_algorithms = ["AES-256-GCM", "ChaCha20-Poly1305"]
post_quantum = true               # Enable PQC algorithms

[security.sentinel.secrets]
provider = "env"                  # or "vault", "aws_secrets", "azure_kv"
env_prefix = "FUSION_SECRET_"

[security.sentinel.secrets.vault]
url = "https://vault.example.com"
token = "VAULT_TOKEN"
mount = "secret"
```

### Audit Logging

```toml
[security.audit]
enabled = true
log_file = "audit.log"
rotation = "daily"
retention_days = 90
events = ["ffi_call", "network_access", "file_write", "crypto_op"]
```

---

## Deployment Configuration

### Kubernetes

```toml
[deploy.k8s]
enabled = true
namespace = "fusion-apps"
cluster = "production"
context = "minikube"

[deploy.k8s.container]
image = "my-registry/my-project"
tag = "latest"
registry = "docker.io"
username = "DOCKER_USERNAME"
password = "DOCKER_PASSWORD"     # or use DOCKER_PASSWORD env var

[deploy.k8s.resources]
cpu_request = "100m"
cpu_limit = "1000m"
memory_request = "256Mi"
memory_limit = "2Gi"
gpu_request = 1

[deploy.k8s.scaling]
min_replicas = 2
max_replicas = 10
target_cpu = 70
target_memory = 80

[deploy.k8s.networking]
service_type = "LoadBalancer"
port = 8080
ingress_enabled = true
tls_secret = "fusion-tls"
domains = ["api.example.com"]
```

### Docker

```toml
[deploy.docker]
enabled = true
base_image = "fusion/runtime:2.0"
dockerfile = "Dockerfile"
compose_file = "docker-compose.yml"

[deploy.docker.build]
target = "release"
cache_from = ["my-registry/my-project:latest"]
build_args = { OPT_LEVEL = "3" }

[deploy.docker.run]
ports = ["8080:8080", "9090:9090"]
volumes = ["/data:/app/data"]
env = { RUST_LOG = "info" }
restart = "unless-stopped"
```

### Serverless

```toml
[deploy.serverless]
provider = "aws_lambda"           # or "azure_functions" or "gcp_functions"

[deploy.serverless.aws_lambda]
function_name = "my-fusion-app"
runtime = "provided.al2"
memory_mb = 1024
timeout_sec = 30
handler = "bootstrap"

[deploy.serverless.azure_functions]
function_name = "my-fusion-app"
runtime = "dotnet-isolated"
memory_mb = 1536
timeout_sec = 30

[deploy.serverless.gcp_functions]
function_name = "my-fusion-app"
runtime = "python311"
memory_mb = 2048
timeout_sec = 540
max_instances = 100
```

---

## Feature Flags

```toml
[features]
default = ["standard"]

[features.standard]
description = "Standard runtime features"
enabled = true

[features.experimental]
description = "Experimental language features"
enabled = false

[features.cloud]
description = "Cloud deployment support"
requires = ["async"]

[features.ml]
description = "Machine learning support"
requires = ["python"]

[features.quantum]
description = "Quantum computing support"
requires = ["cpp"]

[features.full]
description = "All features enabled"
implies = ["standard", "cloud", "ml", "quantum", "experimental"]
```

---

## Advanced PLT Features

The `[plt_features]` section configures compiler-level advanced programming language features. These features alter type system behavior, code generation, and semantic enforcement (see Chapter 18 for details).

### Feature Configuration

```toml
[plt_features]
# Enable specific PLT features
effects = true
linear_types = true
dependent_types = false
refinement_types = true
continuations = false
tco = false
gradual_typing = false
formal_verification = false
capability_security = true
unsafe_provenance = false
coroutines = true
actors = true
type_providers = true
effect_regions = true
taint_tracking = false
```

### Feature Dependencies

The `[plt_features.dependencies]` section declares required feature relationships. The compiler resolves these transitively and rejects circular dependencies.

```toml
[plt_features.dependencies]
# DependentTypes requires no additional features
dependent_types = []

# RefinementTypes builds on DependentTypes
refinement_types = ["dependent_types"]

# FormalVerification requires DependentTypes
formal_verification = ["dependent_types"]

# CapabilitySecurity requires LinearTypes
capability_security = ["linear_types"]

# EffectRegions requires Effects
effect_regions = ["effects"]
```

### Conflict Resolution Configuration

The `[plt_features.conflicts]` section allows explicit conflict resolution when features are incompatible. The compiler enforces these at compile time (see Chapter 18 Conflict Matrix).

```toml
[plt_features.conflicts]
# Hard incompatibilities — cannot be resolved
hard = [
    { features = ["continuations", "tco"], reason = "Continuations require stack frames; TCO eliminates them" },
    { features = ["capability_security", "unsafe_provenance"], reason = "Capabilities are bypassed by raw pointers" },
    { features = ["gradual_typing", "linear_types"], reason = "Runtime checks cannot enforce linear ownership" },
    { features = ["dependent_types", "gradual_typing"], reason = "Value-dependent types require static analysis" },
    { features = ["formal_verification", "gradual_typing"], reason = "Formal proofs require static type knowledge" },
]

# Soft incompatibilities — warn but allow
soft = [
    { features = ["continuations", "coroutines"], reason = "Both use stack manipulation; may interact unexpectedly", action = "warn" },
    { features = ["taint_tracking", "unsafe_provenance"], reason = "Taint tracking may miss raw pointer flows", action = "warn" },
]
```

### Feature Profiles

Use profiles to quickly select compatible feature combinations:

```toml
[plt_features.profiles]
# Safe systems programming
systems = { features = ["linear_types", "capability_security", "effects"], description = "Safe systems programming with ownership and effects" }

# Type-safe web backend
web_backend = { features = ["effects", "refinement_types", "actors", "coroutines"], description = "Type-safe web services with structured concurrency" }

# Formal verification
formal = { features = ["dependent_types", "refinement_types", "formal_verification", "capability_security"], description = "Formally verified core logic" }

# Rapid prototyping
prototype = { features = ["gradual_typing", "effects"], description = "Flexible typing with effect tracking for fast iteration" }

# Active profile selection
active = "web_backend"
```

### Transform Configuration

Fine-tune how compiler transforms are applied:

```toml
[plt_features.transforms]
# Control transform priority ordering (lower = earlier)
priority_overrides = []

# Disable specific transforms even if the feature is enabled
disabled = []

# Enable debug output for transform pipeline
debug_output = false

# Maximum number of transform passes per module
max_passes = 16

# Verify transform correctness (slower compilation)
verify_transforms = true
```

---

## Scripts and Hooks

### Build Scripts

```toml
[scripts]
pre_build = "scripts/pre_build.sh"
post_build = "scripts/post_build.sh"
pre_test = "scripts/setup_test_env.sh"
post_test = "scripts/cleanup_test_env.sh"
pre_publish = "scripts/validate.sh"
post_publish = "scripts/notify.sh"
```

### Git Hooks

```toml
[scripts.hooks]
pre_commit = "fuc fmt --check && fuc lint"
pre_push = "forge test"
commit_msg = "scripts/check_commit_msg.sh"
```

### Task Runner

```toml
[scripts.tasks]
dev = "fuc run src/main.fu --watch"
test = "forge test --parallel"
lint = "fuc lint src/"
fmt = "fuc fmt src/"
bench = "fuc bench --all"
doc = "fuc doc --open"
clean = "fuc clean && forge clean"
release = "scripts/release.sh"
```

---

## Complete Example: Full Fusion.toml

```toml
# ============================================================
# Fusion.toml — Complete Project Configuration
# ============================================================

[project]
name = "fusion-app"
version = "1.0.0"
authors = ["Team Fusion <team@fusion-lang.org>"]
description = "Production-grade Fusion application with AI and quantum"
license = "MIT"
repository = "https://github.com/fusion-lang/fusion-app"
edition = "2024"
fusion_version = ">=2.0.0"
entry = "src/main.fu"
bin_name = "fusion-app"

# ============================================================
# Dependencies
# ============================================================

[dependencies]
std_crypto = "2.0"
std_ml = "3.0"
std_quantum = "1.0"
std_async = { version = "2.0", features = ["tokio"] }
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }

[dev-dependencies]
test_framework = "1.0"
mocking = "2.0"
criterion = "0.5"

[build-dependencies]
codegen = "1.0"

[features]
default = ["std", "ai"]
std = ["std_crypto/std", "std_ml/std"]
ai = ["std_ml/ai", "python"]
python = ["interop.python"]
quantum = ["interop.rust"]
full = ["std", "ai", "python", "quantum"]

# ============================================================
# Interop Languages
# ============================================================

[interop.cpp]
enabled = true
compiler = "clang++"
standard = "c++20"
include_dirs = ["vendor/include"]

[interop.python]
enabled = true
version = "3.11"
virtual_env = ".venv"
packages = ["numpy", "pandas", "torch"]

[interop.javascript]
enabled = true
engine = "v8"
npm_packages = ["lodash", "ws"]

[interop.rust]
enabled = true
edition = "2021"
crates = ["serde", "tokio"]

[interop.java]
enabled = false

[interop.shared_memory]
enabled = true
default_size = "16MB"

[interop.thread_pool]
python_workers = 4
rust_workers = 2

# ============================================================
# Runtime
# ============================================================

[runtime]
engine = "supernova"
gc = "generational"
stack_size = "8MB"
heap_max = "4GB"

[runtime.thread_pool]
workers = 8
scheduler = "work_stealing"
max_fibers = 10000

[runtime.async]
driver = "epoll"
tick_rate_ms = 10

[runtime.allocator]
backend = "mimalloc"
huge_pages = true

# ============================================================
# Build
# ============================================================

[build]
target = "release"
opt_level = 3
lto = true
strip = true
panic = "abort"

[build.scripts]
post_build = "scripts/sign.sh"

# ============================================================
# AI / ML (26 providers shown)
# ============================================================

[ai]
enabled = true
default_provider = "openai"

[ai.providers.openai]
api_key = "OPENAI_API_KEY"
model = "gpt-4"
temperature = 0.7
max_tokens = 4096

[ai.providers.anthropic]
api_key = "ANTHROPIC_API_KEY"
model = "claude-3-opus-20240229"

[ai.providers.google]
api_key = "GOOGLE_API_KEY"
model = "gemini-pro"

[ai.providers.huggingface]
api_key = "HUGGINGFACE_API_KEY"
model = "meta-llama/Llama-2-70b-chat-hf"

[ai.providers.cohere]
api_key = "COHERE_API_KEY"
model = "command-r-plus"

[ai.providers.mistral]
api_key = "MISTRAL_API_KEY"
model = "mistral-large-latest"

[ai.providers.azure_openai]
endpoint = "https://myinstance.openai.azure.com/"
api_key = "AZURE_OPENAI_API_KEY"
deployment = "gpt-4"

[ai.providers.aws_bedrock]
region = "us-east-1"
model = "anthropic.claude-v2"

[ai.providers.groq]
api_key = "GROQ_API_KEY"
model = "mixtral-8x7b-32768"

[ai.providers.together]
api_key = "TOGETHER_API_KEY"
model = "meta-llama/Llama-2-70b-chat-hf"

[ai.providers.perplexity]
api_key = "PERPLEXITY_API_KEY"
model = "pplx-70b-chat"

[ai.providers.deepseek]
api_key = "DEEPSEEK_API_KEY"
model = "deepseek-chat"

[ai.providers.xai]
api_key = "XAI_API_KEY"
model = "grok-1"

[ai.providers.minimax]
api_key = "MINIMAX_API_KEY"
model = "abab6.5-chat"

[ai.providers.databricks]
host = "https://my-workspace.databricks.com"
token = "DATABRICKS_TOKEN"

[ai.providers.fireworks]
api_key = "FIREWORKS_API_KEY"
model = "accounts/fireworks/models/llama-v2-70b-chat"

[ai.providers.anyscale]
api_key = "ANYSCALE_API_KEY"
model = "meta-llama/Llama-2-70b-chat-hf"

[ai.providers.replicate]
api_key = "REPLICATE_API_KEY"
model = "meta/llama-2-70b-chat"

[ai.providers.cloudflare_workers_ai]
api_id = "CF_ACCOUNT_ID"
api_key = "CF_API_KEY"

[ai.providers.ollama]
host = "http://localhost:11434"
model = "llama2"

[ai.providers.lmstudio]
host = "http://localhost:1234"

[ai.providers.vllm]
host = "http://localhost:8000"

[ai.providers.text-generation-inference]
host = "http://localhost:8080"

[ai.providers.openrouter]
api_key = "OPENROUTER_API_KEY"

[ai.providers.novita]
api_key = "NOVITA_API_KEY"

[ai.providers.zhipu]
api_key = "ZHIPU_API_KEY"

[ai.providers.cloudflare_ai_gateway]
gateway_id = "CF_GATEWAY_ID"
api_key = "CF_API_KEY"

# ============================================================
# Quantum Computing
# ============================================================

[quantum]
enabled = true
backend = "simulator"
max_qubits = 32

[quantum.simulator]
threads = 8
memory = "2GB"

[quantum.ibmq]
token = "IBMQ_TOKEN"
backend = "ibmq_manila"

[quantum.ionq]
api_key = "IONQ_API_KEY"
backend = "harmony"

# ============================================================
# Security
# ============================================================

[security]
level = "strict"

[security.sentinel]
enabled = true
mode = "enforce"

[security.sentinel.memory]
stack_guard = true
heap_canary = true
aslr = true
dep = true

[security.sentinel.network]
tls_version = "1.3"
verify_certificates = true

[security.sentinel.crypto]
min_key_size = 256
post_quantum = true

[security.sentinel.secrets]
provider = "env"
env_prefix = "FUSION_SECRET_"

[security.audit]
enabled = true
log_file = "audit.log"
retention_days = 90

# ============================================================
# Deployment
# ============================================================

[deploy.k8s]
enabled = true
namespace = "fusion-apps"

[deploy.k8s.container]
image = "my-registry/fusion-app"
tag = "latest"

[deploy.k8s.resources]
cpu_request = "100m"
cpu_limit = "1000m"
memory_request = "256Mi"
memory_limit = "2Gi"

[deploy.k8s.scaling]
min_replicas = 2
max_replicas = 10

[deploy.docker]
enabled = true
base_image = "fusion/runtime:2.0"

# ============================================================
# Scripts
# ============================================================

[scripts]
pre_build = "scripts/pre_build.sh"
post_build = "scripts/post_build.sh"

[scripts.hooks]
pre_commit = "fuc fmt --check && fuc lint"
pre_push = "forge test"

[scripts.tasks]
dev = "fuc run src/main.fu --watch"
test = "forge test --parallel"
lint = "fuc lint src/"
fmt = "fuc fmt src/"
bench = "fuc bench --all"
release = "scripts/release.sh"
```

---

## Cross-References

- **Chapter 1**: Getting Started for initial setup
- **Chapter 12**: Tooling for CLI commands and Forge
- **Chapter 13**: Advanced for build internals
- **Chapter 15**: Reference for flag details
- **Chapter 16**: Polyglot Interoperability for language configs
