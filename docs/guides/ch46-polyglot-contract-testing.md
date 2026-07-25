# Chapter 46: Contract Testing & Mocking Across Boundaries

In a polyglot system, the most dangerous assumption is that two services agree on what their interface looks like. They might agree today and break tomorrow. Contract testing catches these breaks before they reach production. This chapter covers how to define, verify, and evolve contracts across language boundaries.

## Why Contract Testing Matters

Unit tests verify that a single module works correctly. Integration tests verify that two modules work together. Contract tests verify that two modules *agree on the shape of their communication*. In polyglot systems, this distinction is critical because:

1. Service A (Python) and Service B (Rust) are built by different teams
2. They communicate over HTTP with JSON payloads
3. Service A's Python model has `user_id: int` but Service B's Rust struct has `user_id: String`
4. Both services pass their own unit tests
5. In production, every request fails with a deserialization error

Contract testing would catch this in CI.

## Consumer-Driven Contract Testing (Pact)

Pact is the most widely used contract testing framework. The key insight: the *consumer* (the service making a request) defines what it expects, and the *provider* (the service handling the request) verifies it can fulfill that contract.

### How Pact Works

```
Step 1: Consumer writes a test describing what it expects
        "When I send POST /users with {name, email, roles}",
        "I expect back {id, name, email, roles, created_at}"

Step 2: Pact generates a contract file (JSON)
        This is the "agreement" between consumer and provider

Step 3: Provider runs pact-verify against its own implementation
        "Can I actually produce what the consumer expects?"

Step 4: If verification passes → contract is satisfied
        If it fails → build fails, developer fixes the mismatch
```

### Python Consumer Example

```python
# tests/contracts/test_user_api_consumer.py
"""Consumer contract tests for the User API.

These tests define what the Python service EXPECTS from the Rust service.
They do not test the Rust service directly — they define the contract.
"""
from pact import Consumer, Provider
import requests

pact = Consumer("python-user-service").has_pact_with(
    Provider("rust-user-service"),
    pact_dir="pacts",
    log_dir="logs/pacts",
)

def test_get_user():
    """Contract: GET /users/{id} returns a UserRecord."""
    expected_user = {
        "id": 123,
        "name": "Alice",
        "email": "alice@example.com",
        "roles": ["admin", "user"],
        "metadata": None,
        "created_at": "2024-01-15T10:30:00Z",
        "version": 2,
    }

    pact.given("a user with id 123 exists")
    pact.upon_receiving("a request for user 123")
    pact.with_request("get", "/users/123")
    pact.will_respond_with(200, body=expected_user)

    with pact:
        response = requests.get(f"{pact.uri}/users/123")
        assert response.status_code == 200
        user = response.json()
        assert user["id"] == 123
        assert user["name"] == "Alice"
        assert "admin" in user["roles"]

def test_create_user():
    """Contract: POST /users with valid data returns created user."""
    request_body = {
        "name": "Bob",
        "email": "bob@example.com",
        "roles": ["user"],
    }
    expected_response = {
        "id": 456,
        "name": "Bob",
        "email": "bob@example.com",
        "roles": ["user"],
        "created_at": "2024-01-15T10:30:00Z",
        "version": 1,
    }

    pact.given("the system is ready to create users")
    pact.upon_receiving("a request to create a user")
    pact.with_request("post", "/users", body=request_body)
    pact.will_respond_with(201, body=expected_response)

    with pact:
        response = requests.post(f"{pact.uri}/users", json=request_body)
        assert response.status_code == 201
        assert response.json()["name"] == "Bob"
```

### Rust Provider Verification

