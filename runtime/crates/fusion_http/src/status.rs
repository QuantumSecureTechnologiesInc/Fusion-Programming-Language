//! HTTP status codes per RFC 7231.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusCode {
    code: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
    Unknown,
}

impl StatusCode {
    pub fn new(code: u16) -> Self {
        Self { code }
    }

    pub fn as_u16(&self) -> u16 {
        self.code
    }

    pub fn canonical_reason(&self) -> Option<&'static str> {
        match self.code {
            100 => Some("Continue"),
            101 => Some("Switching Protocols"),
            200 => Some("OK"),
            201 => Some("Created"),
            204 => Some("No Content"),
            301 => Some("Moved Permanently"),
            302 => Some("Found"),
            304 => Some("Not Modified"),
            400 => Some("Bad Request"),
            401 => Some("Unauthorized"),
            403 => Some("Forbidden"),
            404 => Some("Not Found"),
            405 => Some("Method Not Allowed"),
            500 => Some("Internal Server Error"),
            502 => Some("Bad Gateway"),
            503 => Some("Service Unavailable"),
            _ => None,
        }
    }

    pub fn class(&self) -> StatusClass {
        match self.code {
            100..=199 => StatusClass::Informational,
            200..=299 => StatusClass::Success,
            300..=399 => StatusClass::Redirection,
            400..=499 => StatusClass::ClientError,
            500..=599 => StatusClass::ServerError,
            _ => StatusClass::Unknown,
        }
    }

    pub fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    pub fn is_redirect(&self) -> bool {
        self.code >= 300 && self.code < 400
    }

    pub fn is_client_error(&self) -> bool {
        self.code >= 400 && self.code < 500
    }

    pub fn is_server_error(&self) -> bool {
        self.code >= 500 && self.code < 600
    }

    // Common status codes
    pub fn ok() -> Self { Self { code: 200 } }
    pub fn created() -> Self { Self { code: 201 } }
    pub fn no_content() -> Self { Self { code: 204 } }
    pub fn moved_permanently() -> Self { Self { code: 301 } }
    pub fn not_modified() -> Self { Self { code: 304 } }
    pub fn bad_request() -> Self { Self { code: 400 } }
    pub fn unauthorized() -> Self { Self { code: 401 } }
    pub fn forbidden() -> Self { Self { code: 403 } }
    pub fn not_found() -> Self { Self { code: 404 } }
    pub fn method_not_allowed() -> Self { Self { code: 405 } }
    pub fn internal_server_error() -> Self { Self { code: 500 } }
    pub fn service_unavailable() -> Self { Self { code: 503 } }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        Self { code }
    }
}

impl From<StatusCode> for u16 {
    fn from(s: StatusCode) -> u16 {
        s.code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_basics() {
        let s = StatusCode::ok();
        assert_eq!(s.as_u16(), 200);
        assert!(s.is_success());
        assert_eq!(s.class(), StatusClass::Success);
        assert_eq!(s.canonical_reason(), Some("OK"));
    }

    #[test]
    fn test_status_classes() {
        assert!(StatusCode::new(101).class() == StatusClass::Informational);
        assert!(StatusCode::new(404).is_client_error());
        assert!(StatusCode::new(500).is_server_error());
    }
}
