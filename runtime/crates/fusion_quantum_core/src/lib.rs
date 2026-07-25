//! # Fusion Quantum Core
//!
//! Quantum circuit construction and state vector simulation for Fusion.
//! Integrated from fusion_core.

mod circuit;
mod error;
mod gates;
mod registry;
mod simulator;

pub use circuit::QuantumCircuit;
pub use error::{QuantumError, QuantumResult};
pub use gates::QuantumGate;
pub use registry::{EntanglementGroup, QuantumRegistry, Qubit, QubitId};
pub use simulator::QuantumState;
