//! HTTP request type.

use crate::body::Body;
use crate::headers::{HeaderMap, HeaderName, HeaderValue};
use crate::method::Method;
use crate::HttpError;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub version: HttpVersion,
    pub headers: HeaderMap,
    pub body: Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    Http10,
    Http11,
    H2,
}

impl HttpVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http10 => "HTTP/1.0",
            Self::Http11 => "HTTP/1.1",
            Self::H2 => "HTTP/2",
        }
    }
}

impl Request {
    pub fn new(method: Method, uri: impl Into<String>) -> Self {
        Self {
            method,
            uri: uri.into(),
            version: HttpVersion::Http11,
            headers: HeaderMap::new(),
            body: Body::empty(),
        }
    }

    pub fn get(uri: impl Into<String>) -> Self {
        Self::new(Method::Get, uri)
    }

    pub fn post(uri: impl Into<String>) -> Self {
        Self::new(Method::Post, uri)
    }

    pub fn put(uri: impl Into<String>) -> Self {
        Self::new(Method::Put, uri)
    }

    pub fn delete(uri: impl Into<String>) -> Self {
        Self::new(Method::Delete, uri)
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

    pub fn with_version(mut self, version: HttpVersion) -> Self {
        self.version = version;
        self
    }

    /// Parse the URI into (host, port, path, query).
    pub fn parse_uri(&self) -> Result<UriParts, HttpError> {
        let uri = &self.uri;
        let (scheme_and_rest, path_and_query) = if let Some(idx) = uri.find("://") {
            let rest = &uri[idx + 3..];
            let scheme = &uri[..idx];
            if let Some(pq_idx) = rest.find('/') {
                (Some(scheme), &rest[pq_idx..])
            } else {
                (Some(scheme), "/")
            }
        } else {
            (None, uri.as_str())
        };

        let (path, query) = if let Some(q_idx) = path_and_query.find('?') {
            (&path_and_query[..q_idx], Some(&path_and_query[q_idx + 1..]))
        } else {
            (path_and_query, None)
        };

        Ok(UriParts {
            scheme: scheme_and_rest.map(|s| s.to_string()),
            host: None,
            port: None,
            path: path.to_string(),
            query: query.map(|q| q.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct UriParts {
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
}

impl serde::Serialize for Request {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("method", self.method.as_str())?;
        map.serialize_entry("uri", &self.uri)?;
        map.serialize_entry("version", self.version.as_str())?;
        // Only serialize headers as HashMap
        let headers: std::collections::HashMap<&str, &str> = self
            .headers
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        map.serialize_entry("headers", &headers)?;
        map.serialize_entry("body_len", &self.body.len())?;
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_basics() {
        let req = Request::get("/api/users")
            .header("Accept", "application/json");
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.uri, "/api/users");
        assert_eq!(req.headers.get("Accept").unwrap().as_str(), "application/json");
    }

    #[test]
    fn test_parse_uri() {
        let req = Request::get("https://example.com:8443/path?q=1&b=2");
        let parts = req.parse_uri().unwrap();
        assert_eq!(parts.scheme, Some("https".to_string()));
        assert_eq!(parts.path, "/path");
        assert_eq!(parts.query, Some("q=1&b=2".to_string()));
    }
}
