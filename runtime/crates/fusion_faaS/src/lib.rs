use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// HTTP types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok(body: &[u8]) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "application/json".into());
        Self {
            status: 200,
            headers,
            body: body.to_vec(),
        }
    }

    pub fn error(status: u16, message: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "text/plain".into());
        Self {
            status,
            headers,
            body: message.as_bytes().to_vec(),
        }
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {} ({})", self.status, self.body.len())
    }
}

// ---------------------------------------------------------------------------
// Function registry
// ---------------------------------------------------------------------------

type FunctionHandler = Box<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync>;

pub struct FunctionEntry {
    pub name: String,
    pub route: String,
    pub timeout: Duration,
    handler: FunctionHandler,
}

impl FunctionEntry {
    pub fn invoke(&self, req: &HttpRequest) -> HttpResponse {
        (self.handler)(req)
    }
}

// ---------------------------------------------------------------------------
// Cold-start manager
// ---------------------------------------------------------------------------

pub struct ColdStartPool {
    warm_instances: Vec<FunctionEntry>,
    cold_instances: Vec<FunctionEntry>,
    cold_start_count: AtomicU64,
    warm_start_count: AtomicU64,
}

impl ColdStartPool {
    pub fn new() -> Self {
        Self {
            warm_instances: Vec::new(),
            cold_instances: Vec::new(),
            cold_start_count: AtomicU64::new(0),
            warm_start_count: AtomicU64::new(0),
        }
    }

    pub fn pre_warm(&mut self, entry: FunctionEntry) {
        self.warm_instances.push(entry);
    }

    pub fn add_cold(&mut self, entry: FunctionEntry) {
        self.cold_instances.push(entry);
    }

    pub fn try_get_warm(&mut self, name: &str) -> Option<&FunctionEntry> {
        if let Some(pos) = self.warm_instances.iter().position(|e| e.name == name) {
            self.warm_start_count.fetch_add(1, Ordering::Relaxed);
            Some(&self.warm_instances[pos])
        } else {
            None
        }
    }

    pub fn promote_cold(&mut self, name: &str) -> Option<&FunctionEntry> {
        if let Some(pos) = self.cold_instances.iter().position(|e| e.name == name) {
            self.cold_start_count.fetch_add(1, Ordering::Relaxed);
            // Move to warm for next invocation
            let entry = self.cold_instances.remove(pos);
            self.warm_instances.push(entry);
            self.warm_instances.last()
        } else {
            None
        }
    }

    pub fn cold_start_count(&self) -> u64 {
        self.cold_start_count.load(Ordering::Relaxed)
    }

    pub fn warm_start_count(&self) -> u64 {
        self.warm_start_count.load(Ordering::Relaxed)
    }
}

impl Default for ColdStartPool {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Auto-scaler
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoScalingPolicy {
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_requests_per_instance: u32,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

impl Default for AutoScalingPolicy {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 100,
            target_requests_per_instance: 1000,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        }
    }
}

pub struct AutoScaler {
    policy: AutoScalingPolicy,
    current_instances: u32,
    request_counts: Mutex<HashMap<String, u64>>,
    window_start: Mutex<Instant>,
}

impl AutoScaler {
    pub fn new(policy: AutoScalingPolicy) -> Self {
        Self {
            current_instances: policy.min_instances,
            policy,
            request_counts: Mutex::new(HashMap::new()),
            window_start: Mutex::new(Instant::now()),
        }
    }

    /// Record a request and return whether scaling action is needed.
    pub fn record_request(&self, function_name: &str) -> ScaleDecision {
        let mut counts = self.request_counts.lock().unwrap();
        let count = counts.entry(function_name.to_string()).or_insert(0);
        *count += 1;

        let total: u64 = counts.values().sum();
        let per_instance = total as f64 / self.current_instances as f64;
        let target = self.policy.target_requests_per_instance as f64;

        if per_instance > target * self.policy.scale_up_threshold {
            let desired = self.suggest_scale_up();
            ScaleDecision::ScaleUp(desired)
        } else if per_instance < target * self.policy.scale_down_threshold
            && self.current_instances > self.policy.min_instances
        {
            ScaleDecision::ScaleDown(self.current_instances - 1)
        } else {
            ScaleDecision::NoChange
        }
    }

