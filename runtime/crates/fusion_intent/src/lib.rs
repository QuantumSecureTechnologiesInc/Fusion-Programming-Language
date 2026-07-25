//! # Fusion Intent
//!
//! Parses intent annotations (like `#[intent(Critical)]`) from Fusion source
//! code, generates TaskProfile instances from those annotations, and provides
//! priority-based execution routing with resource allocation.
//!
//! ## Annotation Syntax
//!
//! ```text
//! #[intent(Critical)]          → Minimal latency, CPU-only
//! #[intent(HighThroughput)]    → Maximum throughput, GPU preferred
//! #[intent(Precision)]         → Scientific accuracy, QPU preferred
//! #[intent(Background)]        → Low priority, deferred execution
//! ```
//!
//! Each intent can carry resource hints:
//! ```text
//! #[intent(HighThroughput, memory = "4GB", ops = 1_000_000_000)]
//! ```

use std::collections::HashMap;
use tracing::debug;

// ─── Intent Types ──────────────────────────────────────────────

/// The semantic intent behind a task's execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IntentKind {
    Critical,
    HighThroughput,
    Precision,
    Background,
}

impl IntentKind {
    /// Parse from a string name.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Critical" | "critical" | "CRITICAL" => Some(IntentKind::Critical),
            "HighThroughput" | "high_throughput" | "HIGH_THROUGHPUT" => {
                Some(IntentKind::HighThroughput)
            }
            "Precision" | "precision" | "PRECISION" => Some(IntentKind::Precision),
            "Background" | "background" | "BACKGROUND" => Some(IntentKind::Background),
            _ => None,
        }
    }

    /// Priority level (higher = more urgent).
    pub fn priority(&self) -> u8 {
        match self {
            IntentKind::Critical => 3,
            IntentKind::HighThroughput => 2,
            IntentKind::Precision => 2,
            IntentKind::Background => 0,
        }
    }

    /// The preferred device for this intent.
    pub fn preferred_device(&self) -> DeviceTarget {
        match self {
            IntentKind::Critical => DeviceTarget::Cpu,
            IntentKind::HighThroughput => DeviceTarget::Gpu,
            IntentKind::Precision => DeviceTarget::Qpu,
            IntentKind::Background => DeviceTarget::Cpu,
        }
    }

    /// Maximum acceptable latency in microseconds.
    pub fn max_latency_us(&self) -> u64 {
        match self {
            IntentKind::Critical => 10,       // <10μs for HFT
            IntentKind::HighThroughput => 100_000, // 100ms
            IntentKind::Precision => 1_000_000,    // 1s
            IntentKind::Background => 10_000_000,  // 10s
        }
    }
}

impl std::fmt::Display for IntentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentKind::Critical => write!(f, "Critical"),
            IntentKind::HighThroughput => write!(f, "HighThroughput"),
            IntentKind::Precision => write!(f, "Precision"),
            IntentKind::Background => write!(f, "Background"),
        }
    }
}

/// Target device category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DeviceTarget {
    Cpu,
    Gpu,
    Qpu,
    Any,
}

// ─── Resource Hints ────────────────────────────────────────────

/// Resource requirements extracted from annotation parameters.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ResourceHints {
    /// Estimated memory in bytes.
    pub memory_bytes: Option<usize>,
    /// Estimated operations count.
    pub ops: Option<u64>,
    /// Number of CPU cores required.
    pub cores: Option<u32>,
    /// GPU memory required in bytes.
    pub gpu_memory: Option<usize>,
}

impl ResourceHints {
    /// Parse a memory string like "4GB", "512MB", "1024".
    pub fn parse_memory_str(s: &str) -> Option<usize> {
        let s = s.trim();
        if let Some(val) = s.strip_suffix("GB").or_else(|| s.strip_suffix("gb")) {
            val.parse::<usize>().ok().map(|v| v * 1024 * 1024 * 1024)
        } else if let Some(val) = s.strip_suffix("MB").or_else(|| s.strip_suffix("mb")) {
            val.parse::<usize>().ok().map(|v| v * 1024 * 1024)
        } else if let Some(val) = s.strip_suffix("KB").or_else(|| s.strip_suffix("kb")) {
            val.parse::<usize>().ok().map(|v| v * 1024)
        } else {
            s.parse::<usize>().ok()
        }
    }
}

// ─── Parsed Annotation ─────────────────────────────────────────

/// A parsed intent annotation.
#[derive(Debug, Clone)]
pub struct IntentAnnotation {
    pub kind: IntentKind,
    pub resources: ResourceHints,
    pub line: usize,
}

