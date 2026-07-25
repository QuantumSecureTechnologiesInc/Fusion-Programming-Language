//! gRPC status type carrying code, message, and optional details.

use crate::code::GrpcCode;
use bytes::Bytes;
use std::fmt;

/// A gRPC status with code, message, and binary details.
#[derive(Debug, Clone)]
pub struct GrpcStatus {
    pub code: GrpcCode,
    pub message: String,
    pub details: Bytes,
}

impl GrpcStatus {
    pub fn ok() -> Self {
        Self {
            code: GrpcCode::Ok,
            message: String::new(),
            details: Bytes::new(),
        }
    }

    pub fn error(code: GrpcCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Bytes::new(),
        }
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::Cancelled, message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::InvalidArgument, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::NotFound, message)
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::Unimplemented, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::Internal, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::Unavailable, message)
    }

    pub fn unauthenticated(message: impl Into<String>) -> Self {
        Self::error(GrpcCode::Unauthenticated, message)
    }

    pub fn is_ok(&self) -> bool {
        self.code.is_ok()
    }

    pub fn with_details(mut self, details: Bytes) -> Self {
        self.details = details;
        self
    }

    /// Encode status as HTTP/2 trailers for wire transport.
    pub fn encode_trailers(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("grpc-status: {}\r\n", self.code as u32));
        if !self.message.is_empty() {
            let encoded = self.message.replace('\n', "%0A");
            output.push_str(&format!("grpc-message: {}\r\n", encoded));
        }
        output.push_str("\r\n");
        output
    }

    /// Decode status from HTTP/2 trailer key-value pairs.
    pub fn from_trailers(trailers: &std::collections::HashMap<String, String>) -> Self {
        let code = trailers
            .get("grpc-status")
            .and_then(|v| v.parse::<u32>().ok())
            .and_then(GrpcCode::from_u32)
            .unwrap_or(GrpcCode::Unknown);

        let message = trailers
            .get("grpc-message")
            .map(|v| v.replace("%0A", "\n"))
            .unwrap_or_default();

        Self {
            code,
            message,
            details: Bytes::new(),
        }
    }
}

impl fmt::Display for GrpcStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gRPC status {}: {}", self.code, self.message)
    }
}

impl std::error::Error for GrpcStatus {}

impl Default for GrpcStatus {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_ok() {
        let s = GrpcStatus::ok();
        assert!(s.is_ok());
    }

    #[test]
    fn test_status_error() {
        let s = GrpcStatus::not_found("user 42 not found");
        assert_eq!(s.code, GrpcCode::NotFound);
        assert_eq!(s.message, "user 42 not found");
    }

    #[test]
    fn test_encode_decode_trailers() {
        let status = GrpcStatus::error(GrpcCode::InvalidArgument, "bad value\nreally bad");
        let encoded = status.encode_trailers();

        let mut trailers = std::collections::HashMap::new();
        for line in encoded.lines() {
            if let Some((k, v)) = line.split_once(": ") {
                trailers.insert(k.to_string(), v.trim().to_string());
            }
        }

        let decoded = GrpcStatus::from_trailers(&trailers);
        assert_eq!(decoded.code, GrpcCode::InvalidArgument);
        assert_eq!(decoded.message, "bad value\nreally bad");
    }
}
