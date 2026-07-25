//! # Fusion HTTP
//!
//! HTTP/1.1 and HTTP/2 server and client with routing, middleware, and WebSocket upgrade.
//! Built on `hyper` and `tokio` for high-performance async networking.

pub mod method;
pub mod status;
pub mod headers;
pub mod request;
pub mod response;
pub mod body;
pub mod router;
pub mod middleware;
pub mod client;
pub mod server;
pub mod websocket;

pub use method::Method;
pub use status::{StatusCode, StatusClass};
pub use headers::HeaderMap;
pub use request::Request;
pub use response::Response;
pub use body::Body;
pub use router::Router;
pub use middleware::{Middleware, Next};
pub use server::HttpServer;
pub use client::HttpClient;
pub use websocket::WebSocketUpgrade;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP parse error: {0}")]
    Parse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid header value")]
    InvalidHeader,

    #[error("Body overflow: size {size} exceeds limit {limit}")]
    BodyOverflow { size: usize, limit: usize },

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Hyper error: {0}")]
    Hyper(String),

    #[error("WebSocket upgrade failed: {0}")]
    WebSocketUpgrade(String),
}

pub type Result<T> = std::result::Result<T, HttpError>;
