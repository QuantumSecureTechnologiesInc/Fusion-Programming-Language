//! WebSocket client — connects to servers and manages the connection.

use crate::error::{Result, WsError};
use crate::frame::Frame;
use crate::message::Message;
use crate::opcodes::Opcode;
use crate::server::WebSocketServer;
use bytes::BytesMut;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// WebSocket client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub connect_timeout: Duration,
    pub max_message_size: usize,
    pub ping_interval: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            max_message_size: 64 * 1024,
            ping_interval: Duration::from_secs(30),
        }
    }
}

/// A WebSocket client connection.
pub struct WebSocketClient {
    config: ClientConfig,
    stream: Option<TcpStream>,
    read_buffer: BytesMut,
    connected: bool,
    messages_sent: u64,
    messages_received: u64,
}

impl WebSocketClient {
    pub fn new() -> Self {
        Self::with_config(ClientConfig::default())
    }

    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            config,
            stream: None,
            read_buffer: BytesMut::with_capacity(8192),
            connected: false,
            messages_sent: 0,
            messages_received: 0,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn messages_sent(&self) -> u64 {
        self.messages_sent
    }

    pub fn messages_received(&self) -> u64 {
        self.messages_received
    }

    /// Connect to a WebSocket server at the given address.
    pub async fn connect(&mut self, address: &str) -> Result<()> {
        let stream = tokio::time::timeout(
            self.config.connect_timeout,
            TcpStream::connect(address),
        )
        .await
        .map_err(|_| WsError::Timeout)?
        .map_err(WsError::Io)?;

        self.stream = Some(stream);
        self.connected = true;
        Ok(())
    }

    /// Perform the client-side WebSocket handshake.
    pub async fn handshake(&mut self, host: &str, path: &str) -> Result<()> {
        use base64::Engine;
        use rand::RngCore;
        use sha1::{Digest, Sha1};

        // Generate random key
        let mut key_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        let key = base64::engine::general_purpose::STANDARD.encode(key_bytes);

        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Version: 13\r\n\
             Sec-WebSocket-Key: {}\r\n\
             \r\n",
            path, host, key
        );

        if let Some(ref mut stream) = self.stream {
            stream.write_all(request.as_bytes()).await?;

            // Read response
            let mut response_buf = BytesMut::with_capacity(4096);
            loop {
                let n = stream.read_buf(&mut response_buf).await?;
                if n == 0 {
                    return Err(WsError::Handshake("Connection closed during handshake".to_string()));
                }

                if let Some(_) = response_buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break;
                }
            }

            let response = std::str::from_utf8(&response_buf)
                .map_err(|_| WsError::Handshake("Invalid UTF-8 in response".to_string()))?;

            if !response.contains("101") {
                return Err(WsError::Handshake(format!(
                    "Server did not accept upgrade: {}",
                    response.lines().next().unwrap_or("unknown")
                )));
            }

            // Verify accept key
            let mut hasher = Sha1::new();
            hasher.update(key.as_bytes());
            hasher.update(crate::WEBSOCKET_GUID.as_bytes());
            let expected_accept =
                base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

            if !response.contains(&expected_accept) {
                return Err(WsError::Handshake("Invalid Sec-WebSocket-Accept".to_string()));
            }

            Ok(())
        } else {
            Err(WsError::Handshake("Not connected".to_string()))
        }
    }

    /// Read the next message from the server.
    pub async fn read_message(&mut self) -> Result<Option<Message>> {
        loop {
            match Frame::parse(&mut self.read_buffer) {
                Ok(Some(frame)) => {
                    self.messages_received += 1;
                    let msg = match frame.opcode() {
                        Opcode::Text => {
                            let text = frame.payload_text()?.to_string();
                            Message::Text(text)
                        }
                        Opcode::Binary => {
                            Message::Binary(frame.payload_bytes().to_vec())
                        }
                        Opcode::Ping => {
                            // Auto-respond with pong
                            self.send_frame(&Frame::pong(frame.payload_bytes())).await?;
                            Message::Ping(frame.payload_bytes().to_vec())
                        }
                        Opcode::Pong => {
                            Message::Pong(frame.payload_bytes().to_vec())
                        }
                        Opcode::Close => {
                            let payload = frame.payload_bytes();
                            let (code, reason) = if payload.len() >= 2 {
                                let code = u16::from_be_bytes([payload[0], payload[1]]);
                                let reason = String::from_utf8_lossy(&payload[2..]).to_string();
                                (code, reason)
                            } else {
                                (1000, "normal".to_string())
                            };
                            // Send close response
                            self.send_frame(&Frame::close(code, &reason)).await?;
                            self.connected = false;
                            return Ok(Some(Message::Close(Some(
                                crate::message::CloseFrame { code, reason },
                            ))));
                        }
                        Opcode::Continuation => {
                            return Err(WsError::Protocol(
                                "Fragmented messages not supported".to_string(),
                            ));
                        }
                    };
                    return Ok(Some(msg));
                }
                Ok(None) => {
                    if let Some(ref mut stream) = self.stream {
                        let n = stream.read_buf(&mut self.read_buffer).await?;
                        if n == 0 {
                            self.connected = false;
                            return Ok(None);
                        }
                    } else {
                        return Err(WsError::Handshake("Not connected".to_string()));
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Send a message to the server.
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

    /// Send a text message.
    pub async fn send_text(&mut self, text: &str) -> Result<()> {
        self.send_message(&Message::text(text)).await
    }

    /// Send a binary message.
    pub async fn send_binary(&mut self, data: &[u8]) -> Result<()> {
        self.send_message(&Message::binary(data)).await
    }

    /// Send a frame.
    async fn send_frame(&mut self, frame: &Frame) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            let bytes = frame.serialize();
            stream.write_all(&bytes).await?;
            stream.flush().await?;
            self.messages_sent += 1;
        }
        Ok(())
    }

    /// Close the connection gracefully.
    pub async fn close(&mut self) -> Result<()> {
        if self.connected {
            let frame = Frame::close(1000, "normal");
            self.send_frame(&frame).await?;
            self.connected = false;
        }
        Ok(())
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WebSocketClient::new();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_client_with_config() {
        let config = ClientConfig {
            connect_timeout: Duration::from_secs(5),
            max_message_size: 1024,
            ping_interval: Duration::from_secs(10),
        };
        let client = WebSocketClient::with_config(config);
        assert_eq!(client.config.max_message_size, 1024);
    }
}
