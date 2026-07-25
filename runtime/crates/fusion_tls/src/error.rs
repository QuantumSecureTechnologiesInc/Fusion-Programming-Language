//! Error types for the TLS module.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("Private key error: {0}")]
    PrivateKey(String),

    #[error("Handshake error: {0}")]
    Handshake(String),

    #[error("Post-quantum error: {0}")]
    PostQuantum(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Rustls error: {0}")]
    Rustls(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, TlsError>;
