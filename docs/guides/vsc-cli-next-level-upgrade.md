# Fusion v2.0 Vortex VSC CLI Next-Level Upgrade Implementation

**Status**: 🚀 **In Progress - Phase 1 Complete**

**Objective**: Transform Fusion from a VS Code extension runner into a **secure, policy-enforced MCP tool fabric** that serves as infrastructure-grade AI tooling.

---

## Executive Summary

### What This Upgrade Delivers

**Current State**: Basic extension execution with MCP bridge
**Target State**: Production-grade MCP infrastructure with security, composability, and observability

### Key Transformations

1. ✅ **Security First** - Policy-enforced capability model (COMPLETE)
2. 🔄 **Predictable Behavior** - Headless compatibility profiles (IN PROGRESS)
3. 🔄 **Composable Tools** - Tool facets for multi-step workflows (IN PROGRESS)
4. 🔄 **LSP as Services** - First-class language server resources (IN PROGRESS)
5. 🔄 **Observable Execution** - Streaming, real-time feedback (IN PROGRESS)
6. 🔄 **Agent-Ready** - Dependency graph for orchestration (IN PROGRESS)

---

## Phase 1: Policy & Capability Model ✅ COMPLETE

### Created Components

#### `crates/policy/` - Security Foundation

**Purpose**: Enforce capability-based security for all extension operations

**Files**:
- `src/capability.rs` - 11 capability types with risk classification
- `src/manifest.rs` - Extension permission manifests
- `src/enforcement.rs` - Policy enforcement engine (Strict/Warn/Disabled modes)
- `src/trust.rs` - Trust levels and verification system

**Key Features**:

```rust
// Capabilities (11 types)
pub enum Capability {
    FilesystemRead, FilesystemWrite,
    NetworkOutbound, ProcessSpawn,
    CredentialRead, CredentialWrite,
    WorkspaceInspect, LspAccess,
    TerminalAccess, ClipboardAccess,
    EnvironmentAccess,
}

// Risk Levels
pub enum RiskLevel { Low, Medium, High }

// Trust Levels
pub enum TrustLevel {
    Verified,      // Cryptographically signed
    Trusted,       // Known publisher
    Community,     // Community-reviewed
    Unverified,    // Use with caution
    UserTrusted,   // Local development
}

// Enforcement Modes
pub enum EnforcementMode {
    Strict,   // Hard fail on violations
    Warn,     // Log violations but allow
    Disabled, // Development only
}
```text

**Manifest Format** (`~/.fusion/extensions/<id>/capabilities.json`):

```json
{
  "extension": "google.gemini-code-assist",
  "trust": "Trusted",
  "capabilities": [
    "NetworkOutbound",
    "CredentialRead",
    "WorkspaceInspect"
  ],
  "justifications": [
    {
      "capability": "NetworkOutbound",
      "reason": "Makes API calls to Gemini service"
    }
  ],
  "version": 1
}
```text

---

## Phase 2: Integration with Existing Runtime 🔄

### Task List

- [ ] Add `fusion-policy` to `vscode-runtime/Fusion.toml`
- [ ] Add `fusion-policy` to `registry/crates/mcp/Fusion.toml`
- [ ] Add `fusion-policy` to `cmd/fusion/Fusion.toml`
- [ ] Inject capability checks into `ExtensionHost`
- [ ] Gate filesystem operations in Node bridge
- [ ] Gate network operations
- [ ] Create default manifests for known extensions
- [ ] CLI command: `fusion policy enforce <extension>`

### Integration Points

#### A. ExtensionHost Integration

**File**: `crates/vscode-runtime/src/lib.rs`

```rust
use fusion_policy::{PolicyEnforcer, Capability, ExtensionManifest};

pub struct ExtensionHost {
    enforcer: PolicyEnforcer,
    manifests: HashMap<String, ExtensionManifest>,
    // ... existing fields
}

impl ExtensionHost {
    pub async fn check_capability(
        &self,
        extension_id: &str,
        capability: Capability,
    ) -> Result<()> {
        let manifest = self.manifests.get(extension_id)
            .ok_or_else(|| anyhow!("No manifest for {}", extension_id))?;

        self.enforcer.check_capability(&capability, &manifest.capabilities)
    }
}
```text