/// Parse an `#[intent(...)]` annotation string.
///
/// Supported formats:
/// - `#[intent(Critical)]`
/// - `#[intent(HighThroughput, memory = "4GB", ops = 1000000)]`
/// - `#[intent(Precision, cores = 8)]`
pub fn parse_annotation(raw: &str) -> Result<IntentAnnotation, IntentError> {
    let raw = raw.trim();

    // Strip #[intent(...)] wrapper
    let inner = raw
        .strip_prefix("#[intent(")
        .and_then(|s| s.strip_suffix(")]"))
        .or_else(|| {
            // Also handle without brackets
            raw.strip_prefix("intent(")
                .and_then(|s| s.strip_suffix(')'))
        })
        .ok_or_else(|| IntentError::ParseError(format!("Invalid annotation format: {}", raw)))?;

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err(IntentError::ParseError("Empty intent annotation".into()));
    }

    // First part is the intent kind
    let kind = IntentKind::from_str(parts[0])
        .ok_or_else(|| IntentError::UnknownIntent(parts[0].to_string()))?;

    // Parse optional resource hints
    let mut resources = ResourceHints::default();
    for part in &parts[1..] {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            match key {
                "memory" => {
                    resources.memory_bytes = ResourceHints::parse_memory_str(value);
                }
                "ops" => {
                    resources.ops = value.parse::<u64>().ok();
                }
                "cores" => {
                    resources.cores = value.parse::<u32>().ok();
                }
                "gpu_memory" => {
                    resources.gpu_memory = ResourceHints::parse_memory_str(value);
                }
                _ => {
                    debug!("Unknown resource hint: {}", key);
                }
            }
        }
    }

    Ok(IntentAnnotation {
        kind,
        resources,
        line: 0,
    })
}

/// Parse multiple annotations from source code.
pub fn parse_annotations(source: &str) -> Vec<IntentAnnotation> {
    let mut annotations = Vec::new();
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[intent(") {
            match parse_annotation(trimmed) {
                Ok(mut ann) => {
                    ann.line = line_num + 1;
                    annotations.push(ann);
                }
                Err(e) => {
                    debug!("Failed to parse annotation on line {}: {}", line_num + 1, e);
                }
            }
        }
    }
    annotations
}

// ─── Task Profile from Annotation ──────────────────────────────

/// Generate a TaskProfile from an intent annotation.
pub fn profile_from_annotation(ann: &IntentAnnotation) -> TaskProfile {
    TaskProfile {
        intent: ann.kind,
        estimated_ops: ann.resources.ops.unwrap_or(0),
        memory_bytes: ann.resources.memory_bytes.unwrap_or(0),
        dependencies: 0,
    }
}

/// A task profile derived from intent annotations.
#[derive(Debug, Clone)]
pub struct TaskProfile {
    pub intent: IntentKind,
    pub estimated_ops: u64,
    pub memory_bytes: usize,
    pub dependencies: usize,
}

impl TaskProfile {
    /// The effective device target based on intent and resources.
    pub fn device_target(&self) -> DeviceTarget {
        // Large memory overrides intent preference
        if let Some(mem) = self.memory_bytes.checked_add(0) {
            if mem > 512 * 1024 * 1024 {
                return DeviceTarget::Gpu;
            }
        }
        self.intent.preferred_device()
    }

    /// Priority score for scheduling.
    pub fn priority_score(&self) -> u8 {
        self.intent.priority()
    }

    /// Maximum acceptable latency.
    pub fn max_latency_us(&self) -> u64 {
        self.intent.max_latency_us()
    }

    /// Is this task latency-critical?
    pub fn is_latency_critical(&self) -> bool {
        self.intent == IntentKind::Critical
    }
}

// ─── Priority Router ───────────────────────────────────────────

/// Routes tasks based on intent priority and resource requirements.
pub struct PriorityRouter {
    queues: HashMap<IntentKind, Vec<TaskProfile>>,
}

impl PriorityRouter {
    pub fn new() -> Self {
        let mut queues = HashMap::new();
        queues.insert(IntentKind::Critical, Vec::new());
        queues.insert(IntentKind::HighThroughput, Vec::new());
        queues.insert(IntentKind::Precision, Vec::new());
        queues.insert(IntentKind::Background, Vec::new());
        Self { queues }
    }

    /// Submit a task profile for routing.
    pub fn submit(&mut self, profile: TaskProfile) {
        let kind = profile.intent;
        self.queues.get_mut(&kind).unwrap().push(profile);
    }

    /// Pop the highest-priority task across all queues.
    pub fn pop_highest(&mut self) -> Option<TaskProfile> {
        // Critical first, then HighThroughput, Precision, Background
        let order = [
            IntentKind::Critical,
            IntentKind::HighThroughput,
            IntentKind::Precision,
            IntentKind::Background,
        ];
        for kind in &order {
            if let Some(queue) = self.queues.get_mut(kind) {
                if let Some(task) = queue.pop() {
                    return Some(task);
                }
            }
        }
        None
    }