```rust
// tests/contracts/test_user_api_provider.rs
//! Provider verification tests.
//! These tests verify that the Rust service fulfills the Python consumer's contract.

use pact_consumer::prelude::*;
use serde_json::json;

#[test]
fn verify_pact_against_provider() {
    let config = PactBuilder::new("python-user-service", "rust-user-service")
        .given("a user with id 123 exists")
        .upon_receiving("a request for user 123")
        .method("GET")
        .path("/users/123")
        .respond_with()
        .status(200)
        .json_body(json!({
            "id": 123,
            "name": "Alice",
            "email": "alice@example.com",
            "roles": ["admin", "user"],
            "metadata": null,
            "created_at": "2024-01-15T10:30:00Z",
            "version": 2
        }))
        .build();

    let server = config.create_server(0);
    let server_addr = server.addr();

    // Your actual server code
    let app = fusion_api::create_app();
    let listener = std::net::TcpListener::bind(server_addr).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Pact verifies the contract
    let result = pact_consumer::verify(
        &config.pact,
        &server_addr.to_string(),
    );
    assert!(result.is_ok(), "Provider verification failed: {:?}", result.err());
}
```

### Go Consumer and Provider

```go
// tests/contracts/consumer_test.go
package contracts_test

import (
    "encoding/json"
    "net/http"
    "testing"

    "github.com/pact-foundation/pact-go/v2/consumer"
    "github.com/pact-foundation/pact-go/v2/matchers"
)

func TestGetUserContract(t *testing.T) {
    mockProvider, err := consumer.NewV2Pact(consumer.MockHTTPProviderConfig{
        Consumer: "go-user-service",
        Provider: "rust-user-service",
        PactDir:  "./pacts",
    })
    if err != nil {
        t.Fatal(err)
    }

    expected := consumer.MapMatcher{
        "id":         matchers.Like(123),
        "name":       matchers.Like("Alice"),
        "email":      matchers.Like("alice@example.com"),
        "roles":      matchers.EachLike(matchers.Like("admin"), 1),
        "metadata":   matchers.Null(),
        "created_at": matchers.Like("2024-01-15T10:30:00Z"),
        "version":    matchers.Like(2),
    }

    err = mockProvider.
        Given("a user with id 123 exists").
        AddInteraction().
        UponReceiving("a request for user 123").
        WithRequest("GET", "/users/123").
        WillRespondWith(200, consumer.MapBody(expected)).
        ExecuteTest(t, func(config consumer.MockServerConfig) error {
            resp, err := http.Get(fmt.Sprintf("http://%s:%d/users/123", config.Host, config.Port))
            if err != nil {
                return err
            }
            defer resp.Body.Close()

            var user UserRecord
            json.NewDecoder(resp.Body).Decode(&user)
            assert.Equal(t, "Alice", user.Name)
            return nil
        })
    if err != nil {
        t.Fatal(err)
    }
}
```

## Mocking Foreign Services Without Full Runtime

You often need to test a module that calls a foreign service, but you don't want to spin up the entire foreign runtime. The solution: mock the boundary with a lightweight server that responds with the expected wire format.

### The Boundary Mock Pattern

