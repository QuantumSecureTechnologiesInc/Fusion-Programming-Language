# Chapter 23: Network Programming

Fusion provides powerful networking primitives for building network applications. This chapter covers TCP/UDP sockets, HTTP servers, WebSocket applications, and custom protocols.

## TCP/UDP Sockets

### TCP Client

```fusion
use std::net::TcpStream;
use std::io::{Read, Write};

fn tcp_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    
    // Send data
    stream.write_all(b"Hello, server!")?;
    
    // Read response
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[..bytes_read]));
    
    Ok(())
}
```

### TCP Server

```fusion
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn tcp_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080");
    
    for stream in listener.incoming() {
        let stream = stream?;
        
        thread::spawn(move || {
            handle_client(stream);
        });
    }
    
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                println!("Received: {}", message);
                
                // Echo back
                stream.write_all(&buffer[..n]).unwrap();
            }
            Err(e) => {
                println!("Error reading: {}", e);
                break;
            }
        }
    }
}
```

### UDP Sockets

```fusion
use std::net::UdpSocket;

fn udp_client() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:8081")?;
    
    // Send data
    socket.send(b"Hello, server!")?;
    
    // Receive response
    let mut buffer = [0; 1024];
    let bytes_read = socket.recv(&mut buffer)?;
    println!("Received: {}", String::from_utf8_lossy(&buffer[..bytes_read]));
    
    Ok(())
}

fn udp_server() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:8081")?;
    println!("UDP server listening on port 8081");
    
    let mut buffer = [0; 1024];
    
    loop {
        let (bytes_read, src_addr) = socket.recv_from(&mut buffer)?;
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received from {}: {}", src_addr, message);
        
        // Echo back
        socket.send_to(&buffer[..bytes_read], src_addr)?;
    }
}
```

## HTTP Servers

### Basic HTTP Server

```fusion
use std::net::TcpListener;
use std::io::{Read, Write};
use std::collections::HashMap;

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn ok() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: vec![],
        }
    }
    
    fn not_found() -> Self {
        Self {
            status: 404,
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
        }
    }
    
    fn internal_error() -> Self {
        Self {
            status: 500,
            headers: HashMap::new(),
            body: b"Internal Server Error".to_vec(),
        }
    }
    
    fn send(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        write!(stream, "HTTP/1.1 {} {}\r\n", self.status, status_text(self.status))?;
        
        for (key, value) in &self.headers {
            write!(stream, "{}: {}\r\n", key, value)?;
        }
        
        write!(stream, "Content-Length: {}\r\n", self.body.len())?;
        write!(stream, "\r\n")?;
        
        stream.write_all(&self.body)?;
        
        Ok(())
    }
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

fn parse_request(stream: &mut TcpStream) -> Result<HttpRequest, Box<dyn std::error::Error>> {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    
    let mut lines = request.lines();
    
    // Parse request line
    let request_line = lines.next().ok_or("Missing request line")?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or("Missing method")?.to_string();
    let path = parts.next().ok_or("Missing path")?.to_string();
    
    // Parse headers
    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(
                key.trim().to_lowercase(),
                value.trim().to_string(),
            );
        }
    }
    
    Ok(HttpRequest {
        method,
        path,
        headers,
        body: None,
    })
}

fn handle_request(request: &HttpRequest) -> HttpResponse {
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => {
            let mut response = HttpResponse::ok();
            response.body = b"Hello, World!".to_vec();
            response.headers.insert("Content-Type".into(), "text/plain".into());
            response
        }
        ("GET", "/json") => {
            let mut response = HttpResponse::ok();
            response.body = b"{\"message\": \"Hello, JSON!\"}".to_vec();
            response.headers.insert("Content-Type".into(), "application/json".into());
            response
        }
        _ => HttpResponse::not_found(),
    }
}

fn http_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("HTTP server listening on port 8080");
    
    for stream in listener.incoming() {
        let mut stream = stream?;
        
        match parse_request(&mut stream) {
            Ok(request) => {
                let response = handle_request(&request);
                response.send(&mut stream).ok();
            }
            Err(_) => {
                HttpResponse::internal_error().send(&mut stream).ok();
            }
        }
    }
    
    Ok(())
}
```

### Async HTTP Server

```fusion
use async_std::{net::TcpListener, io::ReadExt, io::WriteExt, task};

async fn async_http_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Async HTTP server listening on port 8080");
    
    let mut incoming = listener.incoming();
    
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        
        task::spawn(async move {
            handle_async_client(stream).await;
        });
    }
    
    Ok(())
}

async fn handle_async_client(mut stream: async_std::net::TcpStream) {
    let mut buffer = [0; 4096];
    
    match stream.read(&mut buffer).await {
        Ok(0) => return,
        Ok(_) => {
            let request = String::from_utf8_lossy(&buffer);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!"
            );
            
            stream.write_all(response.as_bytes()).await.ok();
        }
        Err(_) => {}
    }
}
```

