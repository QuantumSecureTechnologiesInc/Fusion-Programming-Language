//! WebSocket upgrade support for HTTP/1.1.

use crate::headers::{HeaderName, HeaderValue};
use crate::request::Request;
use crate::response::Response;
use crate::status::StatusCode;
use crate::{HttpError, Result};
use base64::Engine;
use sha1::{Digest, Sha1};

/// WebSocket magic GUID for Sec-WebSocket-Accept computation (RFC 6455).
pub const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Check if a request wants a WebSocket upgrade.
pub fn is_upgrade_request(req: &Request) -> bool {
    let has_upgrade = req
        .headers
        .get("Upgrade")
        .map(|v| v.as_str().eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);

    let has_connection_upgrade = req
        .headers
        .get("Connection")
        .map(|v| v.as_str().to_lowercase().contains("upgrade"))
        .unwrap_or(false);

    let has_version = req
        .headers
        .get("Sec-WebSocket-Version")
        .map(|v| v.as_str() == "13")
        .unwrap_or(false);

    let has_key = req.headers.get("Sec-WebSocket-Key").is_some();

    has_upgrade && has_connection_upgrade && has_version && has_key
}

/// Compute the Sec-WebSocket-Accept value per RFC 6455.
pub fn compute_accept_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_GUID.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(result)
}

/// Perform the server-side WebSocket upgrade handshake.
pub fn upgrade(req: Request) -> Result<(Response, WebSocketUpgrade)> {
    let key = req
        .headers
        .get("Sec-WebSocket-Key")
        .ok_or_else(|| HttpError::WebSocketUpgrade("Missing Sec-WebSocket-Key".to_string()))?
        .as_str()
        .to_string();

    let accept_key = compute_accept_key(&key);

    let response = Response::new(StatusCode::new(101))
        .header(HeaderName::new("Upgrade"), HeaderValue::new("websocket"))
        .header(HeaderName::new("Connection"), HeaderValue::new("Upgrade"))
        .header(
            HeaderName::new("Sec-WebSocket-Accept"),
            HeaderValue::new(accept_key),
        );

    Ok((
        response,
        WebSocketUpgrade {
            version: "13".to_string(),
            protocol: req
                .headers
                .get("Sec-WebSocket-Protocol")
                .map(|v| v.as_str().to_string()),
        },
    ))
}

/// Client-side WebSocket upgrade request builder.
pub fn client_upgrade_request(url: &str, key: &str) -> Request {
    Request::get(url)
        .header(HeaderName::new("Upgrade"), HeaderValue::new("websocket"))
        .header(HeaderName::new("Connection"), HeaderValue::new("Upgrade"))
        .header(HeaderName::new("Sec-WebSocket-Version"), HeaderValue::new("13"))
        .header(HeaderName::new("Sec-WebSocket-Key"), HeaderValue::new(key))
}

/// Verify a server's Sec-WebSocket-Accept matches our key.
pub fn verify_accept_key(server_accept: &str, client_key: &str) -> bool {
    compute_accept_key(client_key) == server_accept
}

/// Information about a completed WebSocket upgrade.
#[derive(Debug, Clone)]
pub struct WebSocketUpgrade {
    pub version: String,
    pub protocol: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_accept_key() {
        // RFC 6455 example
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = compute_accept_key(key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_is_upgrade_request() {
        let req = Request::get("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==");
        assert!(is_upgrade_request(&req));
    }

    #[test]
    fn test_not_upgrade_request() {
        let req = Request::get("/api");
        assert!(!is_upgrade_request(&req));
    }

    #[test]
    fn test_upgrade_handshake() {
        let req = Request::get("/ws")
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==");

        let (resp, upgrade) = upgrade(req).unwrap();
        assert_eq!(resp.status.as_u16(), 101);
        assert_eq!(
            resp.headers.get("Sec-WebSocket-Accept").unwrap().as_str(),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
        assert_eq!(upgrade.version, "13");
    }

    #[test]
    fn test_verify_accept_key() {
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
        assert!(verify_accept_key(accept, key));
        assert!(!verify_accept_key("wrong", key));
    }
}
