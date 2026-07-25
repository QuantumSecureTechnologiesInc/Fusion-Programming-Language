//! WebSocket opcode definitions per RFC 6455.

use crate::error::{Result, WsError};

/// WebSocket frame opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    /// Continuation frame
    Continuation = 0x0,
    /// Text frame
    Text = 0x1,
    /// Binary frame
    Binary = 0x2,
    /// Connection close
    Close = 0x8,
    /// Ping
    Ping = 0x9,
    /// Pong
    Pong = 0xA,
}

impl Opcode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x0 => Some(Self::Continuation),
            0x1 => Some(Self::Text),
            0x2 => Some(Self::Binary),
            0x8 => Some(Self::Close),
            0x9 => Some(Self::Ping),
            0xA => Some(Self::Pong),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Continuation => "continuation",
            Self::Text => "text",
            Self::Binary => "binary",
            Self::Close => "close",
            Self::Ping => "ping",
            Self::Pong => "pong",
        }
    }

    pub fn is_control(&self) -> bool {
        matches!(self, Self::Close | Self::Ping | Self::Pong)
    }

    pub fn is_data(&self) -> bool {
        matches!(self, Self::Continuation | Self::Text | Self::Binary)
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(Opcode::from_u8(0x0), Some(Opcode::Continuation));
        assert_eq!(Opcode::from_u8(0x1), Some(Opcode::Text));
        assert_eq!(Opcode::from_u8(0x2), Some(Opcode::Binary));
        assert_eq!(Opcode::from_u8(0x8), Some(Opcode::Close));
        assert_eq!(Opcode::from_u8(0x9), Some(Opcode::Ping));
        assert_eq!(Opcode::from_u8(0xA), Some(Opcode::Pong));
        assert_eq!(Opcode::from_u8(0x5), None);
    }

    #[test]
    fn test_opcode_properties() {
        assert!(Opcode::Close.is_control());
        assert!(Opcode::Ping.is_control());
        assert!(Opcode::Pong.is_control());
        assert!(!Opcode::Text.is_control());

        assert!(Opcode::Text.is_data());
        assert!(Opcode::Binary.is_data());
        assert!(Opcode::Continuation.is_data());
        assert!(!Opcode::Close.is_data());
    }
}
