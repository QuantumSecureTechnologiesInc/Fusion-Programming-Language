//! Feature Toggle Engine for the Fusion Vortex compiler.
//!
//! Ensures that when a module declares `uses: [Continuations]` in its mod.fu,
//! the compiler injects the correct transforms (like CPS) before TCO,
//! preventing feature conflicts.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

// ---- Feature Registration ----

#[derive(Debug, Clone)]
pub struct FeatureToggle {
    pub name: String,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub transform_pass: String,
    pub priority: u32,
}

// ---- Global Feature Registry ----

struct FeatureRegistry {
    features: HashMap<String, FeatureToggle>,
}

impl FeatureRegistry {
    fn new() -> Self {
        let mut registry = FeatureRegistry {
            features: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    fn register_all(&mut self) {
        let defs: Vec<(&str, Vec<&str>, Vec<&str>, &str, u32)> = vec![
            ("AlgebraicEffects", vec![], vec![], "effect_handler_injection", 1),
            ("LinearTypes", vec![], vec![], "linear_resource_tracking", 2),
            ("DependentTypes", vec![], vec![], "dependent_check_insertion", 3),
            ("GradualTyping", vec![], vec![], "gradual_annotation_check", 4),
            ("TailCallOptimization", vec![], vec![], "tco_loop_transform", 5),
            ("Continuations", vec![], vec!["TailCallOptimization"], "cps_transform", 6),
            ("CapabilitySecurity", vec![], vec![], "capability_check_injection", 7),
            ("MultipleDispatch", vec![], vec![], "dispatch_table_generation", 8),
            ("EffectPolymorphism", vec!["AlgebraicEffects"], vec![], "effect_specialization", 9),
            ("FormalVerification", vec![], vec![], "verification_hook_injection", 10),
            ("TailModuloCons", vec!["TailCallOptimization"], vec![], "tmc_transform", 11),
            ("PartialEvaluation", vec![], vec![], "staging_specialization", 12),
            ("RefinementTypes", vec!["DependentTypes"], vec![], "refinement_check_insertion", 13),
            ("ActorModel", vec![], vec![], "actor_supervision_injection", 14),
            ("CustomAllocators", vec![], vec![], "allocator_replacement", 15),
            ("UnsafeProvenance", vec![], vec![], "provenance_tracking_injection", 16),
        ];

        for (name, deps, conflicts, transform, priority) in defs {
            self.toggle_register(
                name.to_string(),
                deps.into_iter().map(String::from).collect(),
                conflicts.into_iter().map(String::from).collect(),
                transform.to_string(),
                priority,
            );
        }
    }

    fn toggle_register(
        &mut self,
        name: String,
        dependencies: Vec<String>,
        conflicts: Vec<String>,
        transform_pass: String,
        priority: u32,
    ) {
        self.features.insert(
            name.clone(),
            FeatureToggle {
                name,
                dependencies,
                conflicts,
                transform_pass,
                priority,
            },
        );
    }
}

// ---- Conflict Matrix ----

struct ConflictMatrix {
    pairs: HashMap<(String, String), String>,
}

impl ConflictMatrix {
    fn new() -> Self {
        let mut pairs = HashMap::new();

        let conflicts: Vec<(&str, &str, &str)> = vec![
            (
                "Continuations",
                "TailCallOptimization",
                "Cannot use TCO with Continuations - CPS transform conflicts with loop-based TCO",
            ),
            (
                "CapabilitySecurity",
                "UnsafeProvenance",
                "Capabilities and Unsafe Provenance are mutually exclusive - one is runtime, one is compile-time",
            ),
            (
                "GradualTyping",
                "LinearTypes",
                "Gradual typing weakens linear type guarantees",
            ),
            (
                "DependentTypes",
                "GradualTyping",
                "Dependent types require static typing",
            ),
            (
                "FormalVerification",
                "GradualTyping",
                "Formal verification requires static typing",
            ),
        ];

        for (a, b, msg) in conflicts {
            pairs.insert(
                (a.to_string(), b.to_string()),
                msg.to_string(),
            );
            pairs.insert(
                (b.to_string(), a.to_string()),
                msg.to_string(),
            );
        }

        ConflictMatrix { pairs }
    }

    fn get_message(&self, feat1: &str, feat2: &str) -> Option<&str> {
        self.pairs
            .get(&(feat1.to_string(), feat2.to_string()))
            .map(|s| s.as_str())
    }

    fn has_conflict(&self, feat1: &str, feat2: &str) -> bool {
        self.pairs
            .contains_key(&(feat1.to_string(), feat2.to_string()))
    }
}

// ---- Public API ----

fn registry() -> &'static FeatureRegistry {
    static REG: OnceLock<FeatureRegistry> = OnceLock::new();
    REG.get_or_init(FeatureRegistry::new)
}

fn conflicts() -> &'static ConflictMatrix {
    static CONF: OnceLock<ConflictMatrix> = OnceLock::new();
    CONF.get_or_init(ConflictMatrix::new)
}

/// Register a feature toggle with its metadata.
pub fn toggle_register(
    name: String,
    deps: Vec<String>,
    conflict_list: Vec<String>,
    transform: String,
    priority: u32,
) {
    let mut reg = FeatureRegistry::new();
    reg.toggle_register(name, deps, conflict_list, transform, priority);
}

