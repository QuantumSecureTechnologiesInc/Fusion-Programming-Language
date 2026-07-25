//! Interaction Witness System for the Fusion V2.0 Vortex compiler.
//!
//! Generates metadata hashes for compiled functions that prove behavioral
//! consistency across all 16 features. When a cross-reference violates the
//! conflict matrix, the compiler emits a human-readable diagnostic error.
//!
//! The witness is deterministic: the same function name + feature set always
//! produces the same hash. This enables incremental verification — if only
//! unrelated code changes, witnesses remain stable.

use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::feature_toggle;

// ---------------------------------------------------------------------------
// Witness structs
// ---------------------------------------------------------------------------

/// A single conflict detected between two features used by a function.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WitnessConflict {
    pub feature_a: String,
    pub feature_b: String,
    pub reason: String,
    pub severity: String,
}

/// Metadata hash proving a function's feature-set is consistent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InteractionWitness {
    pub function_name: String,
    pub feature_set: Vec<String>,
    pub witness_hash: String,
    pub timestamp: String,
    pub conflicts: Vec<WitnessConflict>,
}

/// Aggregated witness report for a set of functions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WitnessReport {
    pub witnesses: Vec<InteractionWitness>,
    pub total_conflicts: usize,
    pub is_compatible: bool,
}

/// Witness for all functions inside a single module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleWitness {
    pub module_name: String,
    pub function_witnesses: Vec<InteractionWitness>,
    pub total_conflicts: usize,
    pub is_compatible: bool,
}

/// Witness for an entire program (collection of modules).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgramWitness {
    pub modules: Vec<ModuleWitness>,
    pub total_conflicts: usize,
    pub is_compatible: bool,
}

// ---------------------------------------------------------------------------
// Hash generation
// ---------------------------------------------------------------------------

/// Compute a SHA-256 hash of a function name concatenated with its sorted
/// feature list.  The sort guarantees that `{A, B}` and `{B, A}` produce
/// the same witness.
pub fn witness_hash_function(name: &str, features: &[String]) -> String {
    let mut sorted = features.to_vec();
    sorted.sort();

    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    for feat in &sorted {
        hasher.update(b":");
        hasher.update(feat.as_bytes());
    }
    hex::encode(hasher.finalize())
}

/// Generate a complete `InteractionWitness` for one function.
pub fn witness_generate(function_name: &str, features_used: Vec<String>) -> InteractionWitness {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "0".to_string());

    let witness_hash = witness_hash_function(function_name, &features_used);
    let conflicts = witness_check_conflicts_raw(&features_used);

    InteractionWitness {
        function_name: function_name.to_string(),
        feature_set: features_used,
        witness_hash,
        timestamp,
        conflicts,
    }
}

/// Verify that the stored hash matches the recomputed hash.
pub fn witness_verify(witness: &InteractionWitness) -> bool {
    let recomputed = witness_hash_function(&witness.function_name, &witness.feature_set);
    recomputed == witness.witness_hash
}

// ---------------------------------------------------------------------------
// Conflict detection
// ---------------------------------------------------------------------------

/// Internal: check every feature pair against the conflict matrix and return
/// `WitnessConflict` entries for each violation.
fn witness_check_conflicts_raw(features: &[String]) -> Vec<WitnessConflict> {
    let mut out = Vec::new();

    for i in 0..features.len() {
        for j in (i + 1)..features.len() {
            let a = &features[i];
            let b = &features[j];

            let msg = feature_toggle::toggle_get_conflict_message(a, b);
            if !msg.starts_with("No known conflict") {
                out.push(WitnessConflict {
                    feature_a: a.clone(),
                    feature_b: b.clone(),
                    reason: msg,
                    severity: "error".to_string(),
                });
            }
        }
    }

    out
}

/// Check all feature pairs in a witness for conflicts.
pub fn witness_check_conflicts(witness: &InteractionWitness) -> Vec<WitnessConflict> {
    witness_check_conflicts_raw(&witness.feature_set)
}

/// Returns `true` when no conflicting feature pair exists.
pub fn witness_is_compatible(witness: &InteractionWitness) -> bool {
    witness_check_conflicts_raw(&witness.feature_set).is_empty()
}

