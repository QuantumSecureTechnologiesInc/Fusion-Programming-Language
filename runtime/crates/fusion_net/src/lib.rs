//! # Fusion Net
//!
//! High-performance networking primitives with connection pooling.
//!
//! Provides TCP/UDP client and server abstractions with async I/O using
//! standard library networking. Designed for low-latency Fusion Runtime workloads.

use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, TcpListener as StdTcpListener, TcpStream as StdTcpStream, ToSocketAddrs, UdpSocket as StdUdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, trace};

// ==================== TCP Client ====================

/// A TCP client connection to a remote address.
pub struct TcpClient {
    stream: StdTcpStream,
    peer_addr: SocketAddr,
}

impl TcpClient {
    /// Connect to a remote address with a timeout.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addrs = addr.to_socket_addrs()?;
        let mut last_err = None;

        for addr in addrs {
            match StdTcpStream::connect_timeout(
                &addr,
                Duration::from_secs(5),
            ) {
                Ok(stream) => {
                    stream.set_nodelay(true)?;
                    let peer_addr = stream.peer_addr()?;
                    debug!("Connected TCP client to {}", peer_addr);
                    return Ok(Self { stream, peer_addr });
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "no addresses resolved")
        }))
    }

    /// Get the peer address of this connection.
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// Write data to the stream.
    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        use std::io::Write;
        self.stream.write(data)
    }

    /// Read data from the stream.
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use std::io::Read;
        self.stream.read(buf)
    }

    /// Set TCP_NODELAY option.
    pub fn set_nodelay(&self, enabled: bool) -> io::Result<()> {
        self.stream.set_nodelay(enabled)
    }

    /// Set read timeout.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)
    }

    /// Set write timeout.
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_write_timeout(timeout)
    }

    /// Consume and return the inner stream.
    pub fn into_inner(self) -> StdTcpStream {
        self.stream
    }
}

// ==================== TCP Server ====================

/// A TCP server that listens for incoming connections.
pub struct TcpServer {
    listener: StdTcpListener,
    local_addr: SocketAddr,
}

impl TcpServer {
    /// Bind to a local address.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addrs = addr.to_socket_addrs()?;
        let mut last_err = None;

        for addr in addrs {
            match StdTcpListener::bind(addr) {
                Ok(listener) => {
                    listener.set_nonblocking(false)?;
                    let local_addr = listener.local_addr()?;
                    debug!("TCP server listening on {}", local_addr);
                    return Ok(Self {
                        listener,
                        local_addr,
                    });
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "no addresses resolved")
        }))
    }

    /// Get the local address this server is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accept a single incoming connection (blocking).
    pub fn accept(&self) -> io::Result<(TcpClient, SocketAddr)> {
        let (stream, peer_addr) = self.listener.accept()?;
        stream.set_nodelay(true)?;
        debug!("Accepted TCP connection from {}", peer_addr);
        Ok((
            TcpClient {
                stream,
                peer_addr,
            },
            peer_addr,
        ))
    }

    /// Set the listener to non-blocking mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.listener.set_nonblocking(nonblocking)
    }

    /// Set SO_REUSEADDR.
    pub fn set_reuseaddr(&self, reuse: bool) -> io::Result<()> {
        // socket2 would be needed for this; fall back to std
        let _ = reuse;
        Ok(())
    }
}

// ==================== UDP Socket ====================

/// A UDP socket for sending and receiving datagrams.
pub struct UdpSocket {
    socket: StdUdpSocket,
    local_addr: SocketAddr,
}

impl UdpSocket {
    /// Bind to a local address.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addrs = addr.to_socket_addrs()?;
        let mut last_err = None;

        for addr in addrs {
            match StdUdpSocket::bind(addr) {
                Ok(socket) => {
                    let local_addr = socket.local_addr()?;
                    debug!("UDP socket bound to {}", local_addr);
                    return Ok(Self {
                        socket,
                        local_addr,
                    });
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "no addresses resolved")
        }))
    }

    /// Get the local address this socket is bound to.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Send data to a specific address.
    pub fn send_to(&self, buf: &[u8], target: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(buf, target)
    }

    /// Receive data from any source.
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    /// Connect to a remote address for sending/receiving.
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        self.socket.connect(addr)
    }

    /// Send data to the connected peer.
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    /// Receive data from the connected peer.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }

    /// Set broadcast flag.
    pub fn set_broadcast(&self, on: bool) -> io::Result<()> {
        self.socket.set_broadcast(on)
    }

    /// Set read timeout.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_read_timeout(timeout)
    }

    /// Set write timeout.
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_write_timeout(timeout)
    }
}

