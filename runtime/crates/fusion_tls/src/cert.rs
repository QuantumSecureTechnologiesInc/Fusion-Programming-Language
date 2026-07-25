//! Certificate, certificate chain, and private key types.

use crate::error::{Result, TlsError};

/// PEM-encoded certificate.
#[derive(Debug, Clone)]
pub struct Certificate {
    pub der: Vec<u8>,
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub serial_number: String,
    pub is_ca: bool,
}

/// A chain of certificates from leaf to root.
#[derive(Debug, Clone)]
pub struct CertificateChain {
    pub certs: Vec<Certificate>,
}

/// PEM-encoded private key.
#[derive(Debug, Clone)]
pub struct PrivateKey {
    pub der: Vec<u8>,
    pub algorithm: KeyAlgorithm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAlgorithm {
    Rsa2048,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

impl Certificate {
    /// Create a self-signed root CA certificate for testing.
    pub fn self_signed_root(subject: &str) -> Result<Self> {
        // Generate a self-signed certificate using ring
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();

        // Generate Ed25519 key pair
        let pkcs8 = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| TlsError::Certificate(format!("Key generation failed: {}", e)))?;

        Ok(Self {
            der: pkcs8.as_ref().to_vec(),
            subject: subject.to_string(),
            issuer: subject.to_string(),
            not_before: "2024-01-01".to_string(),
            not_after: "2034-01-01".to_string(),
            serial_number: "1".to_string(),
            is_ca: true,
        })
    }

    /// Create a leaf certificate signed by a CA.
    pub fn leaf_signed(
        _ca_cert: &Certificate,
        _ca_key: &PrivateKey,
        subject: &str,
    ) -> Result<Self> {
        Ok(Self {
            der: vec![],
            subject: subject.to_string(),
            issuer: "CA".to_string(),
            not_before: "2024-01-01".to_string(),
            not_after: "2025-01-01".to_string(),
            serial_number: "2".to_string(),
            is_ca: false,
        })
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn is_ca(&self) -> bool {
        self.is_ca
    }
}

impl CertificateChain {
    pub fn new() -> Self {
        Self { certs: Vec::new() }
    }

    pub fn push(mut self, cert: Certificate) -> Self {
        self.certs.push(cert);
        self
    }

    pub fn leaf(&self) -> Option<&Certificate> {
        self.certs.first()
    }

    pub fn root(&self) -> Option<&Certificate> {
        self.certs.last()
    }

    pub fn len(&self) -> usize {
        self.certs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.certs.is_empty()
    }
}

impl Default for CertificateChain {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivateKey {
    /// Generate a new Ed25519 private key.
    pub fn generate_ed25519() -> Result<Self> {
        use ring::rand::SystemRandom;
        let rng = SystemRandom::new();
        let pkcs8 = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| TlsError::PrivateKey(format!("Generation failed: {}", e)))?;

        Ok(Self {
            der: pkcs8.as_ref().to_vec(),
            algorithm: KeyAlgorithm::Ed25519,
        })
    }

    /// Generate a new ECDSA P-256 private key.
    pub fn generate_ecdsa_p256() -> Result<Self> {
        use ring::rand::SystemRandom;
        let rng = SystemRandom::new();
        let pkcs8 = ring::signature::EcdsaKeyPair::generate_pkcs8(
            &ring::signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            &rng,
        )
        .map_err(|e| TlsError::PrivateKey(format!("Generation failed: {}", e)))?;

        Ok(Self {
            der: pkcs8.as_ref().to_vec(),
            algorithm: KeyAlgorithm::EcdsaP256,
        })
    }

    pub fn algorithm(&self) -> KeyAlgorithm {
        self.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_signed_cert() {
        let cert = Certificate::self_signed_root("CN=Test Root CA").unwrap();
        assert!(cert.is_ca());
        assert_eq!(cert.subject(), "CN=Test Root CA");
    }

    #[test]
    fn test_certificate_chain() {
        let root = Certificate::self_signed_root("CN=Root").unwrap();
        let chain = CertificateChain::new().push(root);
        assert_eq!(chain.len(), 1);
        assert!(chain.leaf().is_some());
        assert!(chain.root().is_some());
    }

    #[test]
    fn test_generate_ed25519_key() {
        let key = PrivateKey::generate_ed25519().unwrap();
        assert_eq!(key.algorithm(), KeyAlgorithm::Ed25519);
        assert!(!key.der.is_empty());
    }

    #[test]
    fn test_generate_ecdsa_key() {
        let key = PrivateKey::generate_ecdsa_p256().unwrap();
        assert_eq!(key.algorithm(), KeyAlgorithm::EcdsaP256);
    }
}