/// Map a conflict's underlying reason to a severity level.
///
/// Currently every known conflict is `"error"` because the Fusion conflict
/// matrix only records hard incompatibilities.  The function is provided for
/// future extensibility (soft warnings, informational notes, etc.).
pub fn witness_severity_level(conflict: &WitnessConflict) -> &str {
    match conflict.severity.as_str() {
        "error" | "warning" | "info" => &conflict.severity,
        _ => "error",
    }
}

// ---------------------------------------------------------------------------
// Error generation
// ---------------------------------------------------------------------------

/// Produce a human-readable error string for one conflict.
pub fn witness_conflict_error(conflict: &WitnessConflict) -> String {
    format!(
        "Feature conflict: `{}` and `{}` cannot coexist — {}.",
        conflict.feature_a, conflict.feature_b, conflict.reason,
    )
}

/// Produce error strings for every conflict in a report.
pub fn witness_report_errors(report: &WitnessReport) -> Vec<String> {
    let mut msgs = Vec::new();
    for w in &report.witnesses {
        for c in &w.conflicts {
            msgs.push(format!(
                "Function `{}`: {}",
                w.function_name,
                witness_conflict_error(c),
            ));
        }
    }
    msgs
}

/// Format a full `WitnessReport` as a human-readable string.
pub fn witness_format_report(report: &WitnessReport) -> String {
    let mut buf = String::from("=== Interaction Witness Report ===\n\n");
    buf.push_str(&format!(
        "Total functions: {}  |  Total conflicts: {}  |  Compatible: {}\n\n",
        report.witnesses.len(),
        report.total_conflicts,
        if report.is_compatible { "yes" } else { "NO" },
    ));

    for w in &report.witnesses {
        buf.push_str(&format!(
            "  fn {} — hash: {} — conflicts: {}\n",
            w.function_name,
            &w.witness_hash[..16.min(w.witness_hash.len())],
            w.conflicts.len(),
        ));
        for c in &w.conflicts {
            buf.push_str(&format!(
                "    [{}] {} vs {}: {}\n",
                c.severity, c.feature_a, c.feature_b, c.reason,
            ));
        }
    }

    if !report.is_compatible {
        buf.push_str("\n--- Errors ---\n");
        for msg in witness_report_errors(report) {
            buf.push_str(&format!("  {}\n", msg));
        }
    }

    buf
}

// ---------------------------------------------------------------------------
// Module-level witnesses
// ---------------------------------------------------------------------------

/// Generate witnesses for every function in a module.
///
/// `functions` is a list of `(function_name, features_used)` pairs.
pub fn module_witness_generate(
    module_name: &str,
    functions: Vec<(String, Vec<String>)>,
) -> ModuleWitness {
    let function_witnesses: Vec<InteractionWitness> = functions
        .into_iter()
        .map(|(name, feats)| witness_generate(&name, feats))
        .collect();

    let total_conflicts: usize = function_witnesses.iter().map(|w| w.conflicts.len()).sum();
    let is_compatible = total_conflicts == 0;

    ModuleWitness {
        module_name: module_name.to_string(),
        function_witnesses,
        total_conflicts,
        is_compatible,
    }
}

/// Check whether every function witness in the module is compatible.
pub fn module_witness_check(module: &ModuleWitness) -> bool {
    module.is_compatible
}

/// Collect all human-readable error strings for a module.
pub fn module_witness_errors(module: &ModuleWitness) -> Vec<String> {
    let report = WitnessReport {
        witnesses: module.function_witnesses.clone(),
        total_conflicts: module.total_conflicts,
        is_compatible: module.is_compatible,
    };
    witness_report_errors(&report)
}

// ---------------------------------------------------------------------------
// Program-level witnesses
// ---------------------------------------------------------------------------

/// Aggregate module witnesses into a single program witness.
pub fn program_witness_generate(modules: Vec<ModuleWitness>) -> ProgramWitness {
    let total_conflicts: usize = modules.iter().map(|m| m.total_conflicts).sum();
    let is_compatible = total_conflicts == 0;

    ProgramWitness {
        modules,
        total_conflicts,
        is_compatible,
    }
}

/// Check whether the entire program is feature-compatible.
pub fn program_witness_check(program: &ProgramWitness) -> bool {
    program.is_compatible
}

