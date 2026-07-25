//! HTTP client for making outbound requests.

use crate::body::Body;
use crate::headers::{HeaderMap, HeaderName, HeaderValue};
use crate::method::Method;
use crate::request::{HttpVersion, Request};
use crate::response::Response;
use crate::status::StatusCode;
use crate::{HttpError, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the HTTP client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub default_headers: HashMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            follow_redirects: true,
            max_redirects: 5,
            default_headers: HashMap::new(),
        }
    }
}

/// HTTP client for outbound requests.
pub struct HttpClient {
    config: ClientConfig,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    pub fn with_config(config: ClientConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Build a request with default headers applied.
    pub fn build_request(&self, mut req: Request) -> Request {
        for (name, value) in &self.config.default_headers {
            if req.headers.get(name).is_none() {
                req.headers
                    .insert(HeaderName::new(name.clone()), HeaderValue::new(value.clone()));
            }
        }
        req
    }

    /// Serialize a request to raw HTTP/1.1 bytes.
    pub fn serialize_request(req: &Request) -> Vec<u8> {
        let mut output = Vec::new();
        let request_line = format!(
            "{} {} {}\r\n",
            req.method,
            req.uri,
            req.version.as_str()
        );
        output.extend_from_slice(request_line.as_bytes());

        for (name, value) in req.headers.iter() {
            let header = format!("{}: {}\r\n", name.as_str(), value.as_str());
            output.extend_from_slice(header.as_bytes());
        }

        if !req.headers.contains("Content-Length") && !req.body.is_empty() {
            let header = format!("Content-Length: {}\r\n", req.body.len());
            output.extend_from_slice(header.as_bytes());
        }

        if !req.headers.contains("Connection") {
            output.extend_from_slice(b"Connection: close\r\n");
        }

        output.extend_from_slice(b"\r\n");
        output.extend_from_slice(req.body.bytes());
        output
    }

    /// Parse a raw HTTP response from a byte buffer.
    pub fn parse_response(raw: &[u8]) -> Result<Response> {
        let text = std::str::from_utf8(raw)
            .map_err(|_| HttpError::Parse("Invalid UTF-8 in response".to_string()))?;

        let (header_section, body_bytes) = if let Some(idx) = text.find("\r\n\r\n") {
            (&text[..idx], &raw[idx + 4..])
        } else {
            return Err(HttpError::Parse("No header/body separator found".to_string()));
        };

        let mut lines = header_section.lines();
        let status_line = lines
            .next()
            .ok_or_else(|| HttpError::Parse("Empty response".to_string()))?;

        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return Err(HttpError::Parse(format!("Invalid status line: {}", status_line)));
        }

        let status_code: u16 = parts[1]
            .parse()
            .map_err(|_| HttpError::Parse(format!("Invalid status code: {}", parts[1])))?;

        let mut headers = HeaderMap::new();
        for line in lines {
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(
                    HeaderName::new(name.trim().to_string()),
                    HeaderValue::new(value.trim().to_string()),
                );
            }
        }

        // Respect Content-Length for body parsing
        let body = if let Some(len_val) = headers.get("Content-Length") {
            if let Ok(len) = len_val.as_str().parse::<usize>() {
                Body::from(body_bytes[..len.min(body_bytes.len())].to_vec())
            } else {
                Body::from(body_bytes.to_vec())
            }
        } else {
            Body::from(body_bytes.to_vec())
        };

        Ok(Response {
            status: StatusCode::from(status_code),
            version: HttpVersion::Http11,
            headers,
            body,
        })
    }

    /// Build a GET request for the given URL.
    pub fn get(&self, url: &str) -> Request {
        self.build_request(Request::get(url))
    }

    /// Build a POST request for the given URL with a JSON body.
    pub fn post_json(&self, url: &str, data: &impl serde::Serialize) -> Request {
        self.build_request(Request::post(url).json(data))
    }

    /// Build a PUT request for the given URL with a JSON body.
    pub fn put_json(&self, url: &str, data: &impl serde::Serialize) -> Request {
        self.build_request(Request::put(url).json(data))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_request() {
        let req = Request::get("/api/data")
            .header("Accept", "application/json")
            .header("Host", "example.com");
        let bytes = HttpClient::serialize_request(&req);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.starts_with("GET /api/data HTTP/1.1"));
        assert!(text.contains("Accept: application/json"));
    }

    #[test]
    fn test_parse_response() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";
        let resp = HttpClient::parse_response(raw).unwrap();
        assert_eq!(resp.status.as_u16(), 200);
        assert_eq!(resp.body.to_str().unwrap(), "OK");
    }

    #[test]
    fn test_parse_404_response() {
        let raw = b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        let resp = HttpClient::parse_response(raw).unwrap();
        assert_eq!(resp.status.as_u16(), 404);
    }

    #[test]
    fn test_client_build_get() {
        let client = HttpClient::new();
        let req = client.get("http://example.com/");
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.uri, "http://example.com/");
    }
}
