# Chapter 21: Final Project

In this chapter, we'll build a complete application that demonstrates the key concepts covered throughout this book. We'll create a real-time chat application with user authentication, message persistence, and end-to-end encryption.

## Project Overview

### Features

- User registration and authentication
- Real-time messaging
- Message history
- End-to-end encryption
- User presence indicators
- Group chat support

### Architecture

```
src/
├── main.fusion
├── config.fusion
├── models/
│   ├── user.fusion
│   ├── message.fusion
│   └── room.fusion
├── services/
│   ├── auth.fusion
│   ├── messaging.fusion
│   └── encryption.fusion
├── handlers/
│   ├── api.fusion
│   ├── websocket.fusion
│   └── middleware.fusion
├── db/
│   ├── pool.fusion
│   └── migrations/
│       └── 001_initial.sql
└── tests/
    ├── unit/
    └── integration/
```

## Step 1: Project Setup

### Initialize Project

```bash
fusion init chat-app
cd chat-app
```

### Dependencies

```toml
# Fusion.toml
[package]
name = "chat-app"
version = "0.1.0"
edition = "2024"

[dependencies]
std = "2.0"
web = "0.3"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
jsonwebtoken = "9.0"
bcrypt = "0.15"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Step 2: Database Schema

### Migration

```sql
-- migrations/001_initial.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_private BOOLEAN DEFAULT FALSE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID REFERENCES rooms(id),
    user_id UUID REFERENCES users(id),
    content TEXT NOT NULL,
    encrypted_content BYTEA,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE room_members (
    room_id UUID REFERENCES rooms(id),
    user_id UUID REFERENCES users(id),
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (room_id, user_id)
);
```

## Step 3: Models

### User Model

```fusion
// models/user.fusion
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub fn new(new_user: NewUser) -> Result<Self, AuthError> {
        let password_hash = bcrypt::hash(&new_user.password, 12)?;
        
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            username: new_user.username,
            email: new_user.email,
            password_hash,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }
}
```

### Message Model

```fusion
// models/message.fusion
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: uuid::Uuid,
    pub room_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub content: String,
    pub encrypted_content: Option<Vec<u8>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewMessage {
    pub room_id: uuid::Uuid,
    pub content: String,
}

impl Message {
    pub fn new(new_message: NewMessage, user_id: uuid::Uuid) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            room_id: new_message.room_id,
            user_id,
            content: new_message.content,
            encrypted_content: None,
            created_at: chrono::Utc::now(),
        }
    }
}
```

### Room Model

```fusion
// models/room.fusion
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Room {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub created_by: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewRoom {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
}

impl Room {
    pub fn new(new_room: NewRoom, creator_id: uuid::Uuid) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: new_room.name,
            description: new_room.description,
            is_private: new_room.is_private.unwrap_or(false),
            created_by: creator_id,
            created_at: chrono::Utc::now(),
        }
    }
}
```

## Step 4: Services

### Authentication Service

```fusion
// services/auth.fusion
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
    
    pub fn generate_token(&self, user: &User) -> Result<String, AuthError> {
        let claims = Claims {
            sub: user.id.to_string(),
            exp: chrono::Utc::now() + chrono::Duration::hours(24),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;
        
        Ok(token)
    }
    
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        
        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: chrono::DateTime<chrono::Utc>,
}
```

### Messaging Service

```fusion
// services/messaging.fusion
use sqlx::PgPool;

pub struct MessagingService {
    pool: PgPool,
}

impl MessagingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn send_message(&self, new_message: NewMessage, user_id: uuid::Uuid) -> Result<Message, MessageError> {
        let message = Message::new(new_message, user_id);
        