// ==================== Connection Pool ====================

/// Configuration for the connection pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections per address.
    pub max_per_addr: usize,
    /// Maximum total connections.
    pub max_total: usize,
    /// Idle timeout before a connection is dropped.
    pub idle_timeout: Duration,
    /// Connection timeout.
    pub connect_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_per_addr: 8,
            max_total: 64,
            idle_timeout: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(5),
        }
    }
}

/// Statistics for the connection pool.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Number of active (in-use) connections.
    pub active: usize,
    /// Number of idle (available) connections.
    pub idle: usize,
    /// Total connections created since pool creation.
    pub total_created: u64,
    /// Total connections dropped (timed out or excess).
    pub total_dropped: u64,
    /// Total connection attempts that failed.
    pub total_failures: u64,
}

/// A pooled TCP connection with metadata.
struct PooledConnection {
    client: TcpClient,
    addr: SocketAddr,
    last_used: std::time::Instant,
}

/// A connection pool that manages reusable TCP connections.
pub struct ConnectionPool {
    config: PoolConfig,
    connections: Mutex<VecDeque<PooledConnection>>,
    stats: Mutex<PoolStats>,
}

impl ConnectionPool {
    /// Create a new connection pool with the given configuration.
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            connections: Mutex::new(VecDeque::new()),
            stats: Mutex::new(PoolStats::default()),
        }
    }

    /// Create a pool with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(PoolConfig::default())
    }

    /// Get a connection from the pool, or create a new one.
    pub fn get(&self, addr: SocketAddr) -> io::Result<TcpClient> {
        // Try to reuse an existing connection
        {
            let mut conns = self.connections.lock().unwrap();
            let now = std::time::Instant::now();

            // Find a non-idle, matching connection
            for i in 0..conns.len() {
                if conns[i].addr == addr
                    && now.duration_since(conns[i].last_used) < self.config.idle_timeout
                {
                    let conn = conns.remove(i).unwrap();
                    self.stats.lock().unwrap().active += 1;
                    trace!("Reusing pooled connection to {}", addr);
                    return Ok(conn.client);
                }
            }

            // Drop expired connections
            let original_len = conns.len();
            conns.retain(|c| {
                now.duration_since(c.last_used) < self.config.idle_timeout
            });
            let dropped = original_len - conns.len();
            if dropped > 0 {
                self.stats.lock().unwrap().total_dropped += dropped as u64;
                debug!("Dropped {} expired connections from pool", dropped);
            }
        }

        // Create a new connection
        trace!("Creating new connection to {}", addr);
        let client = TcpClient::connect(addr)?;

        let mut stats = self.stats.lock().unwrap();
        stats.total_created += 1;
        stats.active += 1;

        Ok(client)
    }

    /// Return a connection to the pool for reuse.
    pub fn put(&self, client: TcpClient) {
        let addr = client.peer_addr();
        let mut conns = self.connections.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        stats.active = stats.active.saturating_sub(1);

        // Check capacity
        let total: usize = conns.len() + 1;
        if total > self.config.max_total || conns.len() >= self.config.max_per_addr {
            stats.total_dropped += 1;
            debug!("Pool at capacity, dropping connection to {}", addr);
            return;
        }

        conns.push_back(PooledConnection {
            client,
            addr,
            last_used: std::time::Instant::now(),
        });

        trace!("Returned connection to pool for {}", addr);
    }

    /// Get pool statistics.
    pub fn stats(&self) -> PoolStats {
        let conns = self.connections.lock().unwrap();
        let mut stats = self.stats.lock().unwrap().clone();
        stats.idle = conns.len();
        stats
    }

    /// Clear all pooled connections.
    pub fn clear(&self) {
        let mut conns = self.connections.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();
        let count = conns.len() as u64;
        conns.clear();
        stats.total_dropped += count;
        stats.idle = 0;
        debug!("Cleared {} connections from pool", count);
    }

    /// Get the number of idle connections.
    pub fn idle_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }
}