```python
# tests/mocks/mock_rust_service.py
"""Lightweight mock of the Rust service for Python integration tests.

This mock returns the exact wire format the Rust service would return,
without requiring the Rust binary to be compiled.
"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import threading
import pytest

class MockRustHandler(BaseHTTPRequestHandler):
    """Handles requests in the exact wire format of the Rust service."""

    def do_GET(self):
        if self.path.startswith("/users/"):
            user_id = int(self.path.split("/")[-1])
            response = {
                "id": user_id,
                "name": f"User_{user_id}",
                "email": f"user{user_id}@example.com",
                "roles": ["user"],
                "created_at": "2024-01-15T00:00:00Z",
                "version": 1,
            }
            self._send_json(200, response)
        else:
            self._send_json(404, {"error": "not_found", "message": "endpoint not found"})

    def do_POST(self):
        if self.path == "/users":
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))

            # Validate exactly like the Rust service would
            errors = []
            if not body.get("name"):
                errors.append({"field": "name", "message": "name must be non-empty"})
            if "@" not in body.get("email", ""):
                errors.append({"field": "email", "message": "invalid email"})
            if not body.get("roles"):
                errors.append({"field": "roles", "message": "roles must have at least one entry"})

            if errors:
                self._send_json(422, {"error": "validation", "errors": errors})
            else:
                response = {
                    "id": 999,
                    "name": body["name"],
                    "email": body["email"],
                    "roles": body["roles"],
                    "created_at": "2024-01-15T00:00:00Z",
                    "version": 1,
                }
                self._send_json(201, response)
        else:
            self._send_json(404, {"error": "not_found"})

    def _send_json(self, status, data):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def log_message(self, format, *args):
        pass  # Suppress logging during tests

@pytest.fixture
def mock_rust_service():
    """Fixture that starts a mock Rust service on a random port."""
    server = HTTPServer(("127.0.0.1", 0), MockRustHandler)
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever)
    thread.daemon = True
    thread.start()
    yield f"http://127.0.0.1:{port}"
    server.shutdown()

# Usage in tests
def test_python_client_with_mock(mock_rust_service):
    from fusion.client import UserClient

    client = UserClient(base_url=mock_rust_service)
    user = client.get_user(123)

    assert user["name"] == "User_123"
    assert user["email"] == "user123@example.com"
```

### Mocking with gRPC Service Stubs

```protobuf
// proto/user.proto
syntax = "proto3";
package fusion.user;

message UserRequest {
    int64 id = 1;
}

message UserResponse {
    int64 id = 1;
    string name = 2;
    string email = 3;
    repeated string roles = 4;
    string created_at = 5;
    int32 version = 6;
}

service UserService {
    rpc GetUser(UserRequest) returns (UserResponse);
    rpc CreateUser(CreateUserRequest) returns (UserResponse);
}
```

```go
// tests/mocks/grpc_mock.go
package mocks

import (
    "context"
    "testing"

    "google.golang.org/grpc"
    "google.golang.org/grpc/test/bufconn"
    pb "fusion/proto/user"
)

// MockUserServiceServer implements the gRPC UserService for testing.
type MockUserServiceServer struct {
    pb.UnimplementedUserServiceServer
    Users map[int64]*pb.UserResponse
}

func (m *MockUserServiceServer) GetUser(ctx context.Context, req *pb.UserRequest) (*pb.UserResponse, error) {
    user, ok := m.Users[req.Id]
    if !ok {
        return nil, fmt.Errorf("user %d not found", req.Id)
    }
    return user, nil
}

// StartMockGRPC creates an in-memory gRPC server for testing.
func StartMockGRPC(t *testing.T) *grpc.ClientConn {
    lis := bufconn.Listen(1024 * 1024)

    server := grpc.NewServer()
    mock := &MockUserServiceServer{
        Users: map[int64]*pb.UserResponse{
            123: {Id: 123, Name: "Alice", Email: "alice@test.com", Roles: []string{"admin"}},
        },
    }
    pb.RegisterUserServiceServer(server, mock)

    go server.Serve(lis)

    conn, err := grpc.DialContext(
        context.Background(),
        "bufnet",
        grpc.WithContextDialer(func(ctx context.Context, addr string) (net.Conn, error) {
            return lis.Dial()
        }),
        grpc.WithInsecure(),
    )
    if err != nil {
        t.Fatal(err)
    }

    t.Cleanup(func() {
        conn.Close()
        server.Stop()
    })

    return conn
}
```

## gRPC/Protobuf Stub Generation

Protobuf is the natural choice for polyglot service contracts because it generates code in every language from a single schema definition.

### Generating Stubs for All Languages

