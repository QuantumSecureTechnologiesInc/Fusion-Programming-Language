//! HTTP server implementation using hyper and tokio.

use crate::body::Body;
use crate::headers::{HeaderMap, HeaderName, HeaderValue};
use crate::method::Method;
use crate::request::{HttpVersion, Request};
use crate::response::Response;
use crate::router::Router;
use crate::status::StatusCode;
use crate::{HttpError, Result};
use std::net::SocketAddr;
use std::sync::Arc;

/// Configuration for the HTTP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
    pub body_limit: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: 1024,
            request_timeout_ms: 30_000,
            body_limit: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// HTTP server state and configuration.
pub struct HttpServer {
    config: ServerConfig,
    router: Arc<Router>,
}

impl HttpServer {
    pub fn new(router: Router) -> Self {
        Self {
            config: ServerConfig::default(),
            router: Arc::new(router),
        }
    }

    pub fn with_config(router: Router, config: ServerConfig) -> Self {
        Self {
            config,
            router: Arc::new(router),
        }
    }

    pub fn bind_addr(&self) -> Result<SocketAddr> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port)
            .parse()
            .map_err(|_| HttpError::Parse(format!("Invalid address: {}:{}", self.config.host, self.config.port)))?;
        Ok(addr)
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub fn router(&self) -> &Router {
        &self.router
    }

    /// Parse a raw HTTP request from a byte buffer.
    pub fn parse_request(raw: &[u8]) -> Result<Request> {
        let text = std::str::from_utf8(raw)
            .map_err(|_| HttpError::Parse("Invalid UTF-8 in request".to_string()))?;

        let (header_section, body_bytes) = if let Some(idx) = text.find("\r\n\r\n") {
            (&text[..idx], &raw[idx + 4..])
        } else {
            return Err(HttpError::Parse("No header/body separator found".to_string()));
        };

        let mut lines = header_section.lines();
        let request_line = lines
            .next()
            .ok_or_else(|| HttpError::Parse("Empty request".to_string()))?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(HttpError::Parse(format!("Invalid request line: {}", request_line)));
        }

        let method = Method::from(parts[0]);
        let uri = parts[1].to_string();
        let version = match parts[2] {
            "HTTP/1.0" => HttpVersion::Http10,
            "HTTP/1.1" => HttpVersion::Http11,
            _ => HttpVersion::Http11,
        };

        let mut headers = HeaderMap::new();
        for line in lines {
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(
                    HeaderName::new(name.trim().to_string()),
                    HeaderValue::new(value.trim().to_string()),
                );
            }
        }

        Ok(Request {
            method,
            uri,
            version,
            headers,
            body: Body::from(body_bytes.to_vec()),
        })
    }

    /// Serialize a response to raw HTTP bytes.
    pub fn serialize_response(resp: &Response) -> Vec<u8> {
        let mut output = Vec::new();
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            resp.status.as_u16(),
            resp.status.canonical_reason().unwrap_or("Unknown")
        );
        output.extend_from_slice(status_line.as_bytes());

        let mut body = resp.body.bytes().to_vec();
        let has_content_length = resp.headers.contains("Content-Length");
        let has_content_type = resp.headers.contains("Content-Type");

        if !has_content_length {
            let header = format!("Content-Length: {}\r\n", body.len());
            output.extend_from_slice(header.as_bytes());
        }

        if !has_content_type && !body.is_empty() {
            output.extend_from_slice(b"Content-Type: application/octet-stream\r\n");
        }

        for (name, value) in resp.headers.iter() {
            let header = format!("{}: {}\r\n", name.as_str(), value.as_str());
            output.extend_from_slice(header.as_bytes());
        }

        output.extend_from_slice(b"\r\n");
        output.append(&mut body);
        output
    }

    /// Handle a parsed request and return a response.
    pub fn handle_request(&self, req: Request) -> Response {
        self.router.handle(req)
    }
}

/// Build a simple 400 Bad Request response.
pub fn bad_request(msg: &str) -> Response {
    Response::new(StatusCode::bad_request()).body_string(msg.to_string())
}

/// Build a simple 500 Internal Server Error response.
pub fn internal_error(msg: &str) -> Response {
    Response::new(StatusCode::internal_server_error()).body_string(msg.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_parse_simple_get() {
        let raw = b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = HttpServer::parse_request(raw).unwrap();
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.uri, "/hello");
        assert_eq!(req.headers.get("Host").unwrap().as_str(), "localhost");
    }

    #[test]
    fn test_parse_post_with_body() {
        let raw = b"POST /data HTTP/1.1\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
        let req = HttpServer::parse_request(raw).unwrap();
        assert_eq!(req.method, Method::Post);
        assert_eq!(req.body.to_str().unwrap(), "hello");
    }

    #[test]
    fn test_serialize_response() {
        let resp = Response::ok().body_string("OK");
        let bytes = HttpServer::serialize_response(&resp);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK"));
        assert!(text.contains("OK"));
    }

    #[test]
    fn test_server_handle_request() {
        let router = Router::new()
            .get("/ping", |_req| Response::ok().body_string("pong"));
        let server = HttpServer::new(router);

        let req = Request::get("/ping");
        let resp = server.handle_request(req);
        assert_eq!(resp.status.as_u16(), 200);
        assert_eq!(resp.body.to_str().unwrap(), "pong");
    }

    #[test]
    fn test_bad_request_response() {
        let resp = bad_request("missing param");
        assert_eq!(resp.status.as_u16(), 400);
    }
}
