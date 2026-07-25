//! HTTP methods as defined in RFC 7231 and RFC 5789.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
    Custom(String),
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Connect => "CONNECT",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
            Self::Patch => "PATCH",
            Self::Custom(s) => s,
        }
    }

    pub fn is_safe(&self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options | Self::Trace)
    }

    pub fn is_idempotent(&self) -> bool {
        matches!(
            self,
            Self::Get | Self::Head | Self::Put | Self::Delete | Self::Options | Self::Trace
        )
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "CONNECT" => Self::Connect,
            "OPTIONS" => Self::Options,
            "TRACE" => Self::Trace,
            "PATCH" => Self::Patch,
            other => Self::Custom(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_parsing() {
        assert_eq!(Method::from("get"), Method::Get);
        assert_eq!(Method::from("POST"), Method::Post);
        assert_eq!(Method::from("CUSTOM"), Method::Custom("CUSTOM".to_string()));
    }

    #[test]
    fn test_method_properties() {
        assert!(Method::Get.is_safe());
        assert!(Method::Get.is_idempotent());
        assert!(!Method::Post.is_safe());
        assert!(!Method::Post.is_idempotent());
        assert!(Method::Put.is_idempotent());
    }
}