    /// Get the number of pending tasks per priority level.
    pub fn pending_counts(&self) -> HashMap<IntentKind, usize> {
        self.queues
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect()
    }

    /// Total pending tasks.
    pub fn total_pending(&self) -> usize {
        self.queues.values().map(|v| v.len()).sum()
    }
}

impl Default for PriorityRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Resource Allocator ────────────────────────────────────────

/// Allocates resources based on intent requirements.
pub struct ResourceAllocator {
    total_memory: usize,
    total_cores: u32,
    allocated_memory: usize,
    allocated_cores: u32,
}

impl ResourceAllocator {
    pub fn new(total_memory: usize, total_cores: u32) -> Self {
        Self {
            total_memory,
            total_cores,
            allocated_memory: 0,
            allocated_cores: 0,
        }
    }

    /// Try to allocate resources for a task. Returns true if allocation succeeds.
    pub fn allocate(&mut self, profile: &TaskProfile) -> bool {
        let needed_memory = profile.memory_bytes;
        let needed_cores = match profile.intent {
            IntentKind::Critical => 1,    // Pin to one core
            IntentKind::HighThroughput => 4, // Use many cores
            IntentKind::Precision => 2,
            IntentKind::Background => 1,
        };

        if self.allocated_memory + needed_memory > self.total_memory {
            return false;
        }
        if self.allocated_cores + needed_cores > self.total_cores {
            return false;
        }

        self.allocated_memory += needed_memory;
        self.allocated_cores += needed_cores;
        true
    }

    /// Release resources after task completion.
    pub fn release(&mut self, profile: &TaskProfile) {
        let needed_cores = match profile.intent {
            IntentKind::Critical => 1,
            IntentKind::HighThroughput => 4,
            IntentKind::Precision => 2,
            IntentKind::Background => 1,
        };

        self.allocated_memory = self.allocated_memory.saturating_sub(profile.memory_bytes);
        self.allocated_cores = self.allocated_cores.saturating_sub(needed_cores);
    }

    /// Available memory.
    pub fn available_memory(&self) -> usize {
        self.total_memory - self.allocated_memory
    }

    /// Available cores.
    pub fn available_cores(&self) -> u32 {
        self.total_cores - self.allocated_cores
    }
}