    pub fn apply_scale(&mut self, decision: &ScaleDecision) {
        match decision {
            ScaleDecision::ScaleUp(n) => {
                self.current_instances = (*n).min(self.policy.max_instances);
            }
            ScaleDecision::ScaleDown(n) => {
                self.current_instances = (*n).max(self.policy.min_instances);
            }
            ScaleDecision::NoChange => {}
        }
    }

    pub fn current_instances(&self) -> u32 {
        self.current_instances
    }

    fn suggest_scale_up(&self) -> u32 {
        (self.current_instances + 1).min(self.policy.max_instances)
    }

    pub fn reset_window(&self) {
        let mut counts = self.request_counts.lock().unwrap();
        counts.clear();
        *self.window_start.lock().unwrap() = Instant::now();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaleDecision {
    ScaleUp(u32),
    ScaleDown(u32),
    NoChange,
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

pub struct FaasRuntime {
    functions: HashMap<String, FunctionEntry>,
    pool: ColdStartPool,
    scaler: Mutex<AutoScaler>,
    request_counter: AtomicU64,
}

impl FaasRuntime {
    pub fn new(scaling_policy: AutoScalingPolicy) -> Self {
        Self {
            functions: HashMap::new(),
            pool: ColdStartPool::new(),
            scaler: Mutex::new(AutoScaler::new(scaling_policy)),
            request_counter: AtomicU64::new(0),
        }
    }

    pub fn register_function<F>(&mut self, name: &str, route: &str, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        let entry = FunctionEntry {
            name: name.to_string(),
            route: route.to_string(),
            timeout: Duration::from_secs(30),
            handler: Box::new(handler),
        };
        self.functions.insert(name.to_string(), entry);
    }

    pub fn register_function_with_timeout<F>(
        &mut self,
        name: &str,
        route: &str,
        timeout: Duration,
        handler: F,
    ) where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        let entry = FunctionEntry {
            name: name.to_string(),
            route: route.to_string(),
            timeout,
            handler: Box::new(handler),
        };
        self.functions.insert(name.to_string(), entry);
    }

    pub fn pre_warm_function(&mut self, name: &str) {
        if let Some(entry) = self.functions.remove(name) {
            self.pool.pre_warm(entry);
        }
    }

    /// Dispatch an HTTP request to the matching function.
    pub fn handle_request(&mut self, req: &HttpRequest) -> HttpResponse {
        self.request_counter.fetch_add(1, Ordering::Relaxed);

        // Find the function by route match
        let func_name = match self.find_function(&req.path) {
            Some(name) => name,
            None => {
                return HttpResponse::error(404, "Function not found");
            }
        };

        // Record for auto-scaling
        let decision = {
            let scaler = self.scaler.lock().unwrap();
            scaler.record_request(&func_name)
        };
        {
            let mut scaler = self.scaler.lock().unwrap();
            scaler.apply_scale(&decision);
        }

        // Try warm pool first
        if let Some(entry) = self.pool.try_get_warm(&func_name) {
            return entry.invoke(req);
        }

        // Cold-start: find the function, promote from cold pool
        if let Some(entry) = self.pool.promote_cold(&func_name) {
            return entry.invoke(req);
        }

        // Last resort: look up in registry (shouldn't normally happen if pre-warmed)
        if let Some(entry) = self.functions.get(&func_name) {
            return entry.invoke(req);
        }

        HttpResponse::error(500, "Function not available")
    }

    fn find_function(&self, path: &str) -> Option<String> {
        // Simple route matching: exact match first, then prefix match
        for (name, entry) in &self.functions {
            if entry.route == path {
                return Some(name.clone());
            }
        }
        for (name, entry) in &self.functions {
            if path.starts_with(&entry.route) {
                return Some(name.clone());
            }
        }
        None
    }

    pub fn total_requests(&self) -> u64 {
        self.request_counter.load(Ordering::Relaxed)
    }

    pub fn instance_count(&self) -> u32 {
        self.scaler.lock().unwrap().current_instances()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn hello_handler(_req: &HttpRequest) -> HttpResponse {
        HttpResponse::ok(b"{\"message\": \"hello\"}")
    }

    fn echo_handler(req: &HttpRequest) -> HttpResponse {
        let mut headers = HashMap::new();
        headers.insert("x-method".into(), req.method.clone());
        HttpResponse {
            status: 200,
            headers,
            body: req.body.clone(),
        }
    }

    #[test]
    fn http_response_ok() {
        let resp = HttpResponse::ok(b"hi");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"hi");
    }

    #[test]
    fn http_response_error() {
        let resp = HttpResponse::error(404, "not found");
        assert_eq!(resp.status, 404);
        assert_eq!(resp.body, b"not found");
    }

    #[test]
    fn register_and_invoke() {
        let mut runtime = FaasRuntime::new(AutoScalingPolicy::default());
        runtime.register_function("hello", "/api/hello", hello_handler);

        let req = HttpRequest {
            method: "GET".into(),
            path: "/api/hello".into(),
            headers: HashMap::new(),
            body: vec![],
        };
        let resp = runtime.handle_request(&req);
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn not_found_returns_404() {
        let runtime = FaasRuntime::new(AutoScalingPolicy::default());
        let mut rt = runtime;

        let req = HttpRequest {
            method: "GET".into(),
            path: "/nonexistent".into(),
            headers: HashMap::new(),
            body: vec![],
        };
        let resp = rt.handle_request(&req);
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn cold_start_pool_promotes() {
        let mut pool = ColdStartPool::new();
        let entry = FunctionEntry {
            name: "func".into(),
            route: "/func".into(),
            timeout: Duration::from_secs(30),
            handler: Box::new(hello_handler),
        };
        pool.add_cold(entry);

        assert!(pool.try_get_warm("func").is_none());
        let entry = pool.promote_cold("func");
        assert!(entry.is_some());
        assert_eq!(pool.cold_start_count(), 1);

        // Now it should be warm
        assert!(pool.try_get_warm("func").is_some());
        assert_eq!(pool.warm_start_count(), 1);
    }

    #[test]
    fn pre_warm_avoids_cold_start() {
        let mut pool = ColdStartPool::new();
        let entry = FunctionEntry {
            name: "fast".into(),
            route: "/fast".into(),
            timeout: Duration::from_secs(30),
            handler: Box::new(hello_handler),
        };
        pool.pre_warm(entry);

        assert!(pool.try_get_warm("fast").is_some());
        assert_eq!(pool.cold_start_count(), 0);
        assert_eq!(pool.warm_start_count(), 1);
    }

    #[test]
    fn auto_scaler_scales_up() {
        let policy = AutoScalingPolicy {
            min_instances: 1,
            max_instances: 10,
            target_requests_per_instance: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        };
        let mut scaler = AutoScaler::new(policy);
        for _ in 0..9 {
            let _ = scaler.record_request("func");
        }
        let decision = scaler.record_request("func");
        assert!(matches!(decision, ScaleDecision::ScaleUp(2)));
        scaler.apply_scale(&decision);
        assert_eq!(scaler.current_instances(), 2);
    }

    #[test]
    fn auto_scaler_respects_max() {
        let policy = AutoScalingPolicy {
            min_instances: 1,
            max_instances: 3,
            target_requests_per_instance: 1,
            scale_up_threshold: 0.5,
            scale_down_threshold: 0.1,
        };
        let mut scaler = AutoScaler::new(policy);
        // Push to max
        let _ = scaler.record_request("f");
        scaler.apply_scale(&ScaleDecision::ScaleUp(3));
        assert_eq!(scaler.current_instances(), 3);

        // Should not exceed max
        let _ = scaler.record_request("f");
        scaler.apply_scale(&ScaleDecision::ScaleUp(4));
        assert_eq!(scaler.current_instances(), 3);
    }

    #[test]
    fn runtime_counts_requests() {
        let mut runtime = FaasRuntime::new(AutoScalingPolicy::default());
        runtime.register_function("h", "/h", hello_handler);

        let req = HttpRequest {
            method: "GET".into(),
            path: "/h".into(),
            headers: HashMap::new(),
            body: vec![],
        };
        runtime.handle_request(&req);
        runtime.handle_request(&req);
        assert_eq!(runtime.total_requests(), 2);
    }

    #[test]
    fn echo_handler_reflects_body() {
        let mut runtime = FaasRuntime::new(AutoScalingPolicy::default());
        runtime.register_function("echo", "/echo", echo_handler);

        let req = HttpRequest {
            method: "POST".into(),
            path: "/echo".into(),
            headers: HashMap::new(),
            body: b"payload".to_vec(),
        };
        let resp = runtime.handle_request(&req);
        assert_eq!(resp.body, b"payload");
        assert_eq!(resp.headers.get("x-method").unwrap(), "POST");
    }

    #[test]
    fn serialization_roundtrip() {
        let req = HttpRequest {
            method: "GET".into(),
            path: "/test".into(),
            headers: HashMap::new(),
            body: vec![],
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: HttpRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, parsed);
    }
}