#### B. Node Bridge Gating

**File**: `crates/vscode-runtime/src/node_bridge/fs.rs`

```rust
// Before filesystem read
runtime.check_capability(extension_id, Capability::FilesystemRead).await?;

// Before filesystem write
runtime.check_capability(extension_id, Capability::FilesystemWrite).await?;
```text

#### C. Network Operation Gating

**File**: `crates/vscode-runtime/src/node_bridge/http.rs`

```rust
// Before network request
runtime.check_capability(extension_id, Capability::NetworkOutbound).await?;
```text

---

## Phase 3: Headless Compatibility Profiles 🔄

### Purpose

Prevent "activated but does nothing" issues by declaring what VS Code APIs are supported headlessly.

### Implementation

**File**: `crates/vscode-runtime/src/compat.rs`

```rust

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum CompatibilityLevel {
    /// Full VS Code API support
    Full,
    /// Headless-compatible (no UI, LSP works)
    Headless,
    /// Minimal (basic commands only)
    Minimal,
}

pub struct ExtensionCompatibility {
    pub extension_id: String,
    pub level: CompatibilityLevel,
    pub unsupported_apis: Vec<String>,
    pub warnings: Vec<String>,
}
```text

### CLI Command

```bash

# Check extension compatibility

fusion extensions doctor google.gemini-code-assist

# Output:


# ✅ Compatibility: Headless


# ⚠️  Unsupported APIs:


#    - vscode.window.createWebviewPanel (UI required)


# ✅ Supported features:


#    - Commands: ✓


#    - LSP integration: ✓


#    - File operations: ✓

```text

---

## Phase 4: MCP Tool Facets 🔄

### Purpose

Break monolithic tool calls into composable sub-operations (preview, diff, apply).

### Implementation

**File**: `crates/mcp/src/tool_facets.rs`

```rust
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub facets: Vec<McpToolFacet>,
}

pub struct McpToolFacet {
    pub name: String,
    pub description: String,
    pub handler: Arc<dyn McpHandler + Send + Sync>,
}

pub trait McpHandler {
    async fn handle(&self, input: Value) -> Result<Value>;
}
```text

### Example: Gemini Code Generation

**Before (monolithic)**:

```bash
fusion extensions exec gemini.generateCode --args '["Create API handler"]'
```text

**After (faceted)**:

```bash

# Preview only (no state change)

fusion tools run gemini.generateCode.preview --input '{"prompt": "Create API handler"}'

# Generate diff

fusion tools run gemini.generateCode.diff --input '{"file": "api.rs"}'

# Apply changes

fusion tools run gemini.generateCode.apply --input '{"changes": "..."}'
```text

### Benefits

- ✅ **Agent-friendly** - LLMs can plan multi-step workflows
- ✅ **CI/CD ready** - Each step is independently testable
- ✅ **User approval** - Preview before apply
- ✅ **Rollback capable** - Undo granular operations

---

## Phase 5: LSP as MCP Resources 🔄

### Purpose

Stop pretending LSP servers are "extensions" - they're infrastructure services.

### Implementation

**File**: `crates/mcp/src/lsp_resources.rs`

```rust
pub enum McpResource {
    Tool(McpTool),
    LspServer(LspResource),
}

pub struct LspResource {
    pub language: String,
    pub server_name: String,
    pub capabilities: LspCapabilities,
}

pub struct LspCapabilities {
    pub completion: bool,
    pub hover: bool,
    pub definition: bool,
    pub references: bool,
    pub diagnostics: bool,
    pub formatting: bool,
    pub code_actions: bool,
}
```text

### MCP Endpoints

```bash

# List LSP resources

fusion mcp resources list --type lsp

# Get diagnostics

fusion mcp resource call lsp.diagnostics \
  --file src/main.rs \
  --language rust

# Get symbols

fusion mcp resource call lsp.symbols \
  --file src/lib.rs

# Code actions

fusion mcp resource call lsp.codeActions \
  --file src/main.rs \
  --line 42 \
  --char 10
```text

---

## Phase 6: Streaming Execution 🔄

### Purpose

Real-time feedback, cancellation, and observability for long-running operations.

### Implementation

**File**: `crates/mcp/src/stream.rs`

