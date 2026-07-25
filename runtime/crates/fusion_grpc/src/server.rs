//! gRPC server — request handling, service registration, and response building.

use crate::metadata::{MetadataKey, MetadataMap, MetadataValue};
use crate::service::ServiceDefinition;
use crate::status::GrpcStatus;
use bytes::Bytes;

/// A gRPC server that holds registered service definitions.
pub struct GrpcServer {
    services: Vec<ServiceDefinition>,
}

impl GrpcServer {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn add_service(mut self, service: ServiceDefinition) -> Self {
        self.services.push(service);
        self
    }

    pub fn services(&self) -> &[ServiceDefinition] {
        &self.services
    }

    /// Route an incoming request to the appropriate service/method.
    pub fn handle_request(
        &self,
        path: &str,
        body: &[u8],
    ) -> (GrpcStatus, Vec<u8>) {
        // Parse path: /ServiceName/version/MethodName
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        if parts.len() < 2 {
            return (
                GrpcStatus::invalid_argument("Invalid gRPC path"),
                Vec::new(),
            );
        }

        let service_name = parts[0];
        let method_name = parts.last().unwrap();

        for svc in &self.services {
            if svc.descriptor.name == service_name || parts.len() >= 2 {
                match svc.handle(method_name, body) {
                    Ok(response) => return (GrpcStatus::ok(), response),
                    Err(e) => return (GrpcStatus::internal(e), Vec::new()),
                }
            }
        }

        (
            GrpcStatus::unimplemented(format!("Service not found: {}", service_name)),
            Vec::new(),
        )
    }

    /// Build the standard gRPC response headers.
    pub fn response_headers() -> MetadataMap {
        let mut meta = MetadataMap::new();
        meta.insert(
            MetadataKey::new("content-type"),
            MetadataValue::from_str("application/grpc"),
        );
        meta.insert(
            MetadataKey::new("grpc-accept-encoding"),
            MetadataValue::from_str("identity"),
        );
        meta
    }
}

impl Default for GrpcServer {
    fn default() -> Self {
        Self::new()
    }
}

/// A gRPC request received by the server.
#[derive(Debug, Clone)]
pub struct GrpcRequest {
    pub path: String,
    pub metadata: MetadataMap,
    pub body: Bytes,
    pub compression: bool,
}

/// A gRPC response to send back.
#[derive(Debug, Clone)]
pub struct GrpcResponse {
    pub status: GrpcStatus,
    pub metadata: MetadataMap,
    pub body: Bytes,
}

impl GrpcResponse {
    pub fn ok(body: Bytes) -> Self {
        Self {
            status: GrpcStatus::ok(),
            metadata: MetadataMap::new(),
            body,
        }
    }

    pub fn error(status: GrpcStatus) -> Self {
        Self {
            status,
            metadata: MetadataMap::new(),
            body: Bytes::new(),
        }
    }
}

/// Trait for implementing gRPC service handlers.
pub trait GrpcService: Send + Sync {
    /// The service name as it appears in gRPC paths.
    fn service_name(&self) -> &str;

    /// Handle an RPC call and return a response.
    fn handle_method(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, GrpcStatus>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::GrpcCode;
    use crate::service::{ServiceDefinition, ServiceDescriptor};

    fn test_service() -> ServiceDefinition {
        ServiceDefinition::new(ServiceDescriptor::new("Echo"))
            .register_handler("Echo", |input| Ok(input.to_vec()))
    }

    #[test]
    fn test_server_handle_request() {
        let server = GrpcServer::new().add_service(test_service());
        let (status, body) = server.handle_request("/Echo/v1/Echo", b"hello");
        assert!(status.is_ok());
        assert_eq!(body, b"hello");
    }

    #[test]
    fn test_server_unknown_service() {
        let server = GrpcServer::new();
        let (status, _) = server.handle_request("/Unknown/v1/Method", b"");
        assert_eq!(status.code, GrpcCode::Unimplemented);
    }

    #[test]
    fn test_server_response_headers() {
        let headers = GrpcServer::response_headers();
        assert_eq!(
            headers.get("content-type").unwrap().to_str().unwrap(),
            "application/grpc"
        );
    }

    #[test]
    fn test_grpc_request_response() {
        let req = GrpcRequest {
            path: "/Echo/v1/Echo".to_string(),
            metadata: MetadataMap::new(),
            body: Bytes::from("test"),
            compression: false,
        };

        let resp = GrpcResponse::ok(Bytes::from("response"));
        assert!(resp.status.is_ok());
    }
}