// ─── Errors ────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IntentError {
    #[error("Failed to parse annotation: {0}")]
    ParseError(String),
    #[error("Unknown intent: {0}")]
    UnknownIntent(String),
    #[error("Invalid resource value: {0}")]
    InvalidResource(String),
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_annotation() {
        let ann = parse_annotation("#[intent(Critical)]").unwrap();
        assert_eq!(ann.kind, IntentKind::Critical);
        assert!(ann.resources.ops.is_none());
    }

    #[test]
    fn test_parse_with_resources() {
        let ann = parse_annotation("#[intent(HighThroughput, memory = \"4GB\", ops = 1000000)]")
            .unwrap();
        assert_eq!(ann.kind, IntentKind::HighThroughput);
        assert_eq!(ann.resources.memory_bytes, Some(4 * 1024 * 1024 * 1024));
        assert_eq!(ann.resources.ops, Some(1_000_000));
    }

    #[test]
    fn test_parse_with_cores() {
        let ann = parse_annotation("#[intent(Precision, cores = 8)]").unwrap();
        assert_eq!(ann.kind, IntentKind::Precision);
        assert_eq!(ann.resources.cores, Some(8));
    }

    #[test]
    fn test_parse_all_intents() {
        assert_eq!(IntentKind::from_str("Critical"), Some(IntentKind::Critical));
        assert_eq!(IntentKind::from_str("HighThroughput"), Some(IntentKind::HighThroughput));
        assert_eq!(IntentKind::from_str("Precision"), Some(IntentKind::Precision));
        assert_eq!(IntentKind::from_str("Background"), Some(IntentKind::Background));
        assert_eq!(IntentKind::from_str("Unknown"), None);
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(IntentKind::from_str("critical"), Some(IntentKind::Critical));
        assert_eq!(IntentKind::from_str("CRITICAL"), Some(IntentKind::Critical));
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(parse_annotation("not an annotation").is_err());
        assert!(parse_annotation("#[intent()]").is_err());
    }

    #[test]
    fn test_parse_unknown_intent() {
        assert!(parse_annotation("#[intent(Bogus)]").is_err());
    }

    #[test]
    fn test_parse_source_code() {
        let source = r#"
fn compute() {
    #[intent(Critical)]
    fn trade() { }

    #[intent(HighThroughput, memory = "2GB")]
    fn train() { }
}
"#;
        let annotations = parse_annotations(source);
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].kind, IntentKind::Critical);
        assert_eq!(annotations[0].line, 3);
        assert_eq!(annotations[1].kind, IntentKind::HighThroughput);
        assert_eq!(annotations[1].line, 6);
    }

    #[test]
    fn test_profile_from_annotation() {
        let ann = parse_annotation("#[intent(HighThroughput, ops = 5000000)]").unwrap();
        let profile = profile_from_annotation(&ann);
        assert_eq!(profile.intent, IntentKind::HighThroughput);
        assert_eq!(profile.estimated_ops, 5_000_000);
    }

    #[test]
    fn test_profile_device_target() {
        let profile = TaskProfile {
            intent: IntentKind::Critical,
            estimated_ops: 0,
            memory_bytes: 0,
            dependencies: 0,
        };
        assert_eq!(profile.device_target(), DeviceTarget::Cpu);

        let profile = TaskProfile {
            intent: IntentKind::HighThroughput,
            estimated_ops: 0,
            memory_bytes: 1024 * 1024 * 1024, // 1GB
            dependencies: 0,
        };
        assert_eq!(profile.device_target(), DeviceTarget::Gpu);
    }

    #[test]
    fn test_priority_router() {
        let mut router = PriorityRouter::new();

        router.submit(TaskProfile {
            intent: IntentKind::Background,
            estimated_ops: 100,
            memory_bytes: 0,
            dependencies: 0,
        });

        router.submit(TaskProfile {
            intent: IntentKind::Critical,
            estimated_ops: 1000,
            memory_bytes: 0,
            dependencies: 0,
        });

        assert_eq!(router.total_pending(), 2);

        // Critical should come first
        let task = router.pop_highest().unwrap();
        assert_eq!(task.intent, IntentKind::Critical);

        let task = router.pop_highest().unwrap();
        assert_eq!(task.intent, IntentKind::Background);

        assert!(router.pop_highest().is_none());
    }

    #[test]
    fn test_pending_counts() {
        let mut router = PriorityRouter::new();
        router.submit(TaskProfile {
            intent: IntentKind::Critical,
            estimated_ops: 0,
            memory_bytes: 0,
            dependencies: 0,
        });
        router.submit(TaskProfile {
            intent: IntentKind::Critical,
            estimated_ops: 0,
            memory_bytes: 0,
            dependencies: 0,
        });

        let counts = router.pending_counts();
        assert_eq!(counts[&IntentKind::Critical], 2);
        assert_eq!(counts[&IntentKind::Background], 0);
    }

    #[test]
    fn test_resource_allocator() {
        let mut alloc = ResourceAllocator::new(1024 * 1024 * 1024, 8); // 1GB, 8 cores

        let bg_task = TaskProfile {
            intent: IntentKind::Background,
            estimated_ops: 0,
            memory_bytes: 512 * 1024 * 1024, // 512MB
            dependencies: 0,
        };

        assert!(alloc.allocate(&bg_task));
        assert_eq!(alloc.available_memory(), 512 * 1024 * 1024);
        assert_eq!(alloc.available_cores(), 7);

        alloc.release(&bg_task);
        assert_eq!(alloc.available_memory(), 1024 * 1024 * 1024);
        assert_eq!(alloc.available_cores(), 8);
    }

    #[test]
    fn test_resource_allocator_rejects_overallocation() {
        let mut alloc = ResourceAllocator::new(100, 4);

        let task1 = TaskProfile {
            intent: IntentKind::Background,
            estimated_ops: 0,
            memory_bytes: 80,
            dependencies: 0,
        };

        let task2 = TaskProfile {
            intent: IntentKind::Background,
            estimated_ops: 0,
            memory_bytes: 50,
            dependencies: 0,
        };

        assert!(alloc.allocate(&task1));
        assert!(!alloc.allocate(&task2)); // 80 + 50 > 100
    }

    #[test]
    fn test_memory_parsing() {
        assert_eq!(ResourceHints::parse_memory_str("4GB"), Some(4 * 1024 * 1024 * 1024));
        assert_eq!(ResourceHints::parse_memory_str("512MB"), Some(512 * 1024 * 1024));
        assert_eq!(ResourceHints::parse_memory_str("1024KB"), Some(1024 * 1024));
        assert_eq!(ResourceHints::parse_memory_str("4096"), Some(4096));
        assert_eq!(ResourceHints::parse_memory_str("abc"), None);
    }

    #[test]
    fn test_intent_max_latency() {
        assert!(IntentKind::Critical.max_latency_us() < IntentKind::Background.max_latency_us());
    }

    #[test]
    fn test_intent_priority_ordering() {
        assert!(IntentKind::Critical.priority() > IntentKind::HighThroughput.priority());
        assert!(IntentKind::HighThroughput.priority() == IntentKind::Precision.priority());
        assert!(IntentKind::Precision.priority() > IntentKind::Background.priority());
    }
}