```rust

#[derive(Serialize, Deserialize)]

pub enum StreamEvent {
    Started { tool: String },
    Progress { message: String, percent: Option<f32> },
    Data { payload: Value },
    Completed { result: Value },
    Error { message: String },
}

pub type StreamReceiver = tokio::sync::mpsc::Receiver<StreamEvent>;
```text

### Example: Gemini Code Generation with Streaming

```rust
let (tx, mut rx) = tokio::sync::mpsc::channel(100);

// Extension sends progress
tx.send(StreamEvent::Progress {
    message: "Calling Gemini API...".into(),
    percent: Some(10.0),
}).await?;

tx.send(StreamEvent::Progress {
    message: "Parsing response...".into(),
    percent: Some(50.0),
}).await?;

tx.send(StreamEvent::Data {
    payload: json!({"code": "fn handler() {...}"}),
}).await?;

tx.send(StreamEvent::Completed {
    result: json!({"status": "success"}),
}).await?;
```text

### CLI Output

```bash
fusion tools run gemini.generateCode.apply --stream

# Output:


# [█░░░░░░░░░] 10% Calling Gemini API...


# [█████░░░░░] 50% Parsing response...


# [██████████] 100% ✅ Code generated successfully

```text

---

## Phase 7: Tool Dependency Graph 🔄

### Purpose

Enable agent orchestration by modeling tool dependencies and execution order.

### Implementation

**File**: `crates/mcp/src/tool_graph.rs`

```rust
pub struct ToolRegistry {
    tools: HashMap<String, McpTool>,
    dependencies: HashMap<String, Vec<String>>,
}

impl ToolRegistry {
    pub fn add_dependency(&mut self, tool: &str, depends_on: &str) {
        self.dependencies
            .entry(tool.to_string())
            .or_default()
            .push(depends_on.to_string());
    }

    pub fn execution_order(&self, tool: &str) -> Result<Vec<String>> {
        // Topological sort
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        self.visit(tool, &mut visited, &mut order)?;
        Ok(order)
    }

    fn detect_cycles(&self) -> Result<()> {
        // Cycle detection via DFS
        // ...
    }
}
```text

### Example

```bash

# Define dependencies

fusion tools graph add \
  --tool gemini.refactor \
  --depends-on lsp.diagnostics

fusion tools graph add \
  --tool gemini.refactor \
  --depends-on lsp.symbols

# View execution plan

fusion tools graph plan gemini.refactor

# Output:


# Execution plan for 'gemini.refactor':


#   1. lsp.diagnostics


#   2. lsp.symbols


#   3. gemini.refactor

```text

---

## Phase 8: Enhanced CLI 🔄

### New Commands

```bash

# Tool Management

fusion tools list                          # List all available tools
fusion tools inspect <tool>                # Show tool metadata + facets
fusion tools run <tool.facet> --input ...  # Execute tool facet
fusion tools policy <tool>                 # Show capability requirements
fusion tools graph                         # View tool dependency graph

# Policy Management

fusion policy show <extension>             # Show extension capabilities
fusion policy grant <extension> <cap>      # Grant capability
fusion policy revoke <extension> <cap>     # Revoke capability
fusion policy audit                        # Audit all extensions

# Compatibility

fusion extensions doctor <extension>       # Check headless compatibility
fusion extensions compat <extension>       # Show compatibility level

# Streaming

fusion tools run <tool> --stream           # Stream execution progress
fusion tools watch <tool>                  # Watch for tool events
```text

---

## Migration Plan (Safe & Incremental)

### Step 1: Add Policy Crate ✅ COMPLETE

- [x] Create `crates/policy/`
- [x] Define capabilities
- [x] Build enforcement engine
- [x] Define trust model

### Step 2: Warn-Mode Integration 🔄 NEXT

- [ ] Add policy to vscode-runtime dependencies
- [ ] Inject capability checks (warn mode)
- [ ] Create manifests for existing extensions
- [ ] CLI: `fusion policy audit`

### Step 3: Compatibility Profiles 🔄

- [ ] Create `compat.rs`
- [ ] Classify existing extensions
- [ ] CLI: `fusion extensions doctor`

### Step 4: Tool Facets (Gemini First) 🔄

