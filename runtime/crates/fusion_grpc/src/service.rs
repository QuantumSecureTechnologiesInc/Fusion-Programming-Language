//! gRPC service definition types — protobuf-style service descriptors and method metadata.

/// Describes a single RPC method within a gRPC service.
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    pub name: String,
    pub service: String,
    pub method_type: MethodType,
    pub input_type: String,
    pub output_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodType {
    Unary,
    ServerStreaming,
    ClientStreaming,
    BidirectionalStreaming,
}

impl MethodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unary => "unary",
            Self::ServerStreaming => "server_streaming",
            Self::ClientStreaming => "client_streaming",
            Self::BidirectionalStreaming => "bidirectional_streaming",
        }
    }
}

/// Describes a gRPC service and its methods.
#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub name: String,
    pub version: String,
    pub methods: Vec<MethodDescriptor>,
}

impl ServiceDescriptor {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "v1".to_string(),
            methods: Vec::new(),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn method(mut self, method: MethodDescriptor) -> Self {
        self.methods.push(method);
        self
    }

    pub fn full_name(&self) -> String {
        format!("/{}/{}", self.name, self.version)
    }

    pub fn method_path(&self, method_name: &str) -> String {
        format!("/{}/{}/{}", self.name, self.version, method_name)
    }
}

/// Service definition with handler registration.
pub struct ServiceDefinition {
    pub descriptor: ServiceDescriptor,
    handlers: Vec<MethodHandler>,
}

struct MethodHandler {
    method_name: String,
    handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, String> + Send + Sync>,
}

impl ServiceDefinition {
    pub fn new(descriptor: ServiceDescriptor) -> Self {
        Self {
            descriptor,
            handlers: Vec::new(),
        }
    }

    pub fn register_handler(
        mut self,
        method_name: &str,
        handler: impl Fn(&[u8]) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(MethodHandler {
            method_name: method_name.to_string(),
            handler: Box::new(handler),
        });
        self
    }

    pub fn handle(&self, method_name: &str, input: &[u8]) -> Result<Vec<u8>, String> {
        self.handlers
            .iter()
            .find(|h| h.method_name == method_name)
            .map(|h| (h.handler)(input))
            .unwrap_or_else(|| Err(format!("Method {} not found", method_name)))
    }

    pub fn method_names(&self) -> Vec<&str> {
        self.handlers.iter().map(|h| h.method_name.as_str()).collect()
    }
}

/// Registrar for building a service definition incrementally.
pub struct ServiceRegistrar {
    descriptor: ServiceDescriptor,
}

impl ServiceRegistrar {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            descriptor: ServiceDescriptor::new(service_name),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.descriptor = self.descriptor.version(version);
        self
    }

    pub fn add_unary(
        mut self,
        name: &str,
        input_type: &str,
        output_type: &str,
    ) -> Self {
        self.descriptor.methods.push(MethodDescriptor {
            name: name.to_string(),
            service: self.descriptor.name.clone(),
            method_type: MethodType::Unary,
            input_type: input_type.to_string(),
            output_type: output_type.to_string(),
        });
        self
    }

    pub fn add_server_streaming(
        mut self,
        name: &str,
        input_type: &str,
        output_type: &str,
    ) -> Self {
        self.descriptor.methods.push(MethodDescriptor {
            name: name.to_string(),
            service: self.descriptor.name.clone(),
            method_type: MethodType::ServerStreaming,
            input_type: input_type.to_string(),
            output_type: output_type.to_string(),
        });
        self
    }

    pub fn add_bidirectional(
        mut self,
        name: &str,
        input_type: &str,
        output_type: &str,
    ) -> Self {
        self.descriptor.methods.push(MethodDescriptor {
            name: name.to_string(),
            service: self.descriptor.name.clone(),
            method_type: MethodType::BidirectionalStreaming,
            input_type: input_type.to_string(),
            output_type: output_type.to_string(),
        });
        self
    }

    pub fn build(self) -> ServiceDescriptor {
        self.descriptor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_descriptor() {
        let svc = ServiceDescriptor::new("UserService")
            .version("v2")
            .method(MethodDescriptor {
                name: "GetUser".to_string(),
                service: "UserService".to_string(),
                method_type: MethodType::Unary,
                input_type: "GetUserRequest".to_string(),
                output_type: "User".to_string(),
            });

        assert_eq!(svc.full_name(), "/UserService/v2");
        assert_eq!(svc.method_path("GetUser"), "/UserService/v2/GetUser");
    }

    #[test]
    fn test_service_registrar() {
        let desc = ServiceRegistrar::new("Echo")
            .add_unary("Echo", "EchoRequest", "EchoResponse")
            .add_server_streaming("StreamEcho", "EchoRequest", "EchoResponse")
            .build();

        assert_eq!(desc.methods.len(), 2);
        assert_eq!(desc.methods[0].method_type, MethodType::Unary);
        assert_eq!(desc.methods[1].method_type, MethodType::ServerStreaming);
    }

    #[test]
    fn test_service_definition_handler() {
        let svc = ServiceDefinition::new(ServiceDescriptor::new("Math"))
            .register_handler("Add", |input| {
                let data: Vec<u8> = input.to_vec();
                Ok(data)
            });

        assert!(svc.handle("Add", &[1, 2, 3]).is_ok());
        assert!(svc.handle("Subtract", &[1, 2]).is_err());
    }
}
