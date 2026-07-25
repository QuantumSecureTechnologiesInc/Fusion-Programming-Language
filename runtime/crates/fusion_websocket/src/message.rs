//! WebSocket message abstraction over frames.

use crate::frame::Frame;
use crate::opcodes::Opcode;
use bytes::Bytes;

/// A high-level WebSocket message.
#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

/// Close frame payload.
#[derive(Debug, Clone)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

/// Message type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

impl Message {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn binary(data: impl Into<Vec<u8>>) -> Self {
        Self::Binary(data.into())
    }

    pub fn ping(data: impl Into<Vec<u8>>) -> Self {
        Self::Ping(data.into())
    }

    pub fn pong(data: impl Into<Vec<u8>>) -> Self {
        Self::Pong(data.into())
    }

    pub fn close(code: u16, reason: &str) -> Self {
        Self::Close(Some(CloseFrame {
            code,
            reason: reason.to_string(),
        }))
    }

    pub fn close_empty() -> Self {
        Self::Close(None)
    }

    pub fn message_type(&self) -> MessageType {
        match self {
            Self::Text(_) => MessageType::Text,
            Self::Binary(_) => MessageType::Binary,
            Self::Ping(_) => MessageType::Ping,
            Self::Pong(_) => MessageType::Pong,
            Self::Close(_) => MessageType::Close,
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    pub fn is_close(&self) -> bool {
        matches!(self, Self::Close(_))
    }

    /// Get the text content if this is a Text message.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the binary content if this is a Binary message.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Self::Binary(b) => Some(b),
            _ => None,
        }
    }
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<Vec<u8>> for Message {
    fn from(v: Vec<u8>) -> Self {
        Self::Binary(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message() {
        let msg = Message::text("hello");
        assert!(msg.is_text());
        assert_eq!(msg.as_text().unwrap(), "hello");
    }

    #[test]
    fn test_binary_message() {
        let msg = Message::binary(vec![1, 2, 3]);
        assert!(msg.is_binary());
        assert_eq!(msg.as_binary().unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn test_close_message() {
        let msg = Message::close(1000, "normal");
        assert!(msg.is_close());
        assert_eq!(msg.message_type(), MessageType::Close);
    }

    #[test]
    fn test_from_string() {
        let msg: Message = "hello".into();
        assert!(msg.is_text());
    }

    #[test]
    fn test_from_vec() {
        let msg: Message = vec![1, 2, 3].into();
        assert!(msg.is_binary());
    }
}
