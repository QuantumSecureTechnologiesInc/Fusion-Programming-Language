//! gRPC client — connection management and request dispatch.

use crate::code::GrpcCode;
use crate::codec;
use crate::metadata::{MetadataKey, MetadataMap, MetadataValue};
use crate::status::GrpcStatus;
use bytes::Bytes;
use std::collections::HashMap;

/// Connection state for a gRPC client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Ready,
    TransientFailure,
    Shutdown,
}

/// Configuration for a gRPC client connection.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub address: String,
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub keepalive_interval_ms: u64,
    pub max_message_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            address: "http://localhost:50051".to_string(),
            connect_timeout_ms: 10_000,
            request_timeout_ms: 30_000,
            keepalive_interval_ms: 30_000,
            max_message_size: 4 * 1024 * 1024,
        }
    }
}

/// A gRPC client for making RPC calls to a remote server.
pub struct GrpcClient {
    config: ClientConfig,
    state: ConnectionState,
    metadata: MetadataMap,
}

impl GrpcClient {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            config: ClientConfig {
                address: address.into(),
                ..ClientConfig::default()
            },
            state: ConnectionState::Disconnected,
            metadata: MetadataMap::new(),
        }
    }

    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            config,
            state: ConnectionState::Disconnected,
            metadata: MetadataMap::new(),
        }
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn set_metadata(&mut self, key: impl Into<MetadataKey>, value: impl Into<MetadataValue>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Build a unary RPC request for the given service/method.
    pub fn build_unary_request(
        &self,
        service: &str,
        method: &str,
        body: &[u8],
    ) -> UnaryRequest {
        let path = format!("/{}/{}/{}", service, "v1", method);
        let mut headers = MetadataMap::new();
        headers.insert(
            MetadataKey::new("content-type"),
            MetadataValue::from_str("application/grpc"),
        );
        headers.insert(
            MetadataKey::new("te"),
            MetadataValue::from_str("trailers"),
        );
        // Merge client-level metadata
        for (k, v) in self.metadata.iter() {
            headers.insert(k.clone(), v.clone());
        }

        UnaryRequest {
            path,
            headers,
            body: Bytes::copy_from_slice(body),
        }
    }

    /// Parse a gRPC response from raw bytes (header + body + trailers).
    pub fn parse_response(raw: &[u8]) -> Result<GrpcResponse, GrpcStatus> {
        // gRPC framing: [5-byte header][payload][trailers]\r\n\r\n
        // But trailers-only responses have just [trailers]\r\n\r\n with no header.
        let (body, body_end) = if raw.len() > codec::HEADER_SIZE {
            let body_len = u32::from_be_bytes([raw[1], raw[2], raw[3], raw[4]]) as usize;
            let body_start = codec::HEADER_SIZE;
            let body_end = body_start + body_len;
            if body_end <= raw.len() {
                (Bytes::copy_from_slice(&raw[body_start..body_end]), body_end)
            } else {
                // Might be a trailers-only response with no valid header
                (Bytes::new(), 0)
            }
        } else {
            (Bytes::new(), 0)
        };

        // Parse trailers from the remainder after header + body
        let trailer_text = std::str::from_utf8(&raw[body_end..]).unwrap_or("");

        let mut trailers = HashMap::new();
        for line in trailer_text.lines() {
            if let Some((k, v)) = line.split_once(':') {
                let k = k.trim().to_string();
                if !k.is_empty() {
                    trailers.insert(k, v.trim().to_string());
                }
            }
        }

        let status = GrpcStatus::from_trailers(&trailers);

        Ok(GrpcResponse {
            status,
            metadata: MetadataMap::new(),
            body,
        })
    }
}

impl Default for GrpcClient {
    fn default() -> Self {
        Self::new("http://localhost:50051")
    }
}

/// A prepared unary RPC request.
#[derive(Debug)]
pub struct UnaryRequest {
    pub path: String,
    pub headers: MetadataMap,
    pub body: Bytes,
}

/// A parsed gRPC response.
#[derive(Debug)]
pub struct GrpcResponse {
    pub status: GrpcStatus,
    pub metadata: MetadataMap,
    pub body: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GrpcClient::new("http://localhost:50051");
        assert_eq!(client.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_build_unary_request() {
        let client = GrpcClient::new("http://localhost:50051");
        let req = client.build_unary_request("UserService", "GetUser", b"request");
        assert_eq!(req.path, "/UserService/v1/GetUser");
        assert_eq!(req.body.as_ref(), b"request");
    }

    #[test]
    fn test_parse_ok_response() {
        // Build a valid gRPC response: 5-byte header + body + trailers
        let body = b"response data";
        let mut raw = Vec::new();
        raw.push(0u8); // not compressed
        raw.extend_from_slice(&(body.len() as u32).to_be_bytes());
        raw.extend_from_slice(body);
        raw.extend_from_slice(b"\r\ngrpc-status: 0\r\n\r\n");

        let resp = GrpcClient::parse_response(&raw).unwrap();
        assert!(resp.status.is_ok());
        assert_eq!(resp.body.as_ref(), b"response data");
    }

    #[test]
    fn test_parse_error_response() {
        let raw = b"\r\ngrpc-status: 5\r\ngrpc-message: not found\r\n\r\n";
        let resp = GrpcClient::parse_response(raw).unwrap();
        assert_eq!(resp.status.code, GrpcCode::NotFound);
        assert_eq!(resp.status.message, "not found");
    }
}
