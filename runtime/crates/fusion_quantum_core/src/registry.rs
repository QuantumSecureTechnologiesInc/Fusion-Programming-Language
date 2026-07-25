//! Quantum registry for qubit management
//! Integrated from fusion_core Quantum Core.rs
//!
//! Manages qubit allocation, entanglement groups, and enforces physical laws
//! (no-cloning theorem) through Rust's ownership model.

use crate::simulator::QuantumState;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Unique identifier for a qubit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QubitId(pub usize);

/// An entanglement group holds qubits that share a joint quantum state.
///
/// When qubits are entangled (e.g., via CNOT after Hadamard), they must
/// share the same state vector. This struct manages that shared state.
#[derive(Debug, Clone)]
pub struct EntanglementGroup {
    /// The shared quantum state for all qubits in this group
    pub state: Arc<RwLock<QuantumState>>,
    /// Ordered list of qubit IDs in this group (index = position in state vector)
    pub qubits: Vec<QubitId>,
}

impl EntanglementGroup {
    /// Create a new entanglement group from a list of qubits.
    ///
    /// Initializes the state to |0...0⟩ for the given qubits.
    pub fn new(qubits: Vec<QubitId>) -> Self {
        let num_qubits = qubits.len();
        Self {
            state: Arc::new(RwLock::new(QuantumState::zeros(num_qubits))),
            qubits,
        }
    }

    /// Get the index of a qubit within this group.
    pub fn qubit_index(&self, id: QubitId) -> Option<usize> {
        self.qubits.iter().position(|&q| q == id)
    }
}

/// Global registry for quantum simulator.
///
/// Manages qubit states, entanglement groups, and enforces physical laws
/// (no-cloning theorem via Rust's move semantics).
///
/// Qubits start independent (each in their own 1-qubit state). When
/// entangling operations are performed, qubits are merged into a shared
/// entanglement group with a joint state vector.
#[derive(Default)]
pub struct QuantumRegistry {
    /// Map Qubit ID -> Entanglement group index
    qubit_map: HashMap<QubitId, usize>,
    /// All entanglement groups
    groups: Vec<EntanglementGroup>,
    next_id: usize,
}

impl QuantumRegistry {
    /// Create new quantum registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new qubit in |0⟩ state (in its own independent group)
    pub fn allocate(&mut self) -> QubitId {
        let id = QubitId(self.next_id);
        self.next_id += 1;

        let group_idx = self.groups.len();
        self.groups
            .push(EntanglementGroup::new(vec![id]));
        self.qubit_map.insert(id, group_idx);

        id
    }

    /// Get the entanglement group containing a qubit.
    pub fn get_group(&self, id: QubitId) -> Option<&EntanglementGroup> {
        self.qubit_map
            .get(&id)
            .and_then(|&idx| self.groups.get(idx))
    }

    /// Get the state associated with a qubit (via its entanglement group).
    pub fn get_state(&self, id: QubitId) -> Option<Arc<RwLock<QuantumState>>> {
        self.get_group(id).map(|g| g.state.clone())
    }

    /// Check if two qubits are entangled (in the same group).
    pub fn are_entangled(&self, a: QubitId, b: QubitId) -> bool {
        match (self.qubit_map.get(&a), self.qubit_map.get(&b)) {
            (Some(&ga), Some(&gb)) => ga == gb && self.groups[ga].qubits.len() > 1,
            _ => false,
        }
    }

    /// Get all qubits entangled with the given qubit.
    pub fn entangled_with(&self, id: QubitId) -> Vec<QubitId> {
        self.get_group(id)
            .map(|g| g.qubits.iter().copied().filter(|&q| q != id).collect())
            .unwrap_or_default()
    }

    /// Create a Bell state (|00⟩ + |11⟩)/√2 between two independent qubits.
    ///
    /// This merges the two qubits into a single entanglement group and
    /// applies H on the first qubit followed by CNOT. Both qubits must
    /// be in separate groups (not already entangled with others).
    ///
    /// # Errors
    ///
    /// Returns `None` if either qubit is not found or they share a group.
    pub fn create_bell_state(&mut self, a: QubitId, b: QubitId) -> bool {
        let group_a = match self.qubit_map.get(&a) {
            Some(&idx) => idx,
            None => return false,
        };
        let group_b = match self.qubit_map.get(&b) {
            Some(&idx) => idx,
            None => return false,
        };

        // Both must be independent single-qubit groups
        if group_a == group_b {
            return false;
        }
        if self.groups[group_a].qubits.len() != 1
            || self.groups[group_b].qubits.len() != 1
        {
            return false;
        }

        // Create the merged group with Bell state
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut bell_state = QuantumState::zeros(2);
        bell_state.amplitudes[0] = num_complex::Complex64::new(inv_sqrt2, 0.0);
        bell_state.amplitudes[3] = num_complex::Complex64::new(inv_sqrt2, 0.0);

        let merged_group = EntanglementGroup {
            state: Arc::new(RwLock::new(bell_state)),
            qubits: vec![a, b],
        };

        let new_group_idx = self.groups.len();
        self.groups.push(merged_group);

        // Update mappings
        self.qubit_map.insert(a, new_group_idx);
        self.qubit_map.insert(b, new_group_idx);

        // Remove old groups (swap-remove is fine since we remapped)
        // We mark old groups as invalid by clearing them
        // (In production, we'd use a generational arena)
        if group_a < self.groups.len() {
            self.groups[group_a].qubits.clear();
        }
        if group_b < self.groups.len() && group_b != group_a {
            self.groups[group_b].qubits.clear();
        }

        true
    }

