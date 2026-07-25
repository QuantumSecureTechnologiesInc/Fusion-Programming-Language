//! HTTP header map implementation.

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct HeaderMap {
    headers: Vec<(HeaderName, HeaderValue)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeaderName(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderValue(String);

impl HeaderName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_ascii(&self) -> bool {
        self.0.bytes().all(|b| b.is_ascii())
    }
}

impl From<&str> for HeaderName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl HeaderValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_str_lossy(&self) -> &str {
        &self.0
    }
}

impl From<&str> for HeaderValue {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl HeaderMap {
    pub fn new() -> Self {
        Self { headers: Vec::new() }
    }

    pub fn with_capacity(n: usize) -> Self {
        Self { headers: Vec::with_capacity(n) }
    }

    pub fn insert(&mut self, name: HeaderName, value: HeaderValue) -> Option<HeaderValue> {
        if let Some(idx) = self.headers.iter().position(|(n, _)| *n == name) {
            let old = self.headers[idx].1.clone();
            self.headers[idx].1 = value;
            Some(old)
        } else {
            self.headers.push((name, value));
            None
        }
    }

    pub fn get(&self, name: &str) -> Option<&HeaderValue> {
        self.headers
            .iter()
            .find(|(n, _)| n.as_str().eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }

    pub fn get_all(&self, name: &str) -> Vec<&HeaderValue> {
        self.headers
            .iter()
            .filter(|(n, _)| n.as_str().eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
            .collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.headers
            .iter()
            .any(|(n, _)| n.as_str().eq_ignore_ascii_case(name))
    }

    pub fn remove(&mut self, name: &str) -> Option<HeaderValue> {
        if let Some(idx) = self.headers.iter().position(|(n, _)| n.as_str().eq_ignore_ascii_case(name)) {
            Some(self.headers.remove(idx).1)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.headers.iter().map(|(n, v)| (n, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &HeaderName> {
        self.headers.iter().map(|(n, _)| n)
    }

    pub fn extend(&mut self, other: HeaderMap) {
        for (name, value) in other.headers {
            self.insert(name, value);
        }
    }
}

impl fmt::Display for HeaderMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, value) in &self.headers {
            writeln!(f, "{}: {}", name.as_str(), value.as_str())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_map_insert_get() {
        let mut map = HeaderMap::new();
        map.insert(HeaderName::new("Content-Type"), HeaderValue::new("text/html"));
        assert_eq!(map.get("Content-Type").unwrap().as_str(), "text/html");
    }

    #[test]
    fn test_header_case_insensitive() {
        let mut map = HeaderMap::new();
        map.insert(HeaderName::new("X-Custom"), HeaderValue::new("value"));
        assert!(map.contains("x-custom"));
        assert!(map.contains("X-CUSTOM"));
    }

    #[test]
    fn test_header_remove() {
        let mut map = HeaderMap::new();
        map.insert(HeaderName::new("Authorization"), HeaderValue::new("Bearer token"));
        assert!(map.remove("authorization").is_some());
        assert!(map.is_empty());
    }

    #[test]
    fn test_header_map_display() {
        let mut map = HeaderMap::new();
        map.insert(HeaderName::new("Host"), HeaderValue::new("example.com"));
        let display = format!("{}", map);
        assert!(display.contains("Host: example.com"));
    }
}
