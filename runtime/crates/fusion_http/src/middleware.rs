//! HTTP middleware traits and combinators.

use crate::request::Request;
use crate::response::Response;

/// A middleware wraps request processing, potentially modifying the request
/// or short-circuiting with a response.
pub trait Middleware: Send + Sync {
    /// Process the request. Return `Ok(req)` to continue, or `Err(resp)` to short-circuit.
    fn process(&self, req: Request) -> Result<Request, Response>;
}

/// Continuation type for nested middleware chains.
pub struct Next<'a> {
    pub(crate) handler: &'a dyn Fn(Request) -> Response,
}

impl<'a> Next<'a> {
    pub fn call(self, req: Request) -> Response {
        (self.handler)(req)
    }
}

/// Logging middleware that prints request method and path.
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn process(&self, req: Request) -> Result<Request, Response> {
        eprintln!("[HTTP] {} {}", req.method, req.uri);
        Ok(req)
    }
}

/// Authentication middleware that checks for a header.
pub struct AuthMiddleware {
    pub header_name: String,
    pub expected_value: Option<String>,
}

impl AuthMiddleware {
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            header_name: "Authorization".to_string(),
            expected_value: Some(format!("Bearer {}", token.into())),
        }
    }

    pub fn any_auth() -> Self {
        Self {
            header_name: "Authorization".to_string(),
            expected_value: None,
        }
    }
}

impl Middleware for AuthMiddleware {
    fn process(&self, req: Request) -> Result<Request, Response> {
        match req.headers.get(&self.header_name) {
            Some(val) => {
                if let Some(ref expected) = self.expected_value {
                    if val.as_str() == expected {
                        Ok(req)
                    } else {
                        Err(Response::new(crate::status::StatusCode::unauthorized())
                            .body_string("Invalid credentials"))
                    }
                } else {
                    Ok(req)
                }
            }
            None => Err(Response::new(crate::status::StatusCode::unauthorized())
                .body_string("Missing authorization")),
        }
    }
}

/// Rate limit middleware (simple token bucket).
pub struct RateLimitMiddleware {
    pub max_requests: u32,
    pub window_secs: u64,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self { max_requests, window_secs }
    }
}

impl Middleware for RateLimitMiddleware {
    fn process(&self, req: Request) -> Result<Request, Response> {
        // Simplified: in production this would track per-IP counters
        let _ = (self.max_requests, self.window_secs);
        Ok(req)
    }
}

/// Compression middleware stub.
pub struct CompressionMiddleware {
    pub algorithm: String,
}

impl Middleware for CompressionMiddleware {
    fn process(&self, req: Request) -> Result<Request, Response> {
        let _ = &self.algorithm;
        Ok(req)
    }
}

/// CORS middleware.
pub struct CorsMiddleware {
    pub allow_origin: String,
    pub allow_methods: String,
    pub allow_headers: String,
}

impl CorsMiddleware {
    pub fn permissive() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }
}

impl Middleware for CorsMiddleware {
    fn process(&self, req: Request) -> Result<Request, Response> {
        // For OPTIONS preflight, we'd return a response directly
        // Here we pass through and let the response layer add headers
        let _ = (&self.allow_origin, &self.allow_methods, &self.allow_headers);
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_middleware() {
        let mw = LoggingMiddleware;
        let req = Request::get("/test");
        assert!(mw.process(req).is_ok());
    }

    #[test]
    fn test_auth_middleware_reject() {
        let mw = AuthMiddleware::any_auth();
        let req = Request::get("/protected");
        assert!(mw.process(req).is_err());
    }

    #[test]
    fn test_auth_middleware_accept() {
        let mw = AuthMiddleware::any_auth();
        let req = Request::get("/protected").header("Authorization", "Bearer token");
        assert!(mw.process(req).is_ok());
    }

    #[test]
    fn test_cors_middleware() {
        let mw = CorsMiddleware::permissive();
        let req = Request::get("/api/data");
        assert!(mw.process(req).is_ok());
    }
}
