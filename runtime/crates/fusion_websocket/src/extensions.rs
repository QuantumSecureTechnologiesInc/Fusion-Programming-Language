//! WebSocket extension negotiation and support.

use std::collections::HashMap;

/// A WebSocket extension.
#[derive(Debug, Clone)]
pub struct Extension {
    pub name: String,
    pub parameters: HashMap<String, String>,
}

impl Extension {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parameters: HashMap::new(),
        }
    }

    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Parse the Sec-WebSocket-Extensions header value.
    pub fn parse_list(header: &str) -> Vec<Self> {
        header
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|ext_str| {
                let mut parts = ext_str.splitn(2, ';');
                let name = parts.next().unwrap_or("").trim().to_string();
                let mut ext = Extension::new(name);
                if let Some(params_str) = parts.next() {
                    for param in params_str.split(';') {
                        let param = param.trim();
                        if let Some((k, v)) = param.split_once('=') {
                            ext = ext.parameter(k.trim(), v.trim().trim_matches('"'));
                        }
                    }
                }
                ext
            })
            .collect()
    }

    /// Serialize to a Sec-WebSocket-Extensions header value.
    pub fn serialize_list(extensions: &[Extension]) -> String {
        extensions
            .iter()
            .map(|ext| {
                if ext.parameters.is_empty() {
                    ext.name.clone()
                } else {
                    let params: Vec<String> = ext
                        .parameters
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    format!("{}; {}", ext.name, params.join("; "))
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Per-message compression extension (permessage-deflate).
#[derive(Debug, Clone)]
pub struct PerMessageDeflate {
    pub server_max_window_bits: Option<u8>,
    pub client_max_window_bits: Option<u8>,
    pub server_no_context_takeover: bool,
    pub client_no_context_takeover: bool,
}

impl PerMessageDeflate {
    pub fn new() -> Self {
        Self {
            server_max_window_bits: Some(15),
            client_max_window_bits: Some(15),
            server_no_context_takeover: false,
            client_no_context_takeover: false,
        }
    }

    pub fn as_extension(&self) -> Extension {
        let mut ext = Extension::new("permessage-deflate");
        if let Some(bits) = self.server_max_window_bits {
            ext = ext.parameter("server_max_window_bits", bits.to_string());
        }
        if let Some(bits) = self.client_max_window_bits {
            ext = ext.parameter("client_max_window_bits", bits.to_string());
        }
        if self.server_no_context_takeover {
            ext = ext.parameter("server_no_context_takeover", "".to_string());
        }
        if self.client_no_context_takeover {
            ext = ext.parameter("client_no_context_takeover", "".to_string());
        }
        ext
    }
}

impl Default for PerMessageDeflate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extension_list() {
        let header = "permessage-deflate; server_max_window_bits=15, x-webkit-deflate-frame";
        let extensions = Extension::parse_list(header);
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0].name, "permessage-deflate");
        assert_eq!(
            extensions[0].parameters.get("server_max_window_bits").unwrap(),
            "15"
        );
        assert_eq!(extensions[1].name, "x-webkit-deflate-frame");
    }

    #[test]
    fn test_serialize_extension_list() {
        let exts = vec![
            Extension::new("permessage-deflate"),
            Extension::new("x-webkit-deflate-frame"),
        ];
        let serialized = Extension::serialize_list(&exts);
        assert!(serialized.contains("permessage-deflate"));
        assert!(serialized.contains("x-webkit-deflate-frame"));
    }

    #[test]
    fn test_permessage_deflate() {
        let pmd = PerMessageDeflate::new();
        let ext = pmd.as_extension();
        assert_eq!(ext.name, "permessage-deflate");
        assert!(ext.parameters.contains_key("server_max_window_bits"));
    }
}
