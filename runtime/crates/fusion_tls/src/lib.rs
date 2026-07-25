//! # Fusion TLS
//!
//! TLS 1.3 implementation with hybrid post-quantum key exchange (X25519 + ML-KEM-768).
//! Provides certificate management, secure session establishment, and PQC-aware handshake.

pub mod cert;
pub mod session;
pub mod hybrid_pqc;
pub mod error;
pub mod config;
pub mod verifier;

pub use cert::{Certificate, CertificateChain, PrivateKey};
pub use session::{TlsSession, SessionState};
pub use hybrid_pqc::{HybridKeyExchange, HybridSharedSecret, PqcAlgorithm, ClassicalAlgorithm};
pub use error::{TlsError, Result};
pub use config::{TlsConfig, TlsConfigBuilder, ProtocolVersion};
pub use verifier::{CertificateVerifier, VerificationResult};

/// TLS protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

/// Cipher suite identifiers for hybrid PQC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CipherSuite {
    /// X25519 + AES-256-GCM
    X25519Aes256Gcm,
    /// X25519 + ChaCha20-Poly1305
    X25519ChaCha20,
    /// Hybrid X25519 + ML-KEM-768 + AES-256-GCM
    HybridX25519MlKem768Aes256Gcm,
    /// Hybrid X25519 + ML-KEM-768 + ChaCha20-Poly1305
    HybridX25519MlKem768ChaCha20,
}

impl CipherSuite {
    pub fn name(&self) -> &'static str {
        match self {
            Self::X25519Aes256Gcm => "X25519_AES_256_GCM",
            Self::X25519ChaCha20 => "X25519_CHACHA20_POLY1305",
            Self::HybridX25519MlKem768Aes256Gcm => "HYBRID_X25519_ML_KEM_768_AES_256_GCM",
            Self::HybridX25519MlKem768ChaCha20 => "HYBRID_X25519_ML_KEM_768_CHACHA20",
        }
    }

    pub fn is_post_quantum(&self) -> bool {
        matches!(
            self,
            Self::HybridX25519MlKem768Aes256Gcm | Self::HybridX25519MlKem768ChaCha20
        )
    }
}

/// Session statistics for monitoring TLS connections
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub handshakes_completed: u64,
    pub handshakes_failed: u64,
    pub pqc_handshakes: u64,
    pub classical_handshakes: u64,
    pub bytes_encrypted: u64,
    pub bytes_decrypted: u64,
    pub active_sessions: usize,
}