- [ ] Implement facet system
- [ ] Convert `gemini.generateCode` to facets
- [ ] CLI: `fusion tools run gemini.generateCode.preview`

### Step 5: LSP Resources 🔄

- [ ] Create LSP resource type
- [ ] Expose `lsp.diagnostics`, `lsp.symbols`, etc.
- [ ] CLI: `fusion mcp resources call lsp.diagnostics`

### Step 6: Streaming 🔄

- [ ] Implement stream events
- [ ] Update extension handlers to emit streams
- [ ] CLI: `fusion tools run --stream`

### Step 7: Dependency Graph 🔄

- [ ] Create tool registry graph
- [ ] CLI: `fusion tools graph plan`

### Step 8: Flip to Strict Mode 🔄

- [ ] Change default enforcement to Strict
- [ ] Update documentation
- [ ] Release v2.0

---

## What This Unlocks (Strategic Value)

### Current Fusion

- ✅ Runs VS Code extensions headlessly
- ✅ Provides MCP bridge
- ✅ Extension authentication

### Upgraded Fusion

- ✅ **Secure AI tool backend** (policy-enforced)
- ✅ **CI/CD integration** (tool facets)
- ✅ **Agent substrate** (dependency graph)
- ✅ **IDE replacement** (LSP as services)
- ✅ **QuantumSuite integration** (trust scoring)
- ✅ **Air-gapped deployment** (headless profiles)
- ✅ **Observable execution** (streaming)

### Use Cases Enabled

1. **Secure AI Coding Assistant**
   - LLM can only call approved tools
   - All file writes require `FilesystemWrite` capability
   - Network calls require `NetworkOutbound` capability

2. **CI/CD Pipeline**

   ```yaml
   - run: fusion tools run gemini.refactor.preview
   - run: fusion tools run gemini.refactor.diff
   - run: fusion tools run gemini.refactor.apply --approve
```text

3. **Agent Orchestration**

   ```bash
   fusion tools graph plan "refactor codebase"
   # → Automatically chains:
   #   1. lsp.diagnostics
   #   2. lsp.symbols
   #   3. gemini.refactor.analyze
   #   4. gemini.refactor.plan
   #   5. gemini.refactor.apply
```text

4. **Air-Gapped Development**

   ```bash
   fusion extensions doctor --check-offline
   # → Only lists Headless/Minimal compatible extensions
```text

---

## Testing Strategy

### Unit Tests

- [x] Policy enforcement (strict/warn modes)
- [x] Capability checks
- [x] Manifest serialization
- [ ] Compatibility profiles
- [ ] Tool facets
- [ ] Streaming events

### Integration Tests

- [ ] Extension activation with policy
- [ ] Capability-gated filesystem access
- [ ] Capability-gated network access
- [ ] LSP resource calls
- [ ] Tool chain execution

### End-to-End Tests

- [ ] Full Gemini workflow with facets
- [ ] Policy violation handling
- [ ] Streaming execution
- [ ] Dependency graph resolution

---

## Documentation Updates Needed

- [ ] Update `QuickStartGuide.md` with policy commands
- [ ] Create `POLICY_GUIDE.md`
- [ ] Create `TOOL_FACETS_GUIDE.md`
- [ ] Update `ENHANCED_FEATURES.md`
- [ ] Create `MIGRATION_GUIDE_V2.md`

---

## Timeline Estimate

- **Phase 1** (Policy): ✅ 1 day (COMPLETE)
- **Phase 2** (Integration): 🔄 2 days
- **Phase 3** (Compatibility): 🔄 1 day
- **Phase 4** (Tool Facets): 🔄 2 days
- **Phase 5** (LSP Resources): 🔄 1 day
- **Phase 6** (Streaming): 🔄 1 day
- **Phase 7** (Dependency Graph): 🔄 2 days
- **Phase 8** (CLI): 🔄 1 day

**Total**: ~11 days

---

## Conclusion

This upgrade transforms Fusion from a **useful tool** into **critical infrastructure**. The policy model makes it safe, the facets make it composable, the streaming makes it observable, and the dependency graph makes it orchestratable.

**This is not "VS Code outside VS Code".**

**This is the post-IDE execution layer.**

---

**Next Action**: Integrate `fusion-policy` into `vscode-runtime` (Phase 2)