## WebSocket Applications

### WebSocket Server

```fusion
use std::net::TcpListener;
use std::io::{Read, Write};
use sha1::{Sha1, Digest};
use base64::Engine;

struct WebSocket {
    stream: TcpStream,
}

impl WebSocket {
    fn accept(stream: TcpStream) -> Result<Self, WebSocketError> {
        let mut ws = Self { stream };
        
        // Read HTTP upgrade request
        let mut buffer = [0; 4096];
        ws.stream.read(&mut buffer)?;
        
        let request = String::from_utf8_lossy(&buffer);
        
        // Extract Sec-WebSocket-Key
        let key = request.lines()
            .find(|line| line.starts_with("Sec-WebSocket-Key:"))
            .and_then(|line| line.split_once(':'))
            .map(|(_, value)| value.trim())
            .ok_or(WebSocketError::MissingKey)?;
        
        // Compute accept key
        let accept_key = compute_accept_key(key);
        
        // Send upgrade response
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );
        
        ws.stream.write_all(response.as_bytes())?;
        
        Ok(ws)
    }
    
    fn send_text(&mut self, message: &str) -> Result<(), WebSocketError> {
        let payload = message.as_bytes();
        let mask = generate_mask();
        
        let mut frame = Vec::new();
        
        // FIN + text opcode
        frame.push(0x81);
        
        // Masked + length
        if payload.len() < 126 {
            frame.push(0x80 | payload.len() as u8);
        } else if payload.len() < 65536 {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
        }
        
        // Mask
        frame.extend_from_slice(&mask);
        
        // Masked payload
        for (i, &byte) in payload.iter().enumerate() {
            frame.push(byte ^ mask[i % 4]);
        }
        
        self.stream.write_all(&frame)?;
        
        Ok(())
    }
    
    fn receive_frame(&mut self) -> Result<Option<String>, WebSocketError> {
        let mut header = [0; 2];
        self.stream.read_exact(&mut header)?;
        
        let opcode = header[0] & 0x0F;
        let masked = header[1] & 0x80 != 0;
        
        if opcode == 0x08 {
            return Ok(None);  // Close frame
        }
        
        let mut length = (header[1] & 0x7F) as usize;
        
        if length == 126 {
            let mut buf = [0; 2];
            self.stream.read_exact(&mut buf)?;
            length = u16::from_be_bytes(buf) as usize;
        } else if length == 127 {
            let mut buf = [0; 8];
            self.stream.read_exact(&mut buf)?;
            length = u64::from_be_bytes(buf) as usize;
        }
        
        let mask = if masked {
            let mut mask = [0; 4];
            self.stream.read_exact(&mut mask)?;
            Some(mask)
        } else {
            None
        };
        
        let mut payload = vec![0; length];
        self.stream.read_exact(&mut payload)?;
        
        if let Some(mask) = mask {
            for (i, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[i % 4];
            }
        }
        
        let message = String::from_utf8(payload).map_err(WebSocketError::Utf8)?;
        
        Ok(Some(message))
    }
}

fn compute_accept_key(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-5AB905DC3297");
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

fn generate_mask() -> [u8; 4] {
    let mut mask = [0; 4];
    // Use random number generator in production
    mask[0] = 0x12;
    mask[1] = 0x34;
    mask[2] = 0x56;
    mask[3] = 0x78;
    mask
}
```

### WebSocket Client

```fusion
struct WebSocketClient {
    ws: WebSocket,
}

impl WebSocketClient {
    fn connect(url: &str) -> Result<Self, WebSocketError> {
        let url: reqwest::Url = url.parse()?;
        let host = url.host_str().ok_or(WebSocketError::InvalidUrl)?;
        let port = url.port().unwrap_or(80);
        
        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        
        // Generate key
        let key = base64::engine::general_purpose::STANDARD.encode(&[0u8; 16]);
        
        // Send upgrade request
        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {}\r\n\
             Sec-WebSocket-Version: 13\r\n\r\n",
            url.path(),
            host,
            key
        );
        
        stream.write_all(request.as_bytes())?;
        
        // Read response
        let mut buffer = [0; 4096];
        stream.read(&mut buffer)?;
        
        // Verify 101 Switching Protocols
        let response = String::from_utf8_lossy(&buffer);
        if !response.contains("101") {
            return Err(WebSocketError::HandshakeFailed);
        }
        
        Ok(Self {
            ws: WebSocket { stream },
        })
    }
    
    fn send(&mut self, message: &str) -> Result<(), WebSocketError> {
        self.ws.send_text(message)
    }
    
    fn receive(&mut self) -> Result<Option<String>, WebSocketError> {
        self.ws.receive_frame()
    }
}
```

## Network Protocols

### Custom Binary Protocol

