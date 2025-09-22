//! # TCP Server Implementation
//!
//! This module provides the TCP server that handles client connections and processes
//! commands. It implements a simple request-response protocol over TCP sockets.
//!
//! ## Architecture
//!
//! The server uses an asynchronous, multi-connection design:
//! - Main server loop accepts incoming connections
//! - Each connection spawns a separate async task
//! - Commands are parsed and executed against the shared storage
//! - Responses are sent back to the client
//!
//! ## Protocol
//!
//! The server implements a Redis-like text protocol:
//! - Basic Commands: `GET key`, `SET key value`, `DELETE key`
//! - Numeric Operations: `INC key [amount]`, `DEC key [amount]`
//! - String Operations: `APPEND key value`, `PREPEND key value`
//! - Bulk Operations: `MGET key1 key2 ...`, `MSET key1 value1 key2 value2 ...`, `TRUNCATE`
//! - Responses: `VALUE data`, `VALUES count\r\nkey1 value1\r\nkey2 value2...`, `OK`, `NOT_FOUND`, `ERROR message`
//! - All messages are terminated with `\r\n`
//!
//! ## Concurrency
//!
//! The storage engine is wrapped in `Arc<Mutex<>>` to allow safe concurrent access
//! from multiple client connections. Each connection gets its own task but shares
//! the same underlying storage.
use hex; 
use crate::sync::SyncManager;
use crate::protocol::SyncOptions;     // the options parsed by SYNC (full/verify)
use crate::store::KVEngineStoreTrait;
use anyhow::Result;
use log::{error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::collections::HashMap; 
use crate::config::Config;
use crate::protocol::{Command, Protocol};
use crate::replication::Replicator;

/// Server statistics for monitoring and diagnostics.
///
/// This struct tracks various metrics about server operations, including
/// connection counts, operation counts, and memory usage estimates.
#[derive(Debug)]
pub struct ServerStats {
    /// Total number of connections since server start
    pub total_connections: AtomicU64,
    
    /// Current number of active connections
    pub active_connections: AtomicU64,
    
    /// Total number of commands processed
    pub total_commands: AtomicU64,
    
    /// Number of GET commands processed
    pub get_commands: AtomicU64,

    /// Number of SCAN commands processed
    pub scan_commands: AtomicU64,

    /// Number of PING commands processed
    pub ping_commands: AtomicU64,

    /// Number of ECHO commands processed
    pub echo_commands: AtomicU64,

    /// Number of DB size commands processed
    pub dbsize_commands: AtomicU64,

    /// Number of EXISTS commands processed
    pub exists_commands: AtomicU64,

    /// Number of FLUSHDB commands processed
    pub flushdb_commands: AtomicU64,

    /// Number of MEMORY commands processed
    pub memory_commands: AtomicU64,

    /// Number of CLIENT commands processed
    pub clientlist_commands: AtomicU64,

    /// Number of SET commands processed
    pub set_commands: AtomicU64,
    
    /// Number of DELETE commands processed
    pub delete_commands: AtomicU64,
    
    /// Number of numeric operations (INC/DEC) processed
    pub numeric_commands: AtomicU64,
    
    /// Number of string operations (APPEND/PREPEND) processed
    pub string_commands: AtomicU64,
    
    /// Number of bulk operations (MGET/MSET/TRUNCATE) processed
    pub bulk_commands: AtomicU64,
    
    /// Number of statistical commands (STATS/INFO/PING) processed
    pub stat_commands: AtomicU64,
    
    /// Number of server management commands (VERSION/FLUSH/SHUTDOWN) processed
    pub management_commands: AtomicU64,
    
    /// Server start time
    pub start_time: Instant,

    /// Number of SYNC commands processed
    pub sync_commands: AtomicU64,

    /// number of hash commands processed
    pub hash_commands: AtomicU64,
}

struct ClientMeta {
    id: u64,
    addr: SocketAddr,
    connected_unix: u64,                    // thời điểm connect (epoch seconds)
    last_cmd_unix: std::sync::atomic::AtomicU64, // lần cuối gửi lệnh (epoch seconds)
}
type ClientTable = Arc<tokio::sync::Mutex<HashMap<u64, Arc<ClientMeta>>>>;

impl Clone for ServerStats {
    fn clone(&self) -> Self {
        Self {
            total_connections: AtomicU64::new(self.total_connections.load(Ordering::Relaxed)),
            active_connections: AtomicU64::new(self.active_connections.load(Ordering::Relaxed)),
            total_commands: AtomicU64::new(self.total_commands.load(Ordering::Relaxed)),
            get_commands: AtomicU64::new(self.get_commands.load(Ordering::Relaxed)),
            hash_commands: AtomicU64::new(self.hash_commands.load(Ordering::Relaxed)),
            scan_commands: AtomicU64::new(self.scan_commands.load(Ordering::Relaxed)),
            ping_commands: AtomicU64::new(self.ping_commands.load(Ordering::Relaxed)),
            echo_commands: AtomicU64::new(self.echo_commands.load(Ordering::Relaxed)),
            exists_commands: AtomicU64::new(self.exists_commands.load(Ordering::Relaxed)),
            flushdb_commands: AtomicU64::new(self.flushdb_commands.load(Ordering::Relaxed)),
            memory_commands: AtomicU64::new(self.memory_commands.load(Ordering::Relaxed)),
            clientlist_commands: AtomicU64::new(self.clientlist_commands.load(Ordering::Relaxed)),
            dbsize_commands: AtomicU64::new(self.dbsize_commands.load(Ordering::Relaxed)),
            set_commands: AtomicU64::new(self.set_commands.load(Ordering::Relaxed)),
            delete_commands: AtomicU64::new(self.delete_commands.load(Ordering::Relaxed)),
            numeric_commands: AtomicU64::new(self.numeric_commands.load(Ordering::Relaxed)),
            string_commands: AtomicU64::new(self.string_commands.load(Ordering::Relaxed)),
            bulk_commands: AtomicU64::new(self.bulk_commands.load(Ordering::Relaxed)),
            stat_commands: AtomicU64::new(self.stat_commands.load(Ordering::Relaxed)),
            sync_commands: AtomicU64::new(self.sync_commands.load(Ordering::Relaxed)),
            management_commands: AtomicU64::new(self.management_commands.load(Ordering::Relaxed)),
            start_time: self.start_time,
        }
    }
}

impl Default for ServerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerStats {
    /// Create a new ServerStats instance with all counters initialized to zero
    /// and start_time set to the current time.
    pub fn new() -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            total_commands: AtomicU64::new(0),
            get_commands: AtomicU64::new(0),
            scan_commands: AtomicU64::new(0),
            hash_commands: AtomicU64::new(0),
            echo_commands: AtomicU64::new(0),
            ping_commands: AtomicU64::new(0),
            exists_commands: AtomicU64::new(0),
            flushdb_commands: AtomicU64::new(0),
            clientlist_commands: AtomicU64::new(0),
            dbsize_commands: AtomicU64::new(0),
            memory_commands: AtomicU64::new(0),
            set_commands: AtomicU64::new(0),
            delete_commands: AtomicU64::new(0),
            numeric_commands: AtomicU64::new(0),
            string_commands: AtomicU64::new(0),
            bulk_commands: AtomicU64::new(0),
            stat_commands: AtomicU64::new(0),
            management_commands: AtomicU64::new(0),
            start_time: Instant::now(),
            sync_commands: AtomicU64::new(0),
        }
    }
    
    /// Get the server uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// Format uptime as a human-readable string (days:hours:minutes:seconds)
    pub fn uptime_human(&self) -> String {
        let seconds = self.uptime_seconds();
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    }
    
    /// Increment the counter for a specific command type
    pub fn increment_command_counter(&self, command: &Command) {
        self.total_commands.fetch_add(1, Ordering::Relaxed);
        
        match command {
            Command::Get { .. } => {
                self.get_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Scan { .. } => {
                self.scan_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Ping { .. } => {
                self.ping_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Echo { .. } => {
                self.echo_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Dbsize => {
                self.dbsize_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Exists { .. } => {
                self.exists_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Set { .. } => {
                self.set_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Delete { .. } => {
                self.delete_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Increment { .. } | Command::Decrement { .. } => {
                self.numeric_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Append { .. } | Command::Prepend { .. } => {
                self.string_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::MultiGet { .. } | Command::MultiSet { .. } | Command::Truncate => {
                self.bulk_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Stats | Command::Info => {
                self.stat_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Version | Command::Flushdb | Command::Shutdown => {
                self.management_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Memory => {
                self.memory_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Clientlist => {
                self.management_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Sync {..} => {
                self.sync_commands.fetch_add(1, Ordering::Relaxed);
            }
            Command::Hash {..} => {
                self.hash_commands.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    /// Format all statistics as a multi-line string for the STATS command
    pub fn format_stats(&self) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("uptime_seconds:{}\r\n", self.uptime_seconds()));
        result.push_str(&format!("uptime:{}\r\n", self.uptime_human()));
        result.push_str(&format!("total_connections:{}\r\n", self.total_connections.load(Ordering::Relaxed)));
        result.push_str(&format!("active_connections:{}\r\n", self.active_connections.load(Ordering::Relaxed)));
        result.push_str(&format!("total_commands:{}\r\n", self.total_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("get_commands:{}\r\n", self.get_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("scan_commands:{}\r\n", self.scan_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("ping_commands:{}\r\n", self.ping_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("echo_commands:{}\r\n", self.echo_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("flushdb_commands:{}\r\n", self.flushdb_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("memory_commands:{}\r\n", self.memory_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("clientlist_commands:{}\r\n", self.clientlist_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("exists_commands:{}\r\n", self.exists_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("dbsize_commands:{}\r\n", self.dbsize_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("set_commands:{}\r\n", self.set_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("delete_commands:{}\r\n", self.delete_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("numeric_commands:{}\r\n", self.numeric_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("string_commands:{}\r\n", self.string_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("bulk_commands:{}\r\n", self.bulk_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("stat_commands:{}\r\n", self.stat_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("sync_commands:{}\r\n", self.sync_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("hash_commands:{}\r\n", self.hash_commands.load(Ordering::Relaxed)));
        result.push_str(&format!("management_commands:{}\r\n", self.management_commands.load(Ordering::Relaxed)));
        
        // Add memory usage estimate (this is a very rough estimate)
        let estimated_memory_kb = std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        
        result.push_str(&format!("used_memory_kb:{}\r\n", estimated_memory_kb));
        
        result
    }
}

/// TCP server for handling client connections.
///
/// The server binds to a specified address and port, then accepts incoming
/// connections and processes commands asynchronously.
pub struct Server {
    /// Server configuration including bind address and port
    config: Config,

    /// The storage engine that will be shared across all client connections
    store: Box<dyn KVEngineStoreTrait + Send + Sync>,
    
    /// Server statistics for monitoring and diagnostics
    stats: ServerStats,
}

impl Server {
    /// Create a new server instance.
    ///
    /// # Arguments
    /// * `config` - Server configuration (address, port, etc.)
    /// * `store` - Storage engine instance to use for all operations
    ///
    /// # Returns
    /// * `Server` - New server instance ready to run
    pub fn new(config: Config, store: Box<dyn KVEngineStoreTrait + Send + Sync>) -> Self {
        Self { 
            config, 
            store,
            stats: ServerStats::new(),
        }
    }

    /// Start the server and begin accepting connections.
    ///
    /// This method runs indefinitely, accepting new connections and spawning
    /// tasks to handle them. Each connection gets its own async task but all
    /// share the same storage engine.
    ///
    /// # Returns
    /// * `Result<()>` - Never returns normally, only on bind errors
    ///
    /// # Errors
    /// Returns an error if:
    /// - Unable to bind to the specified address/port
    /// - Network-level errors occur
    ///
    /// # Example
    /// ```rust
    /// let config = Config::default();
    /// let store = Box::new(RwLockEngine::new("./data")?);
    /// let server = Server::new(config, store);
    /// server.run().await?; // Runs forever
    /// ```
    pub async fn run(self) -> Result<()> {
        let clients: ClientTable = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let client_id_gen = Arc::new(AtomicU64::new(0));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);

        // Wrap the storage in `Arc<Mutex<>>` for safe concurrent access
        let store = Arc::new(Mutex::new(self.store));
        
        let sync_manager = Arc::new(tokio::sync::Mutex::new(
            SyncManager::new_with_shared_store(&self.config, Arc::clone(&store))
        ));

        // Share server statistics across all connections
        let stats = Arc::new(self.stats.clone());

        // Initialize replication if enabled
        let replicator_opt: Option<Replicator> = if self.config.replication.enabled {
            let r = Replicator::new(&self.config).await?;
            // Start background apply loop
            r.start_replication_handler(Arc::clone(&store)).await;
            Some(r)
        } else { None };

        // TODO: Add graceful shutdown handling
        // TODO: Add connection limits and rate limiting

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("Accepted connection from {}", addr);
                    
                    // Clone the Arc for this connection
                    let store_clone = Arc::clone(&store);
                    let stats_clone = Arc::clone(&stats);
                    
                    // Update connection statistics
                    stats_clone.total_connections.fetch_add(1, Ordering::Relaxed);
                    stats_clone.active_connections.fetch_add(1, Ordering::Relaxed);

                    // Spawn a new task for each client connection
                    let repl_clone = replicator_opt.clone();
                    let sync_manager_clone = Arc::clone(&sync_manager);
                    let id = client_id_gen.fetch_add(1, Ordering::Relaxed) + 1;
                    let now_unix = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs();
                    let meta = Arc::new(ClientMeta {
                        id,
                        addr,
                        connected_unix: now_unix,
                        last_cmd_unix: AtomicU64::new(now_unix),
                    });
                    {
                        let mut tbl = clients.lock().await;
                        tbl.insert(id, Arc::clone(&meta));
                    }
                    let clients_clone = Arc::clone(&clients);
                    let meta_clone = Arc::clone(&meta);
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(socket, addr, store_clone, stats_clone.clone(), repl_clone, meta_clone, clients_clone.clone(), sync_manager_clone).await {
                            error!("Error handling connection from {}: {}", addr, e);
                        }
                        
                        // Decrement active connections when the connection ends
                        stats_clone.active_connections.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection.
    ///
    /// This method processes commands from a client connection until the client
    /// disconnects or an error occurs. Each command is parsed, executed against
    /// the storage, and a response is sent back.
    /// 
    /// # Arguments
    /// * `socket` - The TCP stream for this client connection
    /// * `addr` - Client's address (for logging)
    /// * `store` - Shared reference to the storage engine
    /// * `stats` - Shared reference to server statistics
    /// 
    /// # Returns
    /// * `Result<()>` - Success when client disconnects normally, error on failures
    /// 
    /// # Protocol Handling
    /// - Reads commands from the socket in 1KB chunks
    /// - Parses commands using the Protocol parser
    /// - Executes commands against the storage engine
    /// - Sends appropriate responses back to the client
    /// 
    /// # Error Handling
    /// - Invalid commands result in ERROR responses
    /// - Network errors terminate the connection
    /// - Storage errors are converted to ERROR responses
    async fn handle_connection(
        socket: TcpStream,
        addr: SocketAddr,
        store: Arc<Mutex<Box<dyn KVEngineStoreTrait + Send + Sync>>>,
        stats: Arc<ServerStats>,
        replicator: Option<Replicator>,
        client_meta: Arc<ClientMeta>,
        clients: ClientTable,
        sync_manager: Arc<tokio::sync::Mutex<SyncManager>>,
    ) -> Result<()> {
        let (read_half, mut write_half) = socket.into_split();
        let mut reader = BufReader::new(read_half);
        let protocol = Protocol::new();

        // Local helper describing what to publish after the storage write.
        enum Publish {
            Set(String, String),
            Delete(String),
            Incr(String, i64),
            Decr(String, i64),
            Append(String, String),
            Prepend(String, String),
        }

        loop {
            // Read a complete line from the client (terminated by \n)
            // Defensive upper bound to prevent OOM attacks
            let mut request_line = String::new();
            match reader.read_line(&mut request_line).await {
                Ok(0) => {
                    // Client closed the connection
                    info!("Client {} disconnected", addr);
                    break;
                }
                Ok(bytes_read) => {
                    // Check for line length abuse (1MB limit)
                    if bytes_read > 1024 * 1024 {
                        let error_msg = "ERROR line too long\r\n";
                        let _ = write_half.write_all(error_msg.as_bytes()).await;
                        error!("Dropping connection {}: line too long ({} bytes)", addr, bytes_read);
                        break;
                    }
                    // Successfully read a line
                }
                Err(e) => {
                    error!("Error reading from client {}: {}", addr, e);
                    break;
                }
            };

            match protocol.parse(&request_line) {
                Ok(command) => {
                    let now_unix = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs();
                    client_meta.last_cmd_unix.store(now_unix, Ordering::Relaxed);
                    // Update command statistics
                    stats.increment_command_counter(&command);
                    
                    // Process the command. We avoid holding the store lock across awaits
                    // by computing an optional publish action and performing it afterward.
                    let mut publishes: Vec<Publish> = Vec::new();
                    let response = match command.clone() {
                        Command::Get { key } => {
                            let store = store.lock().await;
                            match store.get(&key) {
                                Some(value) => format!("VALUE {}\r\n", value),
                                None => "NOT_FOUND\r\n".to_string(),
                            }
                        }
                        Command::Ping { message } => {
                            let store = store.lock().await;
                            let pong_response = store.ping(&message);
                            format!("{}\r\n", pong_response)
                        }
                        Command::Echo { message } => {
                            let store = store.lock().await;
                            let echo_response = store.echo(&message);
                            format!("{}\r\n", echo_response)
                        }
                        Command::Dbsize => {
                            let store = store.lock().await;
                            let size = store.dbsize();
                            format!("DBSIZE {}\r\n", size)
                        }
                        Command::Exists { keys } => {
                            let store = store.lock().await;
                            let mut count = 0;
                            for key in keys {
                                if store.exists(&key) {
                                    count += 1;
                                }
                            }
                            format!("EXISTS {}\r\n", count)
                        }
                        Command::Scan { prefix } => {
                            let store = store.lock().await;
                            let results = store.scan(&prefix);
                            let mut response = format!("KEYS {}\r\n", results.len());
                            for k in results {
                                response.push_str(&format!("{}\r\n", k));
                            }
                            response
                        }
                        Command::Set { key, value } => {
                            let store = store.lock().await;
                            match store.set(key.clone(), value.clone()) {
                                Ok(_) => {
                                    publishes.push(Publish::Set(key.clone(), value.clone()));
                                    "OK\r\n".to_string()
                                }
                                Err(e) => format!("ERROR {}\r\n", e),
                            }
                        }
                        Command::Delete { key } => {
                            let deleted = {
                                let store = store.lock().await;
                                store.delete(&key)
                            };
                            if deleted {
                                publishes.push(Publish::Delete(key.clone()));
                                "DELETED\r\n".to_string()
                            } else {
                                "NOT_FOUND\r\n".to_string()
                            }
                        }
                        Command::Memory => {
                            let store = store.lock().await;
                            let usage = store.memory_usage();
                            format!("MEMORY {}\r\n", usage)
                        }
                        Command::Clientlist => {

                            let snapshot: Vec<Arc<ClientMeta>> = {
                                let tbl = clients.lock().await;
                                tbl.values().cloned().collect()
                            };
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or(Duration::from_secs(0))
                                .as_secs();

                            let mut out = String::new();
                            out.push_str("CLIENT LIST\r\n");
                            for c in snapshot {
                                let age = now.saturating_sub(c.connected_unix);
                                let idle = now.saturating_sub(c.last_cmd_unix.load(Ordering::Relaxed));
                                out.push_str(&format!(
                                    "id={} addr={} age={} idle={}\r\n",
                                    c.id, c.addr, age, idle
                                ));
                            }
                            out.push_str("END\r\n");
                            out
                        }
                        Command::Sync { host, port, options: _ } => {
                            let mut mgr = sync_manager.lock().await;
                            match mgr.sync_once(&host, port).await {
                                Ok(_)  => "OK\r\n".to_string(),
                                Err(e) => format!("ERROR {}\r\n", e),
                            }
                        }
                        Command::Hash { pattern } => {
                            // 1) Collect keys (all or prefix)
                            let (keys, pat_string) = {
                                let store = store.lock().await;
                                // convention: empty prefix returns ALL keys (you already added this for SCAN)
                                let ks = match &pattern {
                                    None => store.scan(""),
                                    Some(p) if p == "*" => store.scan(""),           // treat '*' as "all"
                                    Some(p)            => store.scan(p),             // simple prefix
                                };
                                (ks, pattern.unwrap_or_default())
                            };

                            // 2) Build a Merkle tree over selected keys
                            let mut tree = crate::store::merkle::MerkleTree::new();
                            {
                                let store = store.lock().await;
                                for k in keys {
                                    if let Some(v) = store.get(&k) {
                                        tree.insert(&k, &v); // your Merkle uses deterministic key ordering internally
                                    }
                                }
                            }

                            // 3) Compute root → hex (define empty = 64 zeros for determinism)
                            let hex_root = match tree.get_root_hash() {
                                Some(h) => hex::encode(h),
                                None    => "0".repeat(64), // SHA-256 length; empty set sentinel
                            };

                            // 4) Format response
                            let out = if pat_string.is_empty() {
                                format!("HASH {}\r\n", hex_root)
                            } else {
                                format!("HASH {} {}\r\n", pat_string, hex_root)
                            };

                            out
                        }

                        Command::Increment { key, amount } => {
                            // Check if the key already exists
                            let exists = { let store = store.lock().await; store.get(&key).is_some() };
                            
                            // If the key doesn't exist, create it with value 1 or the specified amount
                            if !exists {
                                let value = amount.unwrap_or(1).to_string();
                                {
                                    let store = store.lock().await;
                                    match store.set(key.clone(), value.clone()) {
                                        Ok(_) => {
                                            let nv = value.parse().unwrap_or(1);
                                            publishes.push(Publish::Incr(key.clone(), nv));
                                            format!("VALUE {}\r\n", value)
                                        }
                                        Err(e) => format!("ERROR {}\r\n", e),
                                    }
                                }
                            } else {
                                // Otherwise, increment the existing value
                                let res = { let store = store.lock().await; store.increment(&key, amount) };
                                match res {
                                    Ok(new_value) => { publishes.push(Publish::Incr(key.clone(), new_value)); format!("VALUE {}\r\n", new_value) },
                                    Err(e) => format!("ERROR {}\r\n", e),
                                }
                            }
                        }
                        Command::Decrement { key, amount } => {
                            // Check if the key already exists
                            let exists = { let store = store.lock().await; store.get(&key).is_some() };
                            
                            // If the key doesn't exist, create it with value -1 or the negative of the specified amount
                            if !exists {
                                let value = (-(amount.unwrap_or(1))).to_string();
                                {
                                    let store = store.lock().await;
                                    match store.set(key.clone(), value.clone()) {
                                        Ok(_) => { let v: i64 = value.parse().unwrap_or(-1); publishes.push(Publish::Decr(key.clone(), v)); format!("VALUE {}\r\n", value) },
                                        Err(e) => format!("ERROR {}\r\n", e),
                                    }
                                }
                            } else {
                                // Otherwise, decrement the existing value
                                let res = { let store = store.lock().await; store.decrement(&key, amount) };
                                match res {
                                    Ok(new_value) => { publishes.push(Publish::Decr(key.clone(), new_value)); format!("VALUE {}\r\n", new_value) },
                                    Err(e) => format!("ERROR {}\r\n", e),
                                }
                            }
                        }
                        Command::Append { key, value } => {
                            // Handle empty values for APPEND
                            if value.is_empty() {
                                let store = store.lock().await;
                                match store.get(&key) {
                                    Some(current_value) => format!("VALUE {}\r\n", current_value),
                                    None => "ERROR Key not found\r\n".to_string(),
                                }
                            } else {
                                // Try to get the key first
                                let current_value = { let store = store.lock().await; store.get(&key) };
                                
                                // If the key doesn't exist, create it with the value
                                if current_value.is_none() {
                                    let res = { let store = store.lock().await; store.set(key.clone(), value.clone()) };
                                    match res {
                                        Ok(_) => { publishes.push(Publish::Append(key.clone(), value.clone())); format!("VALUE {}\r\n", value) },
                                        Err(e) => format!("ERROR {}\r\n", e),
                                    }
                                } else {
                                    // Otherwise, append to the existing value
                                    let res = { let store = store.lock().await; store.append(&key, &value) };
                                    match res {
                                        Ok(new_value) => { publishes.push(Publish::Append(key.clone(), new_value.clone())); format!("VALUE {}\r\n", new_value) },
                                        Err(e) => format!("ERROR {}\r\n", e),
                                    }
                                }
                            }
                        }
                        Command::Prepend { key, value } => {
                            // Handle empty values for PREPEND
                            if value.is_empty() {
                                let store = store.lock().await;
                                match store.get(&key) {
                                    Some(current_value) => format!("VALUE {}\r\n", current_value),
                                    None => "ERROR Key not found\r\n".to_string(),
                                }
                            } else {
                                // Try to get the key first
                                let current_value = { let store = store.lock().await; store.get(&key) };
                                
                                // If the key doesn't exist, create it with the value
                                if current_value.is_none() {
                                    let res = { let store = store.lock().await; store.set(key.clone(), value.clone()) };
                                    match res {
                                        Ok(_) => { publishes.push(Publish::Prepend(key.clone(), value.clone())); format!("VALUE {}\r\n", value) },
                                        Err(e) => format!("ERROR {}\r\n", e),
                                    }
                                } else {
                                    // Otherwise, prepend to the existing value
                                    let res = { let store = store.lock().await; store.prepend(&key, &value) };
                                    match res {
                                        Ok(new_value) => { publishes.push(Publish::Prepend(key.clone(), new_value.clone())); format!("VALUE {}\r\n", new_value) },
                                        Err(e) => format!("ERROR {}\r\n", e),
                                    }
                                }
                            }
                        }
                        Command::MultiGet { keys } => {
                            let store = store.lock().await;
                            let mut response = String::new();
                            let mut found_count = 0;
                            
                            for key in keys {
                                match store.get(&key) {
                                    Some(value) => {
                                        response.push_str(&format!("{} {}\r\n", key, value));
                                        found_count += 1;
                                    }
                                    None => {
                                        response.push_str(&format!("{} NOT_FOUND\r\n", key));
                                    }
                                }
                            }
                            
                            if found_count > 0 {
                                format!("VALUES {}\r\n{}", found_count, response)
                            } else {
                                "NOT_FOUND\r\n".to_string()
                            }
                        }
                        Command::MultiSet { pairs } => {
                            let mut result = "OK\r\n".to_string();
                            for (key, value) in pairs {
                                let res = { let store = store.lock().await; store.set(key.clone(), value.clone()) };
                                if let Err(e) = res {
                                    result = format!("ERROR {}\r\n", e);
                                    break;
                                }
                                publishes.push(Publish::Set(key.clone(), value.clone()));
                            }
                            result
                        }
                        Command::Truncate => {
                            let res = { let store = store.lock().await; store.truncate() };
                            match res {
                                Ok(_) => "OK\r\n".to_string(),
                                Err(e) => format!("ERROR {}\r\n", e),
                            }
                        }
                        Command::Stats => {
                            format!("STATS\r\n{}", stats.format_stats())
                        }
                        Command::Info => {
                            let mut info = String::new();
                            
                            // Server version from Cargo.toml
                            info.push_str(&format!("version:{}\r\n", env!("CARGO_PKG_VERSION")));
                            
                            // Server uptime
                            info.push_str(&format!("uptime_seconds:{}\r\n", stats.uptime_seconds()));
                            info.push_str(&format!("uptime:{}\r\n", stats.uptime_human()));
                            
                            // Current time
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or(Duration::from_secs(0))
                                .as_secs();
                            info.push_str(&format!("server_time_unix:{}\r\n", now));
                            
                            // Key count
                            let key_count = { let store = store.lock().await; store.count_keys().unwrap_or(0) };
                            info.push_str(&format!("db_keys:{}\r\n", key_count));
                            
                            format!("INFO\r\n{}", info)
                        }
                        Command::Version => {
                            // Return the server version from Cargo.toml
                            format!("VERSION {}\r\n", env!("CARGO_PKG_VERSION"))
                        }
                        Command::Flushdb => {
                            // Force sync to disk if the storage engine supports it
                            let res = { let store = store.lock().await; store.truncate() };
                            match res {
                                Ok(_) => "OK\r\n".to_string(),
                                Err(e) => format!("ERROR {}\r\n", e),
                            }
                        }
                        Command::Shutdown => {
                            // Send OK response before shutting down
                            let response = "OK\r\n".to_string();
                            if let Err(e) = write_half.write_all(response.as_bytes()).await {
                                error!("Error writing to client {}: {}", addr, e);
                            }
                            
                            // Log shutdown request
                            info!("Shutdown requested by client {}", addr);
                            
                            // Exit the process gracefully
                            // Note: In a production system, we would want to do a more graceful
                            // shutdown, such as closing all connections, flushing data to disk, etc.
                            std::process::exit(0);
                        }
                    };
                    // Perform publishes after the store operations (lock released)
                    if let Some(r) = &replicator {
                        for p in publishes {
                            match p {
                                Publish::Set(k, v) => { let _ = r.publish_set(&k, &v).await; }
                                Publish::Delete(k) => { let _ = r.publish_delete(&k).await; }
                                Publish::Incr(k, nv) => { let _ = r.publish_incr(&k, nv).await; }
                                Publish::Decr(k, nv) => { let _ = r.publish_decr(&k, nv).await; }
                                Publish::Append(k, nv) => { let _ = r.publish_append(&k, &nv).await; }
                                Publish::Prepend(k, nv) => { let _ = r.publish_prepend(&k, &nv).await; }
                            }
                        }
                    }
                    
                    // Send response back to client
                    if let Err(e) = write_half.write_all(response.as_bytes()).await {
                        error!("Error writing to client {}: {}", addr, e);
                        break;
                    }
                }
                Err(e) => {
                    // Send error response for invalid commands
                    let error_msg = format!("ERROR {}\r\n", e);
                    if let Err(e) = write_half.write_all(error_msg.as_bytes()).await {
                        error!("Error writing to client {}: {}", addr, e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