        sqlx::query(
            "INSERT INTO messages (id, room_id, user_id, content, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(message.id)
        .bind(message.room_id)
        .bind(message.user_id)
        .bind(&message.content)
        .bind(message.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(message)
    }
    
    pub async fn get_room_messages(&self, room_id: uuid::Uuid, limit: i64) -> Result<Vec<Message>, MessageError> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE room_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(room_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(messages)
    }
    
    pub async fn join_room(&self, room_id: uuid::Uuid, user_id: uuid::Uuid) -> Result<(), MessageError> {
        sqlx::query(
            "INSERT INTO room_members (room_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(room_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### Encryption Service

```fusion
// services/encryption.fusion
use std::crypto::{KeyPair, encrypt, decrypt};

pub struct EncryptionService {
    key_pair: KeyPair,
}

impl EncryptionService {
    pub fn new() -> Self {
        let key_pair = KeyPair::generate();
        Self { key_pair }
    }
    
    pub fn encrypt_message(&self, message: &str, recipient_public_key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let recipient_key = PublicKey::from_bytes(recipient_public_key)?;
        let shared_secret = self.key_pair.diffie_hellman(&recipient_key);
        
        let encrypted = encrypt(
            message.as_bytes(),
            &shared_secret,
            &self.key_pair.nonce(),
        )?;
        
        Ok(encrypted)
    }
    
    pub fn decrypt_message(&self, encrypted: &[u8], sender_public_key: &[u8]) -> Result<String, EncryptionError> {
        let sender_key = PublicKey::from_bytes(sender_public_key)?;
        let shared_secret = self.key_pair.diffie_hellman(&sender_key);
        
        let decrypted = decrypt(
            encrypted,
            &shared_secret,
            &self.key_pair.nonce(),
        )?;
        
        String::from_utf8(decrypted).map_err(EncryptionError::Utf8)
    }
    
    pub fn public_key(&self) -> Vec<u8> {
        self.key_pair.public_key().to_bytes().to_vec()
    }
}
```

## Step 5: Handlers

### API Handlers

```fusion
// handlers/api.fusion
use web::{get, post, Json, Path, State};

pub async fn register_user(
    State(app): State<AppState>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, ApiError> {
    let user = User::new(new_user)?;
    
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(user.created_at)
    .bind(user.updated_at)
    .execute(&app.pool)
    .await?;
    
    Ok(Json(user))
}

pub async fn login_user(
    State(app): State<AppState>,
    Json(login): Json<LoginUser>,
) -> Result<Json<AuthToken>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&login.email)
    .fetch_optional(&app.pool)
    .await?
    .ok_or(ApiError::UserNotFound)?;
    
    if !user.verify_password(&login.password) {
        return Err(ApiError::InvalidCredentials);
    }
    
    let token = app.auth_service.generate_token(&user)?;
    
    Ok(Json(AuthToken { token }))
}

pub async fn get_rooms(
    State(app): State<AppState>,
) -> Result<Json<Vec<Room>>, ApiError> {
    let rooms = sqlx::query_as::<_, Room>(
        "SELECT * FROM rooms WHERE is_private = FALSE"
    )
    .fetch_all(&app.pool)
    .await?;
    
    Ok(Json(rooms))
}

pub async fn create_room(
    State(app): State<AppState>,
    user: AuthUser,
    Json(new_room): Json<NewRoom>,
) -> Result<Json<Room>, ApiError> {
    let room = Room::new(new_room, user.id);
    
    sqlx::query(
        "INSERT INTO rooms (id, name, description, is_private, created_by, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(room.id)
    .bind(&room.name)
    .bind(&room.description)
    .bind(room.is_private)
    .bind(room.created_by)
    .bind(room.created_at)
    .execute(&app.pool)
    .await?;
    
    // Auto-join creator
    app.messaging_service.join_room(room.id, user.id).await?;
    
    Ok(Json(room))
}
```

### WebSocket Handler

```fusion
// handlers/websocket.fusion
use web::ws::{WebSocket, Message as WsMessage};

pub async fn websocket_handler(
    ws: WebSocket,
    user: AuthUser,
    State(app): State<AppState>,
) {
    let (mut sender, mut receiver) = ws.split();
    
    // Send welcome message
    let welcome = serde_json::to_string(&WsMessage::Connected {
        user_id: user.id,
        username: user.username.clone(),
    }).unwrap();
    
    sender.send(WsMessage::Text(welcome)).await.ok();
    
    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(WsMessage::Text(text)) => {
                if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match msg {
                        ClientMessage::JoinRoom { room_id } => {
                            app.messaging_service.join_room(room_id, user.id).await.ok();
                            
                            // Notify room
                            let notification = serde_json::to_string(&WsMessage::UserJoined {
                                room_id,
                                user_id: user.id,
                                username: user.username.clone(),
                            }).unwrap();
                            
                            app.broadcast_to_room(room_id, &notification).await;
                        }
                        ClientMessage::SendMessage { room_id, content } => {
                            let new_message = NewMessage { room_id, content };
                            let message = app.messaging_service.send_message(new_message, user.id).await;
                            
                            if let Ok(msg) = message {
                                let ws_msg = serde_json::to_string(&WsMessage::NewMessage {
                                    room_id,
                                    message: msg,
                                }).unwrap();
                                
                                app.broadcast_to_room(room_id, &ws_msg).await;
                            }
                        }
                        ClientMessage::LeaveRoom { room_id } => {
                            app.messaging_service.leave_room(room_id, user.id).await.ok();
                            
                            let notification = serde_json::to_string(&WsMessage::UserLeft {
                                room_id,
                                user_id: user.id,
                                username: user.username.clone(),
                            }).unwrap();
                            
                            app.broadcast_to_room(room_id, &notification).await;
                        }
                    }
                }
            }
            Ok(WsMessage::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
    
    // Handle disconnect
    app.handle_disconnect(user.id).await;
}
```

### Middleware

```fusion
// handlers/middleware.fusion
use web::{Request, Response, Next};

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = request.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(ApiError::MissingToken)?;
    
    let claims = request.state()
        .auth_service
        .verify_token(token)?;
    
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| ApiError::InvalidToken)?;
    
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&request.state().pool)
    .await?
    .ok_or(ApiError::UserNotFound)?;
    
    request.extensions_mut().insert(AuthUser {
        id: user.id,
        username: user.username,
    });
    
    Ok(next.run(request).await)
}

pub async fn cors_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if request.method() == "OPTIONS" {
        let response = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
            .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
            .body(())
            .unwrap();
        
        return Ok(response);
    }
    
    let mut response = next.run(request).await?;
    
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        "*".parse().unwrap(),
    );
    
    Ok(response)
}
```

## Step 6: Main Application

```fusion
// main.fusion
use web::{App, HttpServer, State};

struct AppState {
    pool: sqlx::PgPool,
    auth_service: AuthService,
    messaging_service: MessagingService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = config::load()?;
    
    // Setup database
    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    
    // Setup services
    let auth_service = AuthService::new(config.jwt_secret);
    let messaging_service = MessagingService::new(pool.clone());
    
    let app_state = AppState {
        pool,
        auth_service,
        messaging_service,
    };
    
    // Setup routes
    let app = App::new()
        .state(app_state)
        .wrap(cors_middleware)
        .route("/api/register", post(register_user))
        .route("/api/login", post(login_user))
        .route("/api/rooms", get(get_rooms).post(create_room))
        .route("/api/rooms/{room_id}/messages", get(get_messages))
        .route("/ws", get(websocket_handler));
    
    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    println!("Server running on {}", addr);
    
    HttpServer::new(app)
        .bind(&addr)?
        .run()
        .await?;
    
    Ok(())
}
```

## Step 7: Testing

### Unit Tests

```fusion
// tests/unit/user_test.fusion
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let new_user = NewUser {
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "securepassword123".into(),
        };
        
        let user = User::new(new_user).unwrap();
        
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.verify_password("securepassword123"));
        assert!(!user.verify_password("wrongpassword"));
    }
    
    #[test]
    fn test_message_creation() {
        let new_message = NewMessage {
            room_id: uuid::Uuid::new_v4(),
            content: "Hello, world!".into(),
        };
        
        let user_id = uuid::Uuid::new_v4();
        let message = Message::new(new_message, user_id);
        
        assert_eq!(message.content, "Hello, world!");
        assert_eq!(message.user_id, user_id);
    }
}
```

### Integration Tests

```fusion
// tests/integration/api_test.fusion
#[tokio::test]
async fn test_register_and_login() {
    let app = setup_test_app().await;
    
    // Register
    let new_user = NewUser {
        username: "testuser".into(),
        email: "test@example.com".into(),
        password: "securepassword123".into(),
    };
    
    let response = app.post("/api/register")
        .json(&new_user)
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    let user: User = response.json().await.unwrap();
    assert_eq!(user.username, "testuser");
    
    // Login
    let login = LoginUser {
        email: "test@example.com".into(),
        password: "securepassword123".into(),
    };
    
    let response = app.post("/api/login")
        .json(&login)
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    let token: AuthToken = response.json().await.unwrap();
    assert!(!token.token.is_empty());
}
```

## Step 8: Deployment

### Dockerfile

```dockerfile
FROM fusion:2.0 as builder

WORKDIR /app
COPY . .
RUN fusion build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates

COPY --from=builder /app/target/release/chat-app /usr/local/bin/

EXPOSE 8080
CMD ["chat-app"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/chatdb
      - JWT_SECRET=your-secret-key
    depends_on:
      - db
  
  db:
    image: postgres:15
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=chatdb
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## Summary

In this final project, we built a complete chat application demonstrating:

1. **Project Structure**: Organized code into modules, services, and handlers
2. **Database Integration**: Used SQLx for type-safe database access
3. **Authentication**: Implemented JWT-based authentication
4. **Real-time Communication**: Used WebSockets for live messaging
5. **Encryption**: Added end-to-end encryption support
6. **Testing**: Wrote unit and integration tests
7. **Deployment**: Created Docker configuration for deployment

This project showcases how Fusion's features—memory safety, strong typing, async/await, and ecosystem libraries—come together to build production-ready applications.

Congratulations on completing this guide! You now have the knowledge to build secure, efficient, and maintainable applications with Fusion.