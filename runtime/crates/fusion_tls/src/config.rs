//! TLS configuration types and builder.

use crate::cert::{Certificate, CertificateChain, PrivateKey};
use crate::error::Result;
use crate::hybrid_pqc::{ClassicalAlgorithm, PqcAlgorithm};
use crate::CipherSuite;

/// TLS protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolVersion {
    Tls12,
    Tls13,
}

/// TLS server or client configuration.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub protocol_version: ProtocolVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub certificate_chain: Option<CertificateChain>,
    pub private_key: Option<PrivateKey>,
    pub ca_certificates: Vec<Certificate>,
    pub enable_pqc: bool,
    pub pqc_algorithm: PqcAlgorithm,
    pub classical_algorithm: ClassicalAlgorithm,
    pub max_fragment_size: usize,
    pub session_ticket_enabled: bool,
}

impl TlsConfig {
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }

    /// Create a server configuration with the given certificate and key.
    pub fn server(
        cert_chain: CertificateChain,
        private_key: PrivateKey,
    ) -> Result<Self> {
        Ok(Self {
            protocol_version: ProtocolVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::HybridX25519MlKem768Aes256Gcm,
                CipherSuite::X25519Aes256Gcm,
            ],
            certificate_chain: Some(cert_chain),
            private_key: Some(private_key),
            ca_certificates: Vec::new(),
            enable_pqc: true,
            pqc_algorithm: PqcAlgorithm::MLKem768,
            classical_algorithm: ClassicalAlgorithm::X25519,
            max_fragment_size: 16384,
            session_ticket_enabled: true,
        })
    }

    /// Create a client configuration that verifies server certificates.
    pub fn client() -> Self {
        Self {
            protocol_version: ProtocolVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::HybridX25519MlKem768Aes256Gcm,
                CipherSuite::X25519Aes256Gcm,
            ],
            certificate_chain: None,
            private_key: None,
            ca_certificates: Vec::new(),
            enable_pqc: true,
            pqc_algorithm: PqcAlgorithm::MLKem768,
            classical_algorithm: ClassicalAlgorithm::X25519,
            max_fragment_size: 16384,
            session_ticket_enabled: true,
        }
    }

    pub fn is_server(&self) -> bool {
        self.certificate_chain.is_some() && self.private_key.is_some()
    }

    pub fn pqc_enabled(&self) -> bool {
        self.enable_pqc
    }
}

/// Builder for TLS configuration.
#[derive(Default)]
pub struct TlsConfigBuilder {
    protocol_version: Option<ProtocolVersion>,
    cipher_suites: Vec<Certificate>,
    enable_pqc: Option<bool>,
    pqc_algorithm: Option<PqcAlgorithm>,
    max_fragment_size: Option<usize>,
    session_ticket_enabled: Option<bool>,
    ca_certificates: Vec<Certificate>,
    cert_chain: Option<CertificateChain>,
    private_key: Option<PrivateKey>,
}

impl TlsConfigBuilder {
    pub fn protocol_version(mut self, version: ProtocolVersion) -> Self {
        self.protocol_version = Some(version);
        self
    }

    pub fn enable_pqc(mut self, enable: bool) -> Self {
        self.enable_pqc = Some(enable);
        self
    }

    pub fn pqc_algorithm(mut self, algo: PqcAlgorithm) -> Self {
        self.pqc_algorithm = Some(algo);
        self
    }

    pub fn max_fragment_size(mut self, size: usize) -> Self {
        self.max_fragment_size = Some(size);
        self
    }

    pub fn session_ticket_enabled(mut self, enabled: bool) -> Self {
        self.session_ticket_enabled = Some(enabled);
        self
    }

    pub fn add_ca_certificate(mut self, cert: Certificate) -> Self {
        self.ca_certificates.push(cert);
        self
    }

    pub fn certificate_chain(mut self, chain: CertificateChain) -> Self {
        self.cert_chain = Some(chain);
        self
    }

    pub fn private_key(mut self, key: PrivateKey) -> Self {
        self.private_key = Some(key);
        self
    }

    pub fn build(self) -> TlsConfig {
        let mut cipher_suites = vec![
            CipherSuite::HybridX25519MlKem768Aes256Gcm,
            CipherSuite::X25519Aes256Gcm,
        ];

        TlsConfig {
            protocol_version: self.protocol_version.unwrap_or(ProtocolVersion::Tls13),
            cipher_suites,
            certificate_chain: self.cert_chain,
            private_key: self.private_key,
            ca_certificates: self.ca_certificates,
            enable_pqc: self.enable_pqc.unwrap_or(true),
            pqc_algorithm: self.pqc_algorithm.unwrap_or(PqcAlgorithm::MLKem768),
            classical_algorithm: ClassicalAlgorithm::X25519,
            max_fragment_size: self.max_fragment_size.unwrap_or(16384),
            session_ticket_enabled: self.session_ticket_enabled.unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cert::PrivateKey;

    #[test]
    fn test_server_config() {
        let root = Certificate::self_signed_root("CN=CA").unwrap();
        let chain = CertificateChain::new().push(root);
        let key = PrivateKey::generate_ed25519().unwrap();
        let config = TlsConfig::server(chain, key).unwrap();
        assert!(config.is_server());
        assert!(config.pqc_enabled());
    }

    #[test]
    fn test_client_config() {
        let config = TlsConfig::client();
        assert!(!config.is_server());
        assert!(config.pqc_enabled());
        assert_eq!(config.protocol_version, ProtocolVersion::Tls13);
    }

    #[test]
    fn test_builder() {
        let config = TlsConfig::builder()
            .enable_pqc(true)
            .pqc_algorithm(PqcAlgorithm::MLKem1024)
            .max_fragment_size(8192)
            .session_ticket_enabled(false)
            .build();

        assert!(config.enable_pqc);
        assert_eq!(config.pqc_algorithm, PqcAlgorithm::MLKem1024);
        assert_eq!(config.max_fragment_size, 8192);
        assert!(!config.session_ticket_enabled);
    }
}
