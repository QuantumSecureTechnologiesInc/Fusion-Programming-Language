//! # Fusion WebSocket
//!
//! WebSocket server and client with frame encoding/decoding, ping/pong keepalive,
//! binary and text message support, and close handshake.

pub mod frame;
pub mod message;
pub mod opcodes;
pub mod server;
pub mod client;
pub mod error;
pub mod mask;
pub mod extensions;

pub use frame::{Frame, FrameHeader};
pub use message::{Message, MessageType};
pub use opcodes::Opcode;
pub use server::WebSocketServer;
pub use client::WebSocketClient;
pub use error::{WsError, Result};

use bytes::Bytes;

/// WebSocket protocol version
pub const WEBSOCKET_VERSION: &str = "13";

/// Magic GUID for WebSocket handshake (RFC 6455)
pub const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
