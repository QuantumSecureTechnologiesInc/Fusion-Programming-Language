//! gRPC metadata map for request and response headers.

use bytes::Bytes;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct MetadataMap {
    entries: Vec<(MetadataKey, MetadataValue)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetadataKey(String);

#[derive(Debug, Clone)]
pub struct MetadataValue {
    data: Bytes,
}

impl MetadataKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the key is a gRPC binary header (ends with `-bin`).
    pub fn is_binary(&self) -> bool {
        self.0.ends_with("-bin")
    }
}

impl From<&str> for MetadataKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl MetadataValue {
    pub fn from_str(value: impl Into<String>) -> Self {
        Self {
            data: Bytes::from(value.into()),
        }
    }

    pub fn from_bytes(value: impl Into<Bytes>) -> Self {
        Self { data: value.into() }
    }

    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    pub fn to_bytes(&self) -> &Bytes {
        &self.data
    }
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl fmt::Display for MetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_str() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<binary {} bytes>", self.data.len()),
        }
    }
}

impl MetadataMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: MetadataKey, value: MetadataValue) -> Option<MetadataValue> {
        if let Some(idx) = self.entries.iter().position(|(k, _)| *k == key) {
            let old = self.entries[idx].1.clone();
            self.entries[idx].1 = value;
            Some(old)
        } else {
            self.entries.push((key, value));
            None
        }
    }

    pub fn get(&self, key: &str) -> Option<&MetadataValue> {
        self.entries
            .iter()
            .find(|(k, _)| k.as_str() == key)
            .map(|(_, v)| v)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|(k, _)| k.as_str() == key)
    }

    pub fn remove(&mut self, key: &str) -> Option<MetadataValue> {
        if let Some(idx) = self.entries.iter().position(|(k, _)| k.as_str() == key) {
            Some(self.entries.remove(idx).1)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&MetadataKey, &MetadataValue)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }

    /// Convert to a HashMap<String, String> for wire transport.
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        self.entries
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_string()))
            .collect()
    }

    /// Build from a HashMap<String, String>.
    pub fn from_hashmap(map: HashMap<String, String>) -> Self {
        let mut meta = Self::new();
        for (k, v) in map {
            meta.insert(MetadataKey::new(k), MetadataValue::from_str(v));
        }
        meta
    }
}

impl fmt::Display for MetadataMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.entries {
            writeln!(f, "{}: {}", k.as_str(), v)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_insert_get() {
        let mut map = MetadataMap::new();
        map.insert(MetadataKey::new("content-type"), MetadataValue::from_str("application/grpc"));
        assert!(map.contains("content-type"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_metadata_binary_key() {
        assert!(MetadataKey::new("grpc-payload-bin").is_binary());
        assert!(!MetadataKey::new("content-type").is_binary());
    }

    #[test]
    fn test_metadata_hashmap_roundtrip() {
        let mut map = MetadataMap::new();
        map.insert(MetadataKey::new("x-request-id"), MetadataValue::from_str("123"));
        let hm = map.to_hashmap();
        let map2 = MetadataMap::from_hashmap(hm);
        assert_eq!(map2.get("x-request-id").unwrap().to_str().unwrap(), "123");
    }
}
