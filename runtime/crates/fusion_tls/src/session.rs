//! TLS session management — establishment, state tracking, and statistics.

use crate::error::Result;
use crate::hybrid_pqc::{HybridKeyExchange, HybridSharedSecret};
use crate::CipherSuite;
use std::time::Instant;

/// TLS session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Initial state before handshake.
    Init,
    /// Client hello sent, waiting for server hello.
    ClientHelloSent,
    /// Server hello received, key exchange in progress.
    KeyExchange,
    /// Handshake complete, session is established.
    Established,
    /// Session is closing.
    Closing,
    /// Session has been terminated.
    Closed,
}

/// A TLS session representing an established or in-progress connection.
pub struct TlsSession {
    id: u64,
    state: SessionState,
    cipher_suite: CipherSuite,
    started_at: Instant,
    bytes_encrypted: u64,
    bytes_decrypted: u64,
    pqc_enabled: bool,
    shared_secret: Option<HybridSharedSecret>,
}

impl TlsSession {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            state: SessionState::Init,
            cipher_suite: CipherSuite::X25519Aes256Gcm,
            started_at: Instant::now(),
            bytes_encrypted: 0,
            bytes_decrypted: 0,
            pqc_enabled: false,
            shared_secret: None,
        }
    }

    pub fn with_pqc(mut self, cipher_suite: CipherSuite) -> Self {
        self.pqc_enabled = true;
        self.cipher_suite = cipher_suite;
        self
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn cipher_suite(&self) -> CipherSuite {
        self.cipher_suite
    }

    pub fn is_established(&self) -> bool {
        self.state == SessionState::Established
    }

    pub fn is_pqc(&self) -> bool {
        self.pqc_enabled
    }

    /// Perform the key exchange phase using hybrid PQC.
    pub fn perform_key_exchange(&mut self) -> Result<HybridKeyExchange> {
        self.state = SessionState::KeyExchange;

        let kex = HybridKeyExchange::new();
        let shared_secret = kex.compute_shared_secret(&[])?;

        self.shared_secret = Some(shared_secret);
        self.state = SessionState::Established;

        Ok(kex)
    }

    /// Mark the session as established with the given parameters.
    pub fn establish(&mut self, cipher_suite: CipherSuite) {
        self.state = SessionState::Established;
        self.cipher_suite = cipher_suite;
    }

    /// Record encrypted bytes sent.
    pub fn record_encrypted(&mut self, bytes: u64) {
        self.bytes_encrypted += bytes;
    }

    /// Record decrypted bytes received.
    pub fn record_decrypted(&mut self, bytes: u64) {
        self.bytes_decrypted += bytes;
    }

    /// Get session statistics.
    pub fn stats(&self) -> SessionStats {
        SessionStats {
            session_id: self.id,
            state: self.state,
            cipher_suite: self.cipher_suite,
            uptime: self.started_at.elapsed(),
            bytes_encrypted: self.bytes_encrypted,
            bytes_decrypted: self.bytes_decrypted,
            pqc_enabled: self.pqc_enabled,
        }
    }

    /// Close the session.
    pub fn close(&mut self) {
        self.state = SessionState::Closed;
    }
}

/// Snapshot of session statistics.
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub session_id: u64,
    pub state: SessionState,
    pub cipher_suite: CipherSuite,
    pub uptime: std::time::Duration,
    pub bytes_encrypted: u64,
    pub bytes_decrypted: u64,
    pub pqc_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = TlsSession::new(1);
        assert_eq!(session.state(), SessionState::Init);
        assert!(!session.is_established());
    }

    #[test]
    fn test_session_key_exchange() {
        let mut session = TlsSession::new(1).with_pqc(CipherSuite::HybridX25519MlKem768Aes256Gcm);
        let kex = session.perform_key_exchange().unwrap();
        assert!(session.is_established());
        assert!(session.is_pqc());
    }

    #[test]
    fn test_session_stats() {
        let mut session = TlsSession::new(42);
        session.establish(CipherSuite::X25519Aes256Gcm);
        session.record_encrypted(1024);
        session.record_decrypted(2048);

        let stats = session.stats();
        assert_eq!(stats.session_id, 42);
        assert_eq!(stats.bytes_encrypted, 1024);
        assert_eq!(stats.bytes_decrypted, 2048);
    }

    #[test]
    fn test_session_close() {
        let mut session = TlsSession::new(1);
        session.close();
        assert_eq!(session.state(), SessionState::Closed);
    }
}
