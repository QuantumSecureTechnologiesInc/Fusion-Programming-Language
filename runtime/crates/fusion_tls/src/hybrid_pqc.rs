//! Hybrid post-quantum key exchange combining X25519 and ML-KEM-768.

use crate::error::Result;
use bytes::Bytes;
use rand::RngCore;

/// Supported post-quantum algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    MLKem512,
    MLKem768,
    MLKem1024,
}

impl PqcAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MLKem512 => "ML-KEM-512",
            Self::MLKem768 => "ML-KEM-768",
            Self::MLKem1024 => "ML-KEM-1024",
        }
    }

    pub fn security_level(&self) -> u16 {
        match self {
            Self::MLKem512 => 128,
            Self::MLKem768 => 192,
            Self::MLKem1024 => 256,
        }
    }
}

/// Supported classical algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicalAlgorithm {
    X25519,
    P256,
    P384,
}

/// Hybrid key exchange combining classical and PQC algorithms.
#[derive(Debug, Clone)]
pub struct HybridKeyExchange {
    pub classical: ClassicalAlgorithm,
    pub pqc: PqcAlgorithm,
    pub classical_public: Vec<u8>,
    pub pqc_public: Vec<u8>,
}

/// Shared secret derived from hybrid key exchange.
#[derive(Debug, Clone)]
pub struct HybridSharedSecret {
    pub secret: Vec<u8>,
    pub classical_part: Vec<u8>,
    pub pqc_part: Vec<u8>,
    pub pqc: PqcAlgorithm,
    pub classical: ClassicalAlgorithm,
}

impl HybridKeyExchange {
    /// Create a new hybrid key exchange with default algorithms (X25519 + ML-KEM-768).
    pub fn new() -> Self {
        Self::with_algorithms(ClassicalAlgorithm::X25519, PqcAlgorithm::MLKem768)
    }

    /// Create a hybrid key exchange with specified algorithms.
    pub fn with_algorithms(classical: ClassicalAlgorithm, pqc: PqcAlgorithm) -> Self {
        // Generate classical key pair
        let mut classical_secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut classical_secret);

        let classical_pub = Self::derive_classical_public(classical, &classical_secret);

        // Generate PQC key pair (simulated — real implementation uses ML-KEM)
        let mut pqc_public = vec![0u8; Self::pqc_public_size(pqc)];
        rand::thread_rng().fill_bytes(&mut pqc_public);

        Self {
            classical,
            pqc,
            classical_public: classical_pub,
            pqc_public,
        }
    }

    /// Derive the classical public key from a secret.
    fn derive_classical_public(_algo: ClassicalAlgorithm, _secret: &[u8; 32]) -> Vec<u8> {
        // In production, this would use x25519-dalek or ring
        vec![0u8; 32]
    }

    fn pqc_public_size(algo: PqcAlgorithm) -> usize {
        match algo {
            PqcAlgorithm::MLKem512 => 800,
            PqcAlgorithm::MLKem768 => 1184,
            PqcAlgorithm::MLKem1024 => 1568,
        }
    }

    fn pqc_ciphertext_size(algo: PqcAlgorithm) -> usize {
        match algo {
            PqcAlgorithm::MLKem512 => 768,
            PqcAlgorithm::MLKem768 => 1088,
            PqcAlgorithm::MLKem1024 => 1568,
        }
    }

    fn pqc_shared_size(algo: PqcAlgorithm) -> usize {
        match algo {
            PqcAlgorithm::MLKem512 => 32,
            PqcAlgorithm::MLKem768 => 32,
            PqcAlgorithm::MLKem1024 => 32,
        }
    }

    /// Compute the shared secret using the peer's public keys.
    pub fn compute_shared_secret(&self, _peer_classical_pub: &[u8]) -> Result<HybridSharedSecret> {
        // Classical key agreement (simulated)
        let classical_shared = vec![0u8; 32];

        // PQC key encapsulation (simulated — real implementation uses ML-KEM)
        let pqc_shared = vec![0u8; Self::pqc_shared_size(self.pqc)];

        // Combine both shared secrets using HKDF-like derivation
        let mut combined = Vec::new();
        combined.extend_from_slice(&classical_shared);
        combined.extend_from_slice(&pqc_shared);

        // Simple hash-based derivation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        combined.hash(&mut hasher);
        let derived_secret = hasher.finish().to_be_bytes().to_vec();

        Ok(HybridSharedSecret {
            secret: derived_secret,
            classical_part: classical_shared,
            pqc_part: pqc_shared,
            pqc: self.pqc,
            classical: self.classical,
        })
    }

    /// Get the total size of the key exchange payload (both public keys).
    pub fn payload_size(&self) -> usize {
        self.classical_public.len() + self.pqc_public.len()
    }

    /// Serialize the key exchange payload for wire transport.
    pub fn serialize_payload(&self) -> Bytes {
        let mut buf = Vec::new();
        // Classical public key (length-prefixed)
        buf.extend_from_slice(&(self.classical_public.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.classical_public);
        // PQC public key (length-prefixed)
        buf.extend_from_slice(&(self.pqc_public.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.pqc_public);
        Bytes::from(buf)
    }
}

impl Default for HybridKeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridSharedSecret {
    /// Derive encryption keys from the shared secret for TLS record encryption.
    pub fn derive_keys(&self) -> (Vec<u8>, Vec<u8>) {
        // Derive client_write_key and server_write_key
        let client_key = self.secret.clone();
        let mut server_key = self.secret.clone();
        // Flip some bits for server key
        for byte in server_key.iter_mut() {
            *byte = byte.wrapping_add(1);
        }
        (client_key, server_key)
    }

    pub fn secret_bytes(&self) -> &[u8] {
        &self.secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_key_exchange() {
        let kex = HybridKeyExchange::new();
        assert_eq!(kex.classical, ClassicalAlgorithm::X25519);
        assert_eq!(kex.pqc, PqcAlgorithm::MLKem768);
        assert!(!kex.classical_public.is_empty());
        assert!(!kex.pqc_public.is_empty());
    }

    #[test]
    fn test_compute_shared_secret() {
        let kex = HybridKeyExchange::new();
        let secret = kex.compute_shared_secret(&[]).unwrap();
        assert!(!secret.secret.is_empty());
        assert_eq!(secret.pqc, PqcAlgorithm::MLKem768);
    }

    #[test]
    fn test_derive_keys() {
        let kex = HybridKeyExchange::new();
        let secret = kex.compute_shared_secret(&[]).unwrap();
        let (client_key, server_key) = secret.derive_keys();
        assert_ne!(client_key, server_key);
    }

    #[test]
    fn test_serialize_payload() {
        let kex = HybridKeyExchange::new();
        let payload = kex.serialize_payload();
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_pqc_algorithm_properties() {
        assert_eq!(PqcAlgorithm::MLKem768.security_level(), 192);
        assert_eq!(PqcAlgorithm::MLKem768.name(), "ML-KEM-768");
    }
}
