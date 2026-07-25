//! # Fusion gRPC
//!
//! gRPC server and client framework with unary and streaming RPCs,
//! protobuf-style service definitions, status codes, and error handling.

pub mod code;
pub mod status;
pub mod codec;
pub mod metadata;
pub mod service;
pub mod server;
pub mod client;
pub mod codec_stream;

pub use code::GrpcCode;
pub use status::GrpcStatus;
pub use metadata::MetadataMap;
pub use service::{ServiceDefinition, MethodDescriptor, ServiceRegistrar};
pub use server::{GrpcServer, GrpcService};
pub use client::GrpcClient;
pub use codec::Codec;

use bytes::Bytes;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("gRPC status {code}: {message}")]
    Status { code: GrpcCode, message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Codec error: {0}")]
    Codec(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout")]
    Timeout,

    #[error("Stream closed")]
    StreamClosed,

    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Hyper error: {0}")]
    Hyper(String),
}

pub type Result<T> = std::result::Result<T, GrpcError>;

impl From<GrpcStatus> for GrpcError {
    fn from(status: GrpcStatus) -> Self {
        GrpcError::Status {
            code: status.code,
            message: status.message,
        }
    }
}

/// A gRPC message wrapper carrying serialized protobuf bytes.
#[derive(Debug, Clone, Default)]
pub struct GrpcMessage {
    pub data: Bytes,
    pub compression: Option<CompressionEncoding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionEncoding {
    Gzip,
    Identity,
}

/// Trailer metadata sent at the end of a gRPC stream.
#[derive(Debug, Clone, Default)]
pub struct Trailers {
    pub metadata: MetadataMap,
    pub status: GrpcStatus,
}
