//! HTTP request router with path matching, parameters, and method dispatch.

use crate::method::Method;
use crate::request::Request;
use crate::response::Response;
use crate::status::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;

pub type Handler = Arc<dyn Fn(Request) -> Response + Send + Sync>;

pub struct Route {
    pub method: Method,
    pub pattern: String,
    pub handler: Handler,
}

impl std::fmt::Debug for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("pattern", &self.pattern)
            .finish()
    }
}

type MiddlewareFn = Arc<dyn Fn(Request) -> Result<Request, Response> + Send + Sync>;

pub struct Router {
    routes: Vec<Route>,
    not_found_handler: Option<Handler>,
    middlewares: Vec<MiddlewareFn>,
}

impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("routes", &self.routes.len())
            .field("has_not_found", &self.not_found_handler.is_some())
            .field("middlewares", &self.middlewares.len())
            .finish()
    }
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            routes: self.routes.iter().map(|r| Route {
                method: r.method.clone(),
                pattern: r.pattern.clone(),
                handler: r.handler.clone(),
            }).collect(),
            not_found_handler: self.not_found_handler.clone(),
            middlewares: self.middlewares.clone(),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            not_found_handler: None,
            middlewares: Vec::new(),
        }
    }

    pub fn route(mut self, method: Method, pattern: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(Route {
            method,
            pattern: pattern.to_string(),
            handler: Arc::new(handler),
        });
        self
    }

    pub fn get(self, pattern: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Get, pattern, handler)
    }

    pub fn post(self, pattern: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Post, pattern, handler)
    }

    pub fn put(self, pattern: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Put, pattern, handler)
    }

    pub fn delete(self, pattern: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.route(Method::Delete, pattern, handler)
    }

    pub fn not_found(mut self, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.not_found_handler = Some(Arc::new(handler));
        self
    }

    pub fn middleware(mut self, mw: impl Fn(Request) -> Result<Request, Response> + Send + Sync + 'static) -> Self {
        self.middlewares.push(Arc::new(mw));
        self
    }

    pub fn handle(&self, mut req: Request) -> Response {
        // Apply middlewares
        for mw in &self.middlewares {
            match mw(req) {
                Ok(r) => req = r,
                Err(resp) => return resp,
            }
        }

        // Find matching route
        for route in &self.routes {
            if route.method != req.method {
                continue;
            }
            if let Some(_params) = Self::match_pattern(&route.pattern, &req.uri) {
                return (route.handler)(req);
            }
        }

        // 404
        if let Some(ref handler) = self.not_found_handler {
            return (handler)(req);
        }

        Response::new(StatusCode::not_found())
            .body_string("Not Found")
    }

    /// Simple pattern matching: supports {param} segments and trailing wildcards.
    fn match_pattern(pattern: &str, uri: &str) -> Option<HashMap<String, String>> {
        let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
        let uri_parts: Vec<&str> = uri.trim_matches('/').split('/').collect();

        // Exact match without params
        if !pattern.contains('{') && !pattern.ends_with('*') {
            return if pattern.trim_matches('/') == uri.trim_matches('/') {
                Some(HashMap::new())
            } else {
                None
            };
        }

        let mut params = HashMap::new();
        for (i, pp) in pattern_parts.iter().enumerate() {
            if pp.starts_with('{') && pp.ends_with('}') {
                let param_name = &pp[1..pp.len() - 1];
                if i >= uri_parts.len() {
                    return None;
                }
                params.insert(param_name.to_string(), uri_parts[i].to_string());
            } else if pp.ends_with('*') {
                return Some(params); // wildcard matches rest
            } else if i >= uri_parts.len() || pp != &uri_parts[i] {
                return None;
            }
        }

        // For non-wildcard patterns, lengths must match
        if !pattern.ends_with('*') && pattern_parts.len() != uri_parts.len() {
            return None;
        }

        Some(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_exact_match() {
        let router = Router::new()
            .get("/health", |_req| Response::ok().body_string("ok"));

        let req = Request::get("/health");
        let resp = router.handle(req);
        assert_eq!(resp.status.as_u16(), 200);
    }

    #[test]
    fn test_router_404() {
        let router = Router::new()
            .get("/exists", |_req| Response::ok());

        let resp = router.handle(Request::get("/missing"));
        assert_eq!(resp.status.as_u16(), 404);
    }

    #[test]
    fn test_router_param_matching() {
        let router = Router::new()
            .get("/users/{id}", |_req| Response::ok().body_string("user found"));

        let resp = router.handle(Request::get("/users/42"));
        assert_eq!(resp.status.as_u16(), 200);
    }

    #[test]
    fn test_router_middleware() {
        let router = Router::new()
            .middleware(|req| {
                if req.headers.get("Authorization").is_some() {
                    Ok(req)
                } else {
                    Err(Response::new(StatusCode::unauthorized()).body_string("No auth"))
                }
            })
            .get("/protected", |_req| Response::ok().body_string("secret"));

        let resp = router.handle(Request::get("/protected"));
        assert_eq!(resp.status.as_u16(), 401);

        let resp = router.handle(
            Request::get("/protected").header("Authorization", "Bearer token"),
        );
        assert_eq!(resp.status.as_u16(), 200);
    }
}
