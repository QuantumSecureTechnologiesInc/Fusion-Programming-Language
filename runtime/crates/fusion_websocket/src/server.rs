//! WebSocket server — accepts upgrades and manages connections.

use crate::error::{Result, WsError};
use crate::frame::Frame;
use crate::message::Message;
use crate::opcodes::Opcode;
use bytes::{Bytes, BytesMut};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// WebSocket server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub max_message_size: usize,
    pub ping_interval: Duration,
    pub pong_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:9001".to_string(),
            max_message_size: 64 * 1024,
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
        }
    }
}

/// Information about an active WebSocket connection.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: u64,
    pub peer_addr: Option<String>,
    pub connected_at: Instant,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// WebSocket server that listens for connections.
pub struct WebSocketServer {
    config: ServerConfig,
    connections: Arc<parking_lot::Mutex<HashMap<u64, ConnectionInfo>>>,
    next_id: Arc<std::sync::atomic::AtomicU64>,
}

impl WebSocketServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            connections: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    pub fn with_address(address: impl Into<String>) -> Self {
        Self::new(ServerConfig {
            address: address.into(),
            ..ServerConfig::default()
        })
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub fn active_connections(&self) -> usize {
        self.connections.lock().len()
    }

    /// Create a handshake response for a WebSocket upgrade request.
    pub fn create_handshake_response(key: &str) -> Result<String> {
        use base64::Engine;
        use sha1::{Digest, Sha1};

        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(crate::WEBSOCKET_GUID.as_bytes());
        let accept = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

        Ok(format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             \r\n",
            accept
        ))
    }

    /// Validate a WebSocket upgrade request headers.
    pub fn validate_upgrade_request(headers: &HashMap<String, String>) -> Result<String> {
        let upgrade = headers
            .get("upgrade")
            .map(|v| v.to_lowercase())
            .unwrap_or_default();
        if upgrade != "websocket" {
            return Err(WsError::Handshake("Missing or invalid Upgrade header".to_string()));
        }

        let connection = headers
            .get("connection")
            .map(|v| v.to_lowercase())
            .unwrap_or_default();
        if !connection.contains("upgrade") {
            return Err(WsError::Handshake("Missing or invalid Connection header".to_string()));
        }

        let version = headers.get("sec-websocket-version").map(|v| v.as_str()).unwrap_or("");
        if version != "13" {
            return Err(WsError::Handshake("Unsupported WebSocket version".to_string()));
        }

        let key = headers
            .get("sec-websocket-key")
            .ok_or_else(|| WsError::Handshake("Missing Sec-WebSocket-Key".to_string()))?;

        Ok(key.clone())
    }

    /// Build the server handshake response from a client key.
    pub fn build_handshake_response(client_key: &str) -> Result<String> {
        Self::create_handshake_response(client_key)
    }

    /// Parse a raw HTTP upgrade request into headers.
    pub fn parse_upgrade_request(raw: &str) -> Result<HashMap<String, String>> {
        let mut headers = HashMap::new();
        for line in raw.lines() {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }
        Ok(headers)
    }
}

/// A connected WebSocket session on the server side.
pub struct WebSocketConnection {
    id: u64,
    stream: TcpStream,
    read_buffer: BytesMut,
    max_message_size: usize,
    last_pong: Instant,
    messages_sent: u64,
    messages_received: u64,
}

impl WebSocketConnection {
    pub fn new(id: u64, stream: TcpStream, max_message_size: usize) -> Self {
        Self {
            id,
            stream,
            read_buffer: BytesMut::with_capacity(8192),
            max_message_size,
            last_pong: Instant::now(),
            messages_sent: 0,
            messages_received: 0,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn messages_sent(&self) -> u64 {
        self.messages_sent
    }

    pub fn messages_received(&self) -> u64 {
        self.messages_received
    }

    pub fn pong_age(&self) -> Duration {
        self.last_pong.elapsed()
    }

    /// Read and parse the next frame from the connection.
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            match Frame::parse(&mut self.read_buffer) {
                Ok(Some(frame)) => {
                    if frame.payload_bytes().len() > self.max_message_size {
                        return Err(WsError::MessageTooLarge {
                            size: frame.payload_bytes().len(),
                            max: self.max_message_size,
                        });
                    }
                    return Ok(Some(frame));
                }
                Ok(None) => {
                    // Need more data
                    let n = self.stream.read_buf(&mut self.read_buffer).await?;
                    if n == 0 {
                        return Ok(None); // Connection closed
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Send a frame to the client.
    pub async fn send_frame(&mut self, frame: &Frame) -> Result<()> {
        let bytes = frame.serialize();
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;
        self.messages_sent += 1;
        Ok(())
    }

    /// Send a message.
    pub async fn send_message(&mut self, msg: &Message) -> Result<()> {
        let frame = match msg {
            Message::Text(text) => Frame::text(text),
            Message::Binary(data) => Frame::binary(data),
            Message::Ping(data) => Frame::ping(data),
            Message::Pong(data) => Frame::pong(data),
            Message::Close(Some(close)) => Frame::close(close.code, &close.reason),
            Message::Close(None) => Frame::close(1000, ""),
        };
        self.send_frame(&frame).await
    }

    /// Send a ping and update tracking.
    pub async fn send_ping(&mut self, data: &[u8]) -> Result<()> {
        self.send_frame(&Frame::ping(data)).await
    }

    /// Record a pong received from the client.
    pub fn record_pong(&mut self) {
        self.last_pong = Instant::now();
    }

    pub fn peer_addr(&self) -> Option<String> {
        self.stream.peer_addr().ok().map(|a| a.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_upgrade_request_valid() {
        let mut headers = HashMap::new();
        headers.insert("upgrade".to_string(), "websocket".to_string());
        headers.insert("connection".to_string(), "Upgrade".to_string());
        headers.insert("sec-websocket-version".to_string(), "13".to_string());
        headers.insert(
            "sec-websocket-key".to_string(),
            "dGhlIHNhbXBsZSBub25jZQ==".to_string(),
        );

        let key = WebSocketServer::validate_upgrade_request(&headers).unwrap();
        assert_eq!(key, "dGhlIHNhbXBsZSBub25jZQ==");
    }

    #[test]
    fn test_validate_upgrade_request_missing_key() {
        let mut headers = HashMap::new();
        headers.insert("upgrade".to_string(), "websocket".to_string());
        headers.insert("connection".to_string(), "Upgrade".to_string());
        headers.insert("sec-websocket-version".to_string(), "13".to_string());

        assert!(WebSocketServer::validate_upgrade_request(&headers).is_err());
    }

    #[test]
    fn test_build_handshake_response() {
        let resp = WebSocketServer::build_handshake_response("dGhlIHNhbXBsZSBub25jZQ==").unwrap();
        assert!(resp.contains("101 Switching Protocols"));
        assert!(resp.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
    }

    #[test]
    fn test_parse_upgrade_request() {
        let raw = "GET /ws HTTP/1.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        let headers = WebSocketServer::parse_upgrade_request(raw).unwrap();
        assert_eq!(headers.get("upgrade").unwrap(), "websocket");
    }
}