```yaml
# .github/workflows/proto-generate.yml
name: Generate Protobuf Stubs
on:
  push:
    paths: ['proto/**']

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install protobuf compiler
        run: |
          curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v25.1/protoc-25.1-linux-x86_64.zip
          unzip protoc-25.1-linux-x86_64.zip -d /usr/local

      # Generate Rust stubs
      - name: Install protoc-gen-prost
        run: cargo install prost-build
      - name: Generate Rust
        run: |
          mkdir -p src/proto
          protoc --rust_out=src/proto proto/user.proto

      # Generate Python stubs
      - name: Generate Python
        run: |
          pip install grpcio-tools
          python -m grpc_tools.protoc -I proto --python_out=bindings/python --grpc_python_out=bindings/python proto/user.proto

      # Generate Go stubs
      - name: Generate Go
        run: |
          go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
          go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
          protoc --go_out=bindings/go --go-grpc_out=bindings/go proto/user.proto

      # Generate TypeScript stubs
      - name: Generate TypeScript
        run: |
          npm install -g protobufjs-cli
          protoc --js_out=bindings/js --grpc-web_out=bindings/js proto/user.proto

      # Commit generated files
      - name: Commit stubs
        run: |
          git config user.name "github-actions"
          git config user.email "actions@github.com"
          git add src/proto/ bindings/
          git diff --staged --quiet || git commit -m "chore: regenerate protobuf stubs"
          git push
```

### Schema Evolution Strategy

```protobuf
// proto/user.proto — Versioned schema with backward compatibility rules

syntax = "proto3";
package fusion.user;

// Rules:
// 1. NEVER remove a field
// 2. NEVER change a field's number
// 3. NEVER change a field's type (unless wire-compatible)
// 4. NEW fields MUST use a new field number
// 5. DEPRECATED fields get a comment but keep their number
// 6. Each schema change gets a MINOR version bump

message UserRecord {
    // v1 fields — NEVER remove or renumber
    int64 id = 1;
    string name = 2;
    string email = 3;
    repeated string roles = 4;

    // v1.1 fields
    map<string, string> metadata = 5;

    // v1.2 fields
    string created_at = 6;
    int32 version = 7;

    // v1.3 fields
    string phone = 8;

    // DEPRECATED: Use phone instead. Kept for backward compatibility.
    // Field number 9 is permanently reserved.
    reserved 9;

    // v2.0 fields (planned)
    // int64 organization_id = 10;  // Reserved for future use
    reserved 10;
}

// Schema version tracking
message SchemaMeta {
    string schema_version = 1;   // "2.0"
    string min_compatible = 2;   // "1.0" — oldest version this can decode
    string generated_at = 3;     // ISO 8601
}
```

## Contract Verification in CI

Every PR that modifies a service's API should run contract verification. If the API change breaks a consumer's contract, the build should fail.

```yaml
# .github/workflows/contract-verification.yml
name: Contract Verification
on:
  pull_request:
    paths:
      - 'src/**'
      - 'proto/**'
      - 'bindings/**'

jobs:
  verify-contracts:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        language: [python, go, js]

    steps:
      - uses: actions/checkout@v4

      # Pull latest contracts from Pact Broker
      - name: Download contracts
        run: |
          curl -o pacts/python-consumer-rust-provider.json \
            "${{ secrets.PACT_BROKER_URL }}/pacts/provider/rust-user-service/consumer/python-user-service/latest"

      # Verify provider (Rust) against each consumer's contract
      - name: Verify Rust provider against Python contract
        run: |
          cargo test --test pact_provider_verify \
            -- --pact-consumer-python-user-service \
               --pact-provider rust-user-service

      # Verify provider (Rust) against Go contract
      - name: Verify Rust provider against Go contract
        run: |
          cargo test --test pact_provider_verify \
            -- --pact-consumer-go-user-service \
               --pact-provider rust-user-service

  # Publish contracts to Pact Broker after merge
  publish-contracts:
    needs: verify-contracts
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - name: Publish Python contract
        run: |
          curl -X PUT \
            -H "Content-Type: application/json" \
            -d @pacts/python-consumer-rust-provider.json \
            "${{ secrets.PACT_BROKER_URL }}/pacts/provider/rust-user-service/consumer/python-user-service/version/${{ github.sha }}"

      - name: Publish Go contract
        run: |
          curl -X PUT \
            -H "Content-Type: application/json" \
            -d @pacts/go-consumer-rust-provider.json \
            "${{ secrets.PACT_BROKER_URL }}/pacts/provider/rust-user-service/consumer/go-user-service/version/${{ github.sha }}"
```