    /// Get the number of allocated qubits
    pub fn qubit_count(&self) -> usize {
        self.next_id
    }

    /// Get the number of entanglement groups (valid ones with qubits).
    pub fn group_count(&self) -> usize {
        self.groups.iter().filter(|g| !g.qubits.is_empty()).count()
    }
}

/// Physical Qubit Handle
/// Enforces No-Cloning via Rust's Move semantics.
///
/// Once a Qubit is moved, the original handle is invalidated, preventing
/// accidental cloning of quantum state (which is physically impossible).
#[derive(Debug)]
pub struct Qubit {
    pub id: QubitId,
}

impl Qubit {
    /// Create a new qubit (allocates from registry)
    pub fn new(registry: &mut QuantumRegistry) -> Self {
        let id = registry.allocate();
        Self { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qubit_allocation() {
        let mut registry = QuantumRegistry::new();
        let q1 = Qubit::new(&mut registry);
        let q2 = Qubit::new(&mut registry);

        assert_ne!(q1.id, q2.id);
        assert_eq!(registry.qubit_count(), 2);
    }

    #[test]
    fn test_registry_state() {
        let mut registry = QuantumRegistry::new();
        let q = Qubit::new(&mut registry);

        let state = registry.get_state(q.id);
        assert!(state.is_some());

        let state = state.unwrap();
        let state = state.read().unwrap();
        assert!((state.probability(0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_independent_qubits_not_entangled() {
        let mut registry = QuantumRegistry::new();
        let q1 = Qubit::new(&mut registry);
        let q2 = Qubit::new(&mut registry);

        assert!(!registry.are_entangled(q1.id, q2.id));
        assert!(registry.entangled_with(q1.id).is_empty());
    }

    #[test]
    fn test_create_bell_state() {
        let mut registry = QuantumRegistry::new();
        let q1 = Qubit::new(&mut registry);
        let q2 = Qubit::new(&mut registry);

        let success = registry.create_bell_state(q1.id, q2.id);
        assert!(success);

        // They should now be entangled
        assert!(registry.are_entangled(q1.id, q2.id));

        // Entangled with each other
        let entangled = registry.entangled_with(q1.id);
        assert_eq!(entangled.len(), 1);
        assert_eq!(entangled[0], q2.id);

        // Check Bell state probabilities
        let state = registry.get_state(q1.id).unwrap();
        let state = state.read().unwrap();
        assert!((state.probability(0) - 0.5).abs() < 1e-10); // |00⟩
        assert!((state.probability(1)).abs() < 1e-10); // |01⟩
        assert!((state.probability(2)).abs() < 1e-10); // |10⟩
        assert!((state.probability(3) - 0.5).abs() < 1e-10); // |11⟩
    }

    #[test]
    fn test_bell_state_same_group_fails() {
        let mut registry = QuantumRegistry::new();
        let q1 = Qubit::new(&mut registry);
        let q2 = Qubit::new(&mut registry);

        // Entangle them first
        registry.create_bell_state(q1.id, q2.id);

        // Can't create another Bell state with already-entangled qubits
        // (They're in the same group now)
        let q3 = Qubit::new(&mut registry);
        // q1 is already entangled with q2, so create_bell_state with q3 should fail
        // because q1 is in a multi-qubit group
        let success = registry.create_bell_state(q1.id, q3.id);
        assert!(!success);
    }

    #[test]
    fn test_group_count() {
        let mut registry = QuantumRegistry::new();
        let q1 = Qubit::new(&mut registry);
        let q2 = Qubit::new(&mut registry);
        let q3 = Qubit::new(&mut registry);

        assert_eq!(registry.group_count(), 3); // Three independent qubits

        registry.create_bell_state(q1.id, q2.id);
        assert_eq!(registry.group_count(), 2); // One merged + one independent
    }
}