```fusion
// Simple length-prefixed protocol
struct LengthPrefixedCodec;

impl LengthPrefixedCodec {
    fn encode(message: &[u8]) -> Vec<u8> {
        let length = message.len() as u32;
        let mut buffer = Vec::with_capacity(4 + message.len());
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend_from_slice(message);
        buffer
    }
    
    fn decode(buffer: &[u8]) -> Result<(&[u8], &[u8]), CodecError> {
        if buffer.len() < 4 {
            return Err(CodecError::Incomplete);
        }
        
        let length = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        
        if buffer.len() < 4 + length {
            return Err(CodecError::Incomplete);
        }
        
        let message = &buffer[4..4 + length];
        let remaining = &buffer[4 + length..];
        
        Ok((message, remaining))
    }
}
```

### Message Queue Protocol

```fusion
// Simple publish-subscribe protocol
enum MqMessage {
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Publish { topic: String, payload: Vec<u8> },
    Message { topic: String, payload: Vec<u8> },
}

impl MqMessage {
    fn encode(&self) -> Vec<u8> {
        match self {
            MqMessage::Subscribe { topic } => {
                let mut buf = vec![0x01];
                buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                buf.extend_from_slice(topic.as_bytes());
                buf
            }
            MqMessage::Unsubscribe { topic } => {
                let mut buf = vec![0x02];
                buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                buf.extend_from_slice(topic.as_bytes());
                buf
            }
            MqMessage::Publish { topic, payload } => {
                let mut buf = vec![0x03];
                buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                buf.extend_from_slice(topic.as_bytes());
                buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
                buf.extend_from_slice(payload);
                buf
            }
            MqMessage::Message { topic, payload } => {
                let mut buf = vec![0x04];
                buf.extend_from_slice(&(topic.len() as u16).to_be_bytes());
                buf.extend_from_slice(topic.as_bytes());
                buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
                buf.extend_from_slice(payload);
                buf
            }
        }
    }
    
    fn decode(buffer: &[u8]) -> Result<(&[u8], Self), CodecError> {
        if buffer.is_empty() {
            return Err(CodecError::Incomplete);
        }
        
        let (message, remaining) = match buffer[0] {
            0x01 => {
                let topic_len = u16::from_be_bytes([buffer[1], buffer[2]]) as usize;
                let topic = String::from_utf8_lossy(&buffer[3..3 + topic_len]).to_string();
                (&buffer[3 + topic_len..], MqMessage::Subscribe { topic })
            }
            0x02 => {
                let topic_len = u16::from_be_bytes([buffer[1], buffer[2]]) as usize;
                let topic = String::from_utf8_lossy(&buffer[3..3 + topic_len]).to_string();
                (&buffer[3 + topic_len..], MqMessage::Unsubscribe { topic })
            }
            0x03 => {
                let topic_len = u16::from_be_bytes([buffer[1], buffer[2]]) as usize;
                let topic = String::from_utf8_lossy(&buffer[3..3 + topic_len]).to_string();
                let payload_len = u32::from_be_bytes([
                    buffer[3 + topic_len],
                    buffer[4 + topic_len],
                    buffer[5 + topic_len],
                    buffer[6 + topic_len],
                ]) as usize;
                let payload = buffer[7 + topic_len..7 + topic_len + payload_len].to_vec();
                (&buffer[7 + topic_len + payload_len..], MqMessage::Publish { topic, payload })
            }
            _ => return Err(CodecError::InvalidMessage),
        };
        
        Ok((remaining, message))
    }
}
```

## Security

### TLS/SSL

```fusion
use std::net::TcpStream;
use std::io::{Read, Write};
use rustls::{ClientConfig, ClientConnection, StreamOwned};

fn tls_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    
    let server_name = "example.com".try_into()?;
    let mut conn = ClientConnection::new(Arc::new(config), server_name)?;
    
    let tcp_stream = TcpStream::connect("example.com:443")?;
    let mut tls_stream = StreamOwned::new(conn, tcp_stream);
    
    tls_stream.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")?;
    
    let mut response = String::new();
    tls_stream.read_to_string(&mut response)?;
    
    println!("{}", response);
    
    Ok(())
}
```

### SSH Client

```fusion
use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;

fn ssh_client() -> Result<(), Box<dyn std::error::Error>> {
    let tcp = TcpStream::connect("example.com:22")?;
    let mut session = Session::new()?;
    
    session.set_tcp_stream(tcp);
    session.handshake()?;
    
    session.userauth_password("username", "password")?;
    
    let mut channel = session.channel_session()?;
    channel.exec("ls -la")?;
    
    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    
    println!("{}", output);
    
    channel.wait_close()?;
    
    Ok(())
}
```

## Summary

Fusion's networking capabilities include:

1. **TCP/UDP Sockets**: Low-level socket programming
2. **HTTP Servers**: Both synchronous and asynchronous implementations
3. **WebSocket Applications**: Real-time bidirectional communication
4. **Custom Protocols**: Binary and text-based protocol implementations
5. **Security**: TLS/SSL and SSH support

Fusion's async/await and type system make building network applications both safe and efficient.

In the next chapter, we'll explore database integration with Fusion.