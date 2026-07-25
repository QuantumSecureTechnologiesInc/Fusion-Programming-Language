//! HTTP body type — a simple bytes-based body with size limits.

use bytes::Bytes;

#[derive(Debug, Clone, Default)]
pub struct Body {
    data: Bytes,
}

impl Body {
    pub fn empty() -> Self {
        Self { data: Bytes::new() }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> Bytes {
        self.data
    }

    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

impl From<Vec<u8>> for Body {
    fn from(data: Vec<u8>) -> Self {
        Self { data: Bytes::from(data) }
    }
}

impl From<&[u8]> for Body {
    fn from(data: &[u8]) -> Self {
        Self { data: Bytes::copy_from_slice(data) }
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self { data: Bytes::from(s) }
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Self { data: Bytes::from(s.to_string()) }
    }
}

impl From<Bytes> for Body {
    fn from(data: Bytes) -> Self {
        Self { data }
    }
}

impl AsRef<[u8]> for Body {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_body() {
        let body = Body::empty();
        assert!(body.is_empty());
        assert_eq!(body.len(), 0);
    }

    #[test]
    fn test_body_from_string() {
        let body = Body::from("hello world");
        assert_eq!(body.to_str().unwrap(), "hello world");
        assert_eq!(body.len(), 11);
    }

    #[test]
    fn test_body_from_bytes() {
        let body = Body::from(vec![0u8, 1, 2, 3]);
        assert_eq!(body.bytes(), &[0, 1, 2, 3]);
    }
}
