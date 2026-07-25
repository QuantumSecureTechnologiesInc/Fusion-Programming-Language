//! Networking IO for the Fusion Standard Library.
use crate::types::*;
use std::io::{Read, Write};
use std::net::TcpStream as StdTcpStream;

/// Represents a TCP Socket connection.
pub struct TcpStream {
    pub fd: i32,
    pub remote_addr: FString,
    inner: StdTcpStream,
}

/// Result of a connection attempt.
pub enum ConnectionResult {
    Success(TcpStream),
    Refused,
    Timeout,
}

/// Connects to a remote address on a specified port.
pub fn connect(address: &str, port: u16) -> ConnectionResult {
    let addr = format!("{}:{}", address, port);
    match StdTcpStream::connect(&addr) {
        Ok(stream) => {
            let remote = stream
                .peer_addr()
                .map(|a| a.to_string())
                .unwrap_or_else(|_| address.to_string());
            ConnectionResult::Success(TcpStream {
                fd: 0,
                remote_addr: remote,
                inner: stream,
            })
        }
        Err(e) => {
            let kind = e.kind();
            if kind == std::io::ErrorKind::ConnectionRefused
                || kind == std::io::ErrorKind::ConnectionReset
            {
                ConnectionResult::Refused
            } else if kind == std::io::ErrorKind::TimedOut
                || kind == std::io::ErrorKind::WouldBlock
            {
                ConnectionResult::Timeout
            } else {
                ConnectionResult::Refused
            }
        }
    }
}

/// Sends raw data over the stream.
pub fn send(stream: &mut TcpStream, data: &[u8]) -> Result<FSize, FString> {
    match stream.inner.write_all(data) {
        Ok(()) => Ok(data.len() as FSize),
        Err(e) => Err(e.to_string()),
    }
}

/// Receives data from the stream.
pub fn receive(stream: &mut TcpStream, buffer_size: FSize) -> Result<FVec<u8>, FString> {
    let mut buf = vec![0u8; buffer_size];
    match stream.inner.read(&mut buf) {
        Ok(n) => {
            buf.truncate(n);
            Ok(buf)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Closes the connection.
pub fn close(stream: TcpStream) -> Result<(), FString> {
    drop(stream);
    Ok(())
}

/// Represents a TCP Listener.
pub struct TcpListener {
    inner: std::net::TcpListener,
}

/// Binds and listens on the specified address and port.
pub fn listen(address: &str, port: u16) -> Result<TcpListener, FString> {
    let addr = format!("{}:{}", address, port);
    match std::net::TcpListener::bind(&addr) {
        Ok(listener) => Ok(TcpListener { inner: listener }),
        Err(e) => Err(e.to_string()),
    }
}

/// Accepts an incoming connection.
pub fn accept(listener: &TcpListener) -> Result<TcpStream, FString> {
    match listener.inner.accept() {
        Ok((stream, _addr)) => {
            let remote = stream
                .peer_addr()
                .map(|a| a.to_string())
                .unwrap_or_default();
            Ok(TcpStream {
                fd: 0,
                remote_addr: remote,
                inner: stream,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}