/// Generate a full program-level report string.
pub fn program_witness_report(program: &ProgramWitness) -> String {
    let mut buf = String::from("=== Program Interaction Witness Report ===\n\n");
    buf.push_str(&format!(
        "Modules: {}  |  Total conflicts: {}  |  Compatible: {}\n\n",
        program.modules.len(),
        program.total_conflicts,
        if program.is_compatible { "yes" } else { "NO" },
    ));

    for m in &program.modules {
        buf.push_str(&format!(
            "Module `{}` — functions: {} — conflicts: {} — compatible: {}\n",
            m.module_name,
            m.function_witnesses.len(),
            m.total_conflicts,
            if m.is_compatible { "yes" } else { "NO" },
        ));
        for w in &m.function_witnesses {
            for c in &w.conflicts {
                buf.push_str(&format!(
                    "    [{}] fn `{}`: {} vs {} — {}\n",
                    c.severity, w.function_name, c.feature_a, c.feature_b, c.reason,
                ));
            }
        }
    }

    if !program.is_compatible {
        buf.push_str("\n--- Blocking Errors ---\n");
        for m in &program.modules {
            for msg in module_witness_errors(m) {
                buf.push_str(&format!("  {}\n", msg));
            }
        }
    }

    buf
}

/// Validate the program. Returns `Ok(())` when all functions are compatible,
/// or `Err` with a list of human-readable error messages.
pub fn program_witness_validate(program: &ProgramWitness) -> Result<(), Vec<String>> {
    if program.is_compatible {
        return Ok(());
    }

    let mut errors = Vec::new();
    for m in &program.modules {
        errors.extend(module_witness_errors(m));
    }
    Err(errors)
}

// ---------------------------------------------------------------------------
// Display impls
// ---------------------------------------------------------------------------

impl fmt::Display for WitnessConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} vs {} — {}",
            self.severity, self.feature_a, self.feature_b, self.reason,
        )
    }
}

impl fmt::Display for InteractionWitness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Witness(fn={}, hash={}, conflicts={})",
            self.function_name,
            &self.witness_hash[..16.min(self.witness_hash.len())],
            self.conflicts.len(),
        )
    }
}

