//! WebSocket error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid opcode: 0x{0:02X}")]
    InvalidOpcode(u8),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    #[error("Connection closed: code {code}, reason: {reason}")]
    ConnectionClosed { code: u16, reason: String },

    #[error("Handshake error: {0}")]
    Handshake(String),

    #[error("Message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, WsError>;