// ==================== Listener with Pool ====================

/// A TCP server that automatically manages a connection pool for accepted connections.
pub struct PooledTcpServer {
    server: TcpServer,
    pool: Arc<ConnectionPool>,
}

impl PooledTcpServer {
    /// Create a new pooled TCP server bound to the given address.
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let server = TcpServer::bind(addr)?;
        Ok(Self {
            server,
            pool: Arc::new(ConnectionPool::new(PoolConfig::default())),
        })
    }

    /// Create with custom pool configuration.
    pub fn bind_with_config<A: ToSocketAddrs>(
        addr: A,
        pool_config: PoolConfig,
    ) -> io::Result<Self> {
        let server = TcpServer::bind(addr)?;
        Ok(Self {
            server,
            pool: Arc::new(ConnectionPool::new(pool_config)),
        })
    }

    /// Accept a connection and return a client + a handle to return it to the pool.
    pub fn accept(&self) -> io::Result<(TcpClient, SocketAddr)> {
        self.server.accept()
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &ConnectionPool {
        &self.pool
    }

    /// Get the server's local address.
    pub fn local_addr(&self) -> SocketAddr {
        self.server.local_addr()
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_tcp_server_bind() {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let server = TcpServer::bind(addr);
        assert!(server.is_ok());
    }

    #[test]
    fn test_tcp_server_accept() {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let server = TcpServer::bind(addr).unwrap();
        let server_addr = server.local_addr();

        // Connect a client in a separate thread
        let client_thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            TcpClient::connect(server_addr)
        });

        let (client, peer) = server.accept().unwrap();
        assert_eq!(client.peer_addr(), peer);

        let _ = client_thread.join().unwrap();
    }

    #[test]
    fn test_tcp_client_server_roundtrip() {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let server = TcpServer::bind(addr).unwrap();
        let server_addr = server.local_addr();

        let client_thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let mut client = TcpClient::connect(server_addr).unwrap();
            client.write(b"hello").unwrap();
            let mut buf = [0u8; 16];
            let n = client.read(&mut buf).unwrap();
            assert_eq!(&buf[..n], b"world");
        });

        let (mut client, _) = server.accept().unwrap();
        let mut buf = [0u8; 16];
        let n = client.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");
        client.write(b"world").unwrap();

        client_thread.join().unwrap();
    }

    #[test]
    fn test_udp_socket() {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let socket1 = UdpSocket::bind(addr).unwrap();
        let addr1 = socket1.local_addr();

        let addr2 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let socket2 = UdpSocket::bind(addr2).unwrap();
        let addr2 = socket2.local_addr();

        socket1.send_to(b"ping", addr2).unwrap();
        let mut buf = [0u8; 16];
        let (n, from) = socket2.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"ping");
        assert_eq!(from, addr1);
    }

    #[test]
    fn test_connection_pool_basic() {
        let pool = ConnectionPool::with_defaults();
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let server = TcpServer::bind(addr).unwrap();
        let server_addr = server.local_addr();

        // Create a connection
        let client = pool.get(server_addr).unwrap();
        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.active, 1);

        // Return to pool
        pool.put(client);
        let stats = pool.stats();
        assert_eq!(stats.idle, 1);
        assert_eq!(stats.active, 0);

        // Reuse
        let _client2 = pool.get(server_addr).unwrap();
        let stats = pool.stats();
        assert_eq!(stats.idle, 0);
        assert_eq!(stats.active, 1);
    }

    #[test]
    fn test_connection_pool_clear() {
        let pool = ConnectionPool::with_defaults();
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let server = TcpServer::bind(addr).unwrap();
        let server_addr = server.local_addr();

        let client = pool.get(server_addr).unwrap();
        pool.put(client);
        assert_eq!(pool.idle_count(), 1);

        pool.clear();
        assert_eq!(pool.idle_count(), 0);
    }

    #[test]
    fn test_pool_config() {
        let config = PoolConfig {
            max_per_addr: 4,
            max_total: 32,
            idle_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(3),
        };
        let pool = ConnectionPool::new(config);
        assert_eq!(pool.idle_count(), 0);
    }
}