impl fmt::Display for WitnessReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WitnessReport(functions={}, conflicts={}, compatible={})",
            self.witnesses.len(),
            self.total_conflicts,
            self.is_compatible,
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Hash generation --

    #[test]
    fn test_witness_hash_deterministic() {
        let h1 = witness_hash_function("main", &["A".into(), "B".into()]);
        let h2 = witness_hash_function("main", &["B".into(), "A".into()]);
        assert_eq!(h1, h2, "hash should be order-independent");
    }

    #[test]
    fn test_witness_hash_different_names() {
        let h1 = witness_hash_function("foo", &["A".into()]);
        let h2 = witness_hash_function("bar", &["A".into()]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_witness_hash_different_features() {
        let h1 = witness_hash_function("main", &["A".into()]);
        let h2 = witness_hash_function("main", &["B".into()]);
        assert_ne!(h1, h2);
    }

    // -- Witness generation & verification --

    #[test]
    fn test_witness_generate_and_verify() {
        let w = witness_generate("process", vec!["LinearTypes".into(), "ActorModel".into()]);
        assert_eq!(w.function_name, "process");
        assert_eq!(w.feature_set.len(), 2);
        assert!(witness_verify(&w));
    }

    #[test]
    fn test_witness_verify_detects_tampering() {
        let mut w = witness_generate("process", vec!["LinearTypes".into()]);
        w.witness_hash = "0000000000000000000000000000000000000000000000000000000000000000".into();
        assert!(!witness_verify(&w));
    }

    // -- Conflict detection --

    #[test]
    fn test_conflicting_features_detected() {
        let w = witness_generate(
            "handler",
            vec!["Continuations".into(), "TailCallOptimization".into()],
        );
        assert_eq!(w.conflicts.len(), 1);
        assert_eq!(w.conflicts[0].feature_a, "Continuations");
        assert_eq!(w.conflicts[0].feature_b, "TailCallOptimization");
    }

    #[test]
    fn test_no_conflict_compatible() {
        let w = witness_generate(
            "safe_fn",
            vec!["AlgebraicEffects".into(), "ActorModel".into()],
        );
        assert!(w.conflicts.is_empty());
        assert!(witness_is_compatible(&w));
    }

    #[test]
    fn test_severity_level() {
        let c = WitnessConflict {
            feature_a: "A".into(),
            feature_b: "B".into(),
            reason: "reason".into(),
            severity: "error".into(),
        };
        assert_eq!(witness_severity_level(&c), "error");

        let c2 = WitnessConflict {
            feature_a: "A".into(),
            feature_b: "B".into(),
            reason: "reason".into(),
            severity: "warning".into(),
        };
        assert_eq!(witness_severity_level(&c2), "warning");

        let c3 = WitnessConflict {
            feature_a: "A".into(),
            feature_b: "B".into(),
            reason: "reason".into(),
            severity: "unknown".into(),
        };
        assert_eq!(witness_severity_level(&c3), "error");
    }

    // -- Error generation --

    #[test]
    fn test_conflict_error_message() {
        let c = WitnessConflict {
            feature_a: "Continuations".into(),
            feature_b: "TailCallOptimization".into(),
            reason: "CPS transform conflicts with loop-based TCO".into(),
            severity: "error".into(),
        };
        let msg = witness_conflict_error(&c);
        assert!(msg.contains("Continuations"));
        assert!(msg.contains("TailCallOptimization"));
        assert!(msg.contains("CPS transform"));
    }

    #[test]
    fn test_report_errors_multiple_functions() {
        let r = WitnessReport {
            witnesses: vec![
                witness_generate("a", vec!["Continuations".into(), "TailCallOptimization".into()]),
                witness_generate("b", vec!["GradualTyping".into(), "LinearTypes".into()]),
            ],
            total_conflicts: 2,
            is_compatible: false,
        };
        let errors = witness_report_errors(&r);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_format_report() {
        let r = WitnessReport {
            witnesses: vec![witness_generate("f", vec![])],
            total_conflicts: 0,
            is_compatible: true,
        };
        let formatted = witness_format_report(&r);
        assert!(formatted.contains("Witness Report"));
        assert!(formatted.contains("Compatible: yes"));
    }

    // -- Module witnesses --

    #[test]
    fn test_module_witness_compatible() {
        let m = module_witness_generate(
            "utils",
            vec![
                ("add".into(), vec!["AlgebraicEffects".into()]),
                ("mul".into(), vec!["ActorModel".into()]),
            ],
        );
        assert!(module_witness_check(&m));
        assert!(module_witness_errors(&m).is_empty());
    }

    #[test]
    fn test_module_witness_conflict() {
        let m = module_witness_generate(
            "handler",
            vec![
                ("process".into(), vec!["Continuations".into(), "TailCallOptimization".into()]),
            ],
        );
        assert!(!module_witness_check(&m));
        assert!(!module_witness_errors(&m).is_empty());
    }

    // -- Program witnesses --

    #[test]
    fn test_program_witness_validate_ok() {
        let m1 = module_witness_generate("a", vec![("f1".into(), vec![])]);
        let m2 = module_witness_generate("b", vec![("f2".into(), vec![])]);
        let p = program_witness_generate(vec![m1, m2]);
        assert!(program_witness_check(&p));
        assert!(program_witness_validate(&p).is_ok());
    }

    #[test]
    fn test_program_witness_validate_err() {
        let m = module_witness_generate(
            "bad",
            vec![("f".into(), vec!["Continuations".into(), "TailCallOptimization".into()])],
        );
        let p = program_witness_generate(vec![m]);
        assert!(!program_witness_check(&p));
        let result = program_witness_validate(&p);
        assert!(result.is_err());
        assert!(!result.unwrap_err().is_empty());
    }

    #[test]
    fn test_program_witness_report() {
        let m = module_witness_generate(
            "network_handler",
            vec![
                ("handle_request".into(), vec!["Continuations".into(), "TailCallOptimization".into()]),
                ("send_response".into(), vec!["AlgebraicEffects".into()]),
            ],
        );
        let p = program_witness_generate(vec![m]);
        let report = program_witness_report(&p);
        assert!(report.contains("Program Interaction Witness Report"));
        assert!(report.contains("network_handler"));
        assert!(report.contains("Blocking Errors"));
    }

    // -- Display --

    #[test]
    fn test_display_conflict() {
        let c = WitnessConflict {
            feature_a: "A".into(),
            feature_b: "B".into(),
            reason: "bad".into(),
            severity: "error".into(),
        };
        let s = format!("{}", c);
        assert!(s.contains("[error]"));
        assert!(s.contains("A vs B"));
    }

    #[test]
    fn test_display_witness() {
        let w = witness_generate("fn", vec![]);
        let s = format!("{}", w);
        assert!(s.contains("Witness(fn="));
    }
}