/// Resolve features from a `uses` list, pulling in dependencies and sorting by priority.
pub fn toggle_resolve(uses: &[String]) -> Vec<FeatureToggle> {
    let mut resolved = HashSet::new();
    let mut queue: Vec<String> = uses.to_vec();

    while let Some(name) = queue.pop() {
        if resolved.contains(&name) {
            continue;
        }
        if let Some(feat) = registry().features.get(&name) {
            resolved.insert(name.clone());
            for dep in &feat.dependencies {
                if !resolved.contains(dep) {
                    queue.push(dep.clone());
                }
            }
        }
    }

    let mut features: Vec<FeatureToggle> = resolved
        .iter()
        .filter_map(|name| registry().features.get(name).cloned())
        .collect();

    features.sort_by_key(|f| f.priority);
    features
}

/// Check for conflicts among the requested features. Returns pairs of conflicting features.
pub fn toggle_check_conflicts(uses: &[String]) -> Vec<(String, String)> {
    let mut result = Vec::new();

    for i in 0..uses.len() {
        for j in (i + 1)..uses.len() {
            if conflicts().has_conflict(&uses[i], &uses[j]) {
                result.push((uses[i].clone(), uses[j].clone()));
            }
        }
    }

    result
}

/// Inject transforms into IR in priority order for the requested features.
pub fn toggle_inject_transforms(uses: &[String], ir: &str) -> String {
    let features = toggle_resolve(uses);
    let mut result = ir.to_string();

    for feat in &features {
        result = format!(
            "// --- Transform: {} (priority: {}) ---\n// Feature: {}\n{}",
            feat.transform_pass, feat.priority, feat.name, result
        );
    }

    result
}

/// Get the ordered list of transform pass names for the requested features.
pub fn toggle_get_transform_order(uses: &[String]) -> Vec<String> {
    let features = toggle_resolve(uses);
    features.iter().map(|f| f.transform_pass.clone()).collect()
}

/// Check if a specific feature is enabled (directly or via dependencies).
pub fn toggle_is_enabled(uses: &[String], feature: &str) -> bool {
    let features = toggle_resolve(uses);
    features.iter().any(|f| f.name == feature)
}

/// Get a human-readable conflict message between two features.
pub fn toggle_get_conflict_message(feat1: &str, feat2: &str) -> String {
    match conflicts().get_message(feat1, feat2) {
        Some(msg) => msg.to_string(),
        None => format!("No known conflict between {} and {}", feat1, feat2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_with_dependencies() {
        let uses = vec!["EffectPolymorphism".to_string()];
        let resolved = toggle_resolve(&uses);
        let names: Vec<&str> = resolved.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"AlgebraicEffects"));
        assert!(names.contains(&"EffectPolymorphism"));
    }

    #[test]
    fn test_resolve_priority_order() {
        let uses = vec![
            "FormalVerification".to_string(),
            "AlgebraicEffects".to_string(),
            "TailCallOptimization".to_string(),
        ];
        let resolved = toggle_resolve(&uses);
        let names: Vec<&str> = resolved.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names[0], "AlgebraicEffects");
        assert_eq!(names[1], "TailCallOptimization");
        assert_eq!(names[2], "FormalVerification");
    }

    #[test]
    fn test_check_conflicts() {
        let uses = vec![
            "Continuations".to_string(),
            "TailCallOptimization".to_string(),
        ];
        let conflicts = toggle_check_conflicts(&uses);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_check_no_conflicts() {
        let uses = vec![
            "AlgebraicEffects".to_string(),
            "LinearTypes".to_string(),
        ];
        let conflicts = toggle_check_conflicts(&uses);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_is_enabled_direct() {
        let uses = vec!["Continuations".to_string()];
        assert!(toggle_is_enabled(&uses, "Continuations"));
        assert!(!toggle_is_enabled(&uses, "TailCallOptimization"));
    }

    #[test]
    fn test_is_enabled_via_dependency() {
        let uses = vec!["EffectPolymorphism".to_string()];
        assert!(toggle_is_enabled(&uses, "AlgebraicEffects"));
        assert!(toggle_is_enabled(&uses, "EffectPolymorphism"));
    }

    #[test]
    fn test_inject_transforms() {
        let uses = vec!["Continuations".to_string()];
        let ir = "fn main() { ret }";
        let result = toggle_inject_transforms(&uses, &ir);
        assert!(result.contains("cps_transform"));
        assert!(result.contains("fn main() { ret }"));
    }

    #[test]
    fn test_get_transform_order() {
        let uses = vec![
            "AlgebraicEffects".to_string(),
            "EffectPolymorphism".to_string(),
        ];
        let order = toggle_get_transform_order(&uses);
        assert_eq!(order[0], "effect_handler_injection");
        assert_eq!(order[1], "effect_specialization");
    }

    #[test]
    fn test_conflict_message() {
        let msg = toggle_get_conflict_message("Continuations", "TailCallOptimization");
        assert!(msg.contains("CPS transform conflicts"));
    }

    #[test]
    fn test_no_conflict_message() {
        let msg = toggle_get_conflict_message("AlgebraicEffects", "LinearTypes");
        assert!(msg.contains("No known conflict"));
    }
}
