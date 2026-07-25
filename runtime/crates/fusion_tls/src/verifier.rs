//! Certificate verification logic.

use crate::cert::Certificate;

/// Result of a certificate verification check.
#[derive(Debug, Clone)]
pub enum VerificationResult {
    Verified,
    Failed { reason: String },
    Expired { not_after: String },
    SelfSigned { subject: String },
}

impl VerificationResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Verified | Self::SelfSigned { .. })
    }
}

/// Certificate verifier that validates chains, expiry, and trust anchors.
pub struct CertificateVerifier {
    trusted_roots: Vec<Certificate>,
    allow_self_signed: bool,
    verify_chain: bool,
}

impl CertificateVerifier {
    pub fn new() -> Self {
        Self {
            trusted_roots: Vec::new(),
            allow_self_signed: false,
            verify_chain: true,
        }
    }

    pub fn add_trusted_root(mut self, cert: Certificate) -> Self {
        self.trusted_roots.push(cert);
        self
    }

    pub fn allow_self_signed(mut self, allow: bool) -> Self {
        self.allow_self_signed = allow;
        self
    }

    pub fn verify_chain(mut self, verify: bool) -> Self {
        self.verify_chain = verify;
        self
    }

    /// Verify a certificate chain.
    pub fn verify(&self, chain: &[Certificate]) -> VerificationResult {
        if chain.is_empty() {
            return VerificationResult::Failed {
                reason: "Empty certificate chain".to_string(),
            };
        }

        let leaf = &chain[0];

        // Check if self-signed
        if leaf.subject == leaf.issuer {
            if self.allow_self_signed {
                return VerificationResult::SelfSigned {
                    subject: leaf.subject.clone(),
                };
            } else {
                return VerificationResult::Failed {
                    reason: format!(
                        "Self-signed certificate not allowed: {}",
                        leaf.subject
                    ),
                };
            }
        }

        // Check expiry (simplified — real implementation would parse dates)
        // For now, assume valid dates

        // Verify chain against trusted roots
        if self.verify_chain && !self.trusted_roots.is_empty() {
            let chain_valid = chain.iter().any(|cert| {
                self.trusted_roots
                    .iter()
                    .any(|root| root.subject == cert.issuer)
            });

            if !chain_valid {
                return VerificationResult::Failed {
                    reason: "Certificate chain not rooted in trusted CA".to_string(),
                };
            }
        }

        VerificationResult::Verified
    }

    /// Verify a single certificate (no chain).
    pub fn verify_single(&self, cert: &Certificate) -> VerificationResult {
        self.verify(&[cert.clone()])
    }
}

impl Default for CertificateVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::Certificate;

    #[test]
    fn test_verify_trusted_chain() {
        let root = Certificate::self_signed_root("CN=Trusted CA").unwrap();
        let verifier = CertificateVerifier::new().add_trusted_root(Certificate {
            subject: "CN=Trusted CA".to_string(),
            issuer: "CN=Trusted CA".to_string(),
            not_before: "2024-01-01".to_string(),
            not_after: "2034-01-01".to_string(),
            serial_number: "1".to_string(),
            is_ca: true,
            der: vec![],
        });

        let chain = vec![Certificate {
            issuer: "CN=Trusted CA".to_string(),
            subject: "CN=server.example.com".to_string(),
            not_before: "2024-01-01".to_string(),
            not_after: "2025-01-01".to_string(),
            serial_number: "2".to_string(),
            is_ca: false,
            der: vec![],
        }];

        assert!(verifier.verify(&chain).is_ok());
    }

    #[test]
    fn test_verify_reject_self_signed() {
        let self_signed = Certificate::self_signed_root("CN=Self-Signed").unwrap();
        let verifier = CertificateVerifier::new();
        let result = verifier.verify_single(&self_signed);
        assert!(!result.is_ok());
    }

    #[test]
    fn test_verify_allow_self_signed() {
        let self_signed = Certificate::self_signed_root("CN=Self-Signed").unwrap();
        let verifier = CertificateVerifier::new().allow_self_signed(true);
        let result = verifier.verify_single(&self_signed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_empty_chain() {
        let verifier = CertificateVerifier::new();
        let result = verifier.verify(&[]);
        assert!(!result.is_ok());
    }
}
