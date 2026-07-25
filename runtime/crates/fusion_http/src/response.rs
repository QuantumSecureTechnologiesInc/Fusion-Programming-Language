//! HTTP response type.

use crate::body::Body;
use crate::headers::{HeaderMap, HeaderName, HeaderValue};
use crate::status::StatusCode;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub version: crate::request::HttpVersion,
    pub headers: HeaderMap,
    pub body: Body,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            version: crate::request::HttpVersion::Http11,
            headers: HeaderMap::new(),
            body: Body::empty(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::ok())
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::not_found())
    }

    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::internal_server_error())
    }

    pub fn header(mut self, name: impl Into<HeaderName>, value: impl Into<HeaderValue>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn body_bytes(mut self, data: Vec<u8>) -> Self {
        self.body = Body::from(data);
        self
    }

    pub fn body_string(self, data: impl Into<String>) -> Self {
        let s = data.into();
        let len = s.len();
        self.body_bytes(s.into_bytes())
            .header(HeaderName::new("Content-Type"), HeaderValue::new("text/plain; charset=utf-8"))
            .header(HeaderName::new("Content-Length"), HeaderValue::new(len.to_string()))
    }

    pub fn json(self, data: &impl serde::Serialize) -> Self {
        match serde_json::to_vec(data) {
            Ok(bytes) => {
                let len = bytes.len();
                self.body_bytes(bytes)
                    .header(HeaderName::new("Content-Type"), HeaderValue::new("application/json"))
                    .header(HeaderName::new("Content-Length"), HeaderValue::new(len.to_string()))
            }
            Err(_) => self,
        }
    }

    pub fn html(self, data: impl Into<String>) -> Self {
        let s = data.into();
        let len = s.len();
        self.body_bytes(s.into_bytes())
            .header(HeaderName::new("Content-Type"), HeaderValue::new("text/html; charset=utf-8"))
            .header(HeaderName::new("Content-Length"), HeaderValue::new(len.to_string()))
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(HeaderName::new(name), HeaderValue::new(value));
        self
    }

    pub fn status_code(&self) -> u16 {
        self.status.as_u16()
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_basics() {
        let resp = Response::ok().body_string("Hello");
        assert_eq!(resp.status.as_u16(), 200);
    }

    #[test]
    fn test_response_not_found() {
        let resp = Response::not_found();
        assert_eq!(resp.status.as_u16(), 404);
    }

    #[test]
    fn test_response_with_header() {
        let resp = Response::ok()
            .with_header("X-Request-Id", "abc-123")
            .with_header("Cache-Control", "no-cache");
        assert_eq!(
            resp.headers.get("X-Request-Id").unwrap().as_str(),
            "abc-123"
        );
        assert_eq!(
            resp.headers.get("Cache-Control").unwrap().as_str(),
            "no-cache"
        );
    }
}