### Pact Broker Integration

```bash
# Install Pact Broker CLI
brew install pact-foundation/pact/pact-ruby-standalone

# Tag a contract version as deployable
pact-broker create-version-tag \
  --pacticipant "rust-user-service" \
  --version ${{ github.sha }} \
  --tag main

# Check if a deployment is safe (can-i-deploy)
pact-broker can-i-deploy \
  --pacticipant "python-user-service@main" \
  --to-environment production

# Record a deployment
pact-broker record-deployment \
  --pacticipant "rust-user-service" \
  --version ${{ github.sha }} \
  --environment production
```

## Schema Evolution Strategies

When you need to change a data structure that crosses language boundaries, follow these rules:

### The Safe Change Checklist

```markdown
## Safe Changes (no version bump needed)
- Add a new optional field (with a default value)
- Add a new enum variant (at the end)
- Widen a numeric type (i32 → i64)

## Breaking Changes (require version bump + migration)
- Remove a field
- Rename a field
- Change a field's type (string → int)
- Make a required field optional
- Change a field's number (Protobuf)
- Add a required field without a default

## Migration Process for Breaking Changes
1. Add the new field (old field stays)
2. Update producers to write both fields
3. Update consumers to read the new field
4. Add a metric: count reads of the old field
5. Wait until old field reads drop to zero
6. Remove the old field
7. Bump the schema version
```

### Versioned Contract Example

```python
# Schema versioning in Python
class UserRecordV1:
    """Version 1.0 schema."""
    def __init__(self, id: int, name: str, email: str, roles: list[str]):
        self.id = id
        self.name = name
        self.email = email
        self.roles = roles

class UserRecordV2(UserRecordV1):
    """Version 2.0 schema — adds phone, deprecates name in favor of full_name."""
    def __init__(self, id: int, full_name: str, email: str, roles: list[str],
                 phone: str = "", name: str = ""):
        super().__init__(id, name or full_name, email, roles)
        self.full_name = full_name
        self.phone = phone

def decode_user(data: dict) -> UserRecordV1:
    """Decode with backward compatibility."""
    schema_version = data.get("_schema_version", "1.0")

    if schema_version.startswith("2."):
        return UserRecordV2(
            id=data["id"],
            full_name=data.get("full_name", data.get("name", "")),
            email=data["email"],
            roles=data["roles"],
            phone=data.get("phone", ""),
        )
    else:
        return UserRecordV1(
            id=data["id"],
            name=data["name"],
            email=data["email"],
            roles=data["roles"],
        )
```

## The Contract Testing Pyramid

```
          /\
         /  \        E2E tests (slow, expensive, few)
        / E2E\       Full integration with all services
       /------\
      / Contract\    Contract tests (medium speed, high value)
     /  Tests    \   Pact/Protobuf verification
    /--------------\
   /   Unit Tests   \  Unit tests (fast, cheap, many)
  /   (per language) \  Isolated module tests
 /--------------------\
```

The ideal ratio: 70% unit, 25% contract, 5% E2E. Contract tests give you 80% of the confidence of E2E tests at 20% of the cost.

## Summary

Contract testing is the bridge between "it works on my machine" and "it works in production." In a polyglot system, the contract is the only source of truth that all languages can agree on. Use Pact for HTTP/REST contracts, Protobuf for gRPC contracts, and schema evolution strategies to keep everything backward compatible. The investment in contract testing pays for itself the first time it catches a type mismatch in CI instead of a 3 AM incident.
