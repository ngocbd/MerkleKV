# MerkleKV 🚀

A high-performance, distributed key-value store with self-healing replication, built in Rust.

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/AI-Decenter/MerkleKV/integration-tests.yml)
[![Crates.io](https://img.shields.io/crates/v/merkledb.svg?style=for-the-badge)](https://crates.io/crates/merkledb)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

MerkleKV is an eventually-consistent, distributed key-value database designed for speed, reliability, and operational simplicity. It uses an MQTT broker for rapid, real-time update propagation and a sophisticated **Merkle tree** anti-entropy mechanism to efficiently detect and repair data inconsistencies in the background.

## 📋 Table of Contents

- [✨ Key Features](#-key-features)
- [🏗️ Architecture](#️-architecture)
  - [System Architecture Overview](#system-architecture-overview)
  - [Data Flow Patterns](#data-flow-patterns)
  - [System Components](#system-components)
  - [Merkle Tree Structure & Synchronization](#merkle-tree-structure--synchronization)
- [🔧 Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Quick Start Guide](#quick-start-guide)
  - [Installation & Setup](#installation--setup)
  - [Verification & Testing](#verification--testing)
- [📚 Official Client Libraries](#-official-client-libraries)
  - [Python](#python)
  - [Node.js](#nodejs)
  - [Go](#go)
  - [Java](#java)
  - [Rust](#rust)
  - [C#/.NET](#cnet)
  - [C++](#c)
  - [Ruby](#ruby)
  - [PHP](#php)
- [📡 Usage (Raw TCP Protocol)](#-usage-raw-tcp-protocol)
  - [Available Commands](#available-commands)
  - [Interactive Session Example](#interactive-session-example)
- [⚙️ Configuration](#️-configuration)
  - [Configuration File Format](#configuration-file-format)
  - [Configuration Options Reference](#configuration-options-reference)
- [🗺️ Roadmap & Implementation Issues](#️-roadmap--implementation-issues)
- [Known Issues](#known-issues)
- [🙌 Contributing](#-contributing)
- [📜 License](#-license)

---

## ✨ Key Features

### 🚀 Performance & Scalability
- **High Performance**: Built with Rust and the Tokio asynchronous runtime for low-latency and high-throughput operations
- **Horizontal Scaling**: Add or remove nodes dynamically without downtime
- **Memory Efficiency**: Optimized data structures and zero-copy operations where possible

### 🔄 Replication & Consistency
- **Real-time Replication**: Updates are immediately published via MQTT and broadcast to all peer nodes with 3-5 second latency
- **MQTT-based Distribution**: Uses public or private MQTT brokers for reliable message transport
- **Eventually Consistent**: Guarantees that all nodes will converge to the same state
- **Self-Healing**: The anti-entropy mechanism runs periodically to automatically find and fix any data drift between replicas
- **Loop Prevention**: Nodes intelligently ignore their own messages to prevent infinite replication loops
- **Bi-directional Sync**: All nodes can both send and receive updates in a peer-to-peer architecture

### 🛡️ Reliability & Safety
- **Memory Safety**: Guarantees provided by the Rust compiler prevent common bugs like null pointer dereferencing and data races
- **No Single Point of Failure**: Peer-to-peer architecture with no leader node
- **Fault Tolerance**: Continues operating even if individual nodes fail
- **Enhanced Error Handling**: Comprehensive error validation and graceful failure handling
- **Protocol Robustness**: Improved input validation and edge case handling

### 🔧 Operational Simplicity
- **Rich Protocol Support**: Extended Memcached-like protocol with support for:
  - Basic operations: `SET`, `GET`, `DEL`
  - Numeric operations: `INCR`, `DECR` with custom amounts
  - String operations: `APPEND`, `PREPEND`
  - Server commands: `VERSION`, `INFO`, `FLUSH`, `SHUTDOWN`
- **Easy Configuration**: TOML-based configuration with sensible defaults
- **Minimal Dependencies**: Only requires an MQTT broker for coordination
- **Comprehensive Testing**: Full integration test suite for reliability assurance

### 📊 Efficiency
- **Efficient Synchronization**: Merkle trees allow nodes to verify data integrity by comparing a single root hash
- **Logarithmic Sync**: Discrepancies are located with logarithmic time complexity (O(log n)) without full data scans
- **Bandwidth Optimization**: Only divergent data is synchronized, not entire datasets

---

## � Recent Improvements

### Latest Enhancements (v1.1.0)

**🔄 Real-time Replication System**
- Implemented complete MQTT-based replication infrastructure using rumqttc client
- Added support for all write operations: SET, DELETE, INCR, DECR, APPEND, PREPEND
- Created CBOR binary encoding for efficient on-wire message format
- Implemented Last-Write-Wins (LWW) conflict resolution with timestamp ordering
- Added idempotency protection and loop prevention mechanisms
- Verified bi-directional replication with 3-5 second latency on public MQTT brokers

**🧪 Comprehensive Replication Testing**
- Developed full Python-based integration test suite for replication functionality
- Added MQTT connectivity validation with test.mosquitto.org public broker
- Created interactive demonstration scripts showcasing real-time synchronization
- Implemented automated test runner with multiple execution modes
- Added comprehensive documentation for replication testing procedures

**🔧 Enhanced Protocol Support**
- Added numeric operations: `INCR` and `DECR` commands with custom amounts
- Implemented string operations: `APPEND` and `PREPEND` commands  
- Added server information commands: `VERSION`, `INFO`, `FLUSH`, `SHUTDOWN`
- Improved protocol parsing with better error detection and validation

**🛡️ Robustness Improvements**
- Fixed critical compilation issues and improved code reliability
- Enhanced error handling for edge cases and malformed input
- Better validation for special characters (newlines, tabs, Unicode)
- Improved memory safety and concurrent access patterns

**🧪 Testing Infrastructure**
- Comprehensive Python-based integration test suite
- Automated testing for all protocol commands and edge cases
- Performance benchmarking and load testing capabilities
- Continuous integration support with detailed test reporting

**📈 Performance Optimizations**
- Optimized numeric operations with proper type handling
- Enhanced string concatenation operations
- Improved error response times and memory usage
- Better handling of concurrent client connections

---

## �🏗️ Architecture

MerkleKV is a distributed key-value store designed around a peer-to-peer architecture with no single point of failure. The system consists of a cluster of `MerkleKV` nodes, where all nodes are equal peers.

### System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            MerkleKV Distributed Cluster                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐        ┌─────────────┐        ┌─────────────┐              │
│  │   Node A    │        │   Node B    │        │   Node C    │              │
│  │             │        │             │        │             │              │
│  │ ┌─────────┐ │        │ ┌─────────┐ │        │ ┌─────────┐ │              │
│  │ │Storage  │ │        │ │Storage  │ │        │ │Storage  │ │              │
│  │ │Engine   │ │        │ │Engine   │ │        │ │Engine   │ │              │
│  │ │+ Merkle │ │        │ │+ Merkle │ │        │ │+ Merkle │ │              │
│  │ │  Tree   │ │        │ │  Tree   │ │        │ │  Tree   │ │              │
│  │ └─────────┘ │        │ └─────────┘ │        │ └─────────┘ │              │
│  │             │        │             │        │             │              │
│  │ ┌─────────┐ │        │ ┌─────────┐ │        │ ┌─────────┐ │              │
│  │ │TCP      │ │        │ │TCP      │ │        │ │TCP      │ │              │
│  │ │Listener │ │        │ │Listener │ │        │ │Listener │ │              │
│  │ │:7379    │ │        │ │:7879    │ │        │ │:7880    │ │              │
│  │ └─────────┘ │        │ └─────────┘ │        │ └─────────┘ │              │
│  └─────────────┘        └─────────────┘        └─────────────┘              │
│         │                       │                       │                   │
│         └───────────────────────┼───────────────────────┘                   │
│                                 │                                           │
│  ┌──────────────────────────────┼──────────────────────────────┐            │
│  │              MQTT BROKER     │                              │            │
│  │                              │                              │            │
│  │     ┌─────────────────────────▼─────────────────────────┐    │            │
│  │     │          Topic: merkle-kv/updates              │    │            │
│  │     │                                                 │    │            │
│  │     │  Real-time update propagation                   │    │            │
│  │     │  • SET operations                               │    │            │
│  │     │  • DEL operations                               │    │            │
│  │     │  • Immediate replication                        │    │            │
│  │     └─────────────────────────────────────────────────┘    │            │
│  └───────────────────────────────────────────────────────────┘            │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     Anti-Entropy Process                           │   │
│  │                                                                     │   │
│  │  Node A ←→ Node B: Compare root hashes                            │   │
│  │  Node B ←→ Node C: Compare root hashes                            │   │
│  │  Node C ←→ Node A: Compare root hashes                            │   │
│  │                                                                     │   │
│  │  If hashes differ: Tree traversal to find inconsistencies         │   │
│  │  Repair any divergent data                                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘

   ▲                              ▲                              ▲
   │                              │                              │
┌──┴──┐                       ┌──┴──┐                       ┌──┴──┐
│     │                       │     │                       │     │
│ TCP │                       │ TCP │                       │ TCP │
│Conn │                       │Conn │                       │Conn │
└─────┘                       └─────┘                       └─────┘
Clients                       Clients                       Clients
```

### Data Flow Patterns

#### 1. Write Path (Hot Replication)
```
Client ────[SET key value]───▶ Node A
   │                             │
   │                             ▼
   │                        ┌─────────┐
   │                        │ Update  │
   │                        │ Local   │
   │                        │ Storage │
   │                        └─────────┘
   │                             │
   │                             ▼
   │                        ┌─────────┐
   │                        │Publish  │
   │                        │to MQTT  │
   │                        │ Topic   │
   │                        └─────────┘
   │                             │
   │                             ▼
   │                     ┌───────────────┐
   │                     │  MQTT Broker  │
   │                     │  broadcasts   │
   │                     │  to all nodes │
   │                     └───────────────┘
   │                             │
   │                     ┌───────┴───────┐
   │                     ▼               ▼
   │                 Node B           Node C
   │                     │               │
   │                     ▼               ▼
   │               ┌─────────┐     ┌─────────┐
   │               │ Apply   │     │ Apply   │
   │               │ Change  │     │ Change  │
   │               │ Locally │     │ Locally │
   │               └─────────┘     └─────────┘
   │
   ▼
[Response: OK]
```

#### 2. Read Path
```
Client ────[GET key]───▶ Any Node
   │                        │
   │                        ▼
   │                   ┌─────────┐
   │                   │ Lookup  │
   │                   │ in Local│
   │                   │ Storage │
   │                   └─────────┘
   │                        │
   ▼                        ▼
[Response: value]    [Return value]
```


### System Components

#### Core Components

- **🗄️ KeyValue Node**: Each node is an independent server instance, holding a complete replica of the dataset. Multiple nodes form a peer-to-peer cluster with no single point of failure.

- **💾 Storage Engine**: The core component responsible for the actual key-value data storage and the management of the Merkle tree structure. Handles all CRUD operations and maintains data integrity.

- **🔌 Client Protocol Listener**: A TCP listener that parses and handles client commands (SET, GET, DEL) using a simple, text-based protocol. Provides the interface for client applications to interact with the database.

- **🔄 Replication Module**: Manages the connection to the MQTT broker for publishing local changes and subscribing to updates from other nodes. Ensures rapid propagation of changes across the cluster.

- **🔧 Anti-Entropy Module**: A background process that periodically initiates a Merkle tree comparison with other peer nodes to detect and repair any data inconsistencies. Provides eventual consistency guarantees.

#### External Dependencies

- **📡 MQTT Broker**: An external message broker (e.g., Mosquitto, EMQ X) that facilitates the pub/sub communication for update propagation between nodes.
 
---

### Merkle Tree Structure & Synchronization

The Merkle tree is the foundation of our efficient data verification strategy. It allows us to verify the integrity of the entire dataset by comparing a single hash, and to rapidly locate inconsistencies if they exist.

#### Merkle Tree Construction

```
                            Root Hash
                               │
                        Hash(H1 + H2)
                         ┌─────┴─────┐
                        H1           H2
                   Hash(H3+H4)   Hash(H5+H6)
                   ┌────┴────┐   ┌────┴────┐
                  H3        H4  H5        H6
              Hash(K1+V1) │   │  Hash(K3+V3)
                         │   │
                      Hash(K2+V2) Hash(K4+V4)
                         │   │
                     ┌───┴─┐ ┌─┴───┐
                   K1:V1  K2:V2  K3:V3  K4:V4
                  (Leaf)  (Leaf) (Leaf) (Leaf)

Keys sorted lexicographically: K1 < K2 < K3 < K4
Each leaf = Hash(key + value)
Each internal node = Hash(left_child + right_child)
```

#### Tree Construction Process

1. **Sort Keys**: All keys in the store are sorted lexicographically
2. **Create Leaves**: Each (key, value) pair is hashed to form a leaf node: `Hash(key + value)`
3. **Build Tree**: Adjacent nodes are concatenated and hashed together to form parent nodes
4. **Repeat**: This process continues recursively until a single root hash is generated

#### Synchronization Protocol

```
Node A                                    Node B
  │                                         │
  ├─── Request root hash ──────────────────▶│
  │                                         │
  │◄─── Root hash: ABC123 ───────────────────┤
  │                                         │
  │  Compare with local root hash (XYZ789)  │
  │  ❌ Hashes don't match!                 │
  │                                         │
  ├─── Request child hashes ───────────────▶│
  │                                         │
  │◄─── Left: DEF456, Right: GHI789 ────────┤
  │                                         │
  │  Compare children:                      │
  │  ✅ Left child matches                  │
  │  ❌ Right child differs                 │
  │                                         │
  ├─── Request right subtree ──────────────▶│
  │                                         │
  │    Continue traversing down only        │
  │    the divergent branch...              │
  │                                         │
  ├─── Found inconsistent key: "user:123" ─▶│
  │                                         │
  │◄─── Correct value: "john_doe" ──────────┤
  │                                         │
  │  Update local storage                   │
  │  Recalculate affected hashes            │
```

#### Synchronization Complexity

- **Time Complexity**: O(log n) where n is the number of keys
- **Space Complexity**: O(log n) for the traversal path
- **Network Efficiency**: Only divergent branches are compared, not the entire dataset

#### Example: Finding Inconsistency

```
Node A Tree (Inconsistent)          Node B Tree (Correct)
                                   
     Root: XYZ789                       Root: ABC123
         │                                  │
    ┌────┴────┐                        ┌────┴────┐
   H1: DEF456  H2: BAD999             H1: DEF456  H2: GHI789
       │           │                      │           │
   ┌───┴───┐   ┌───┴───┐              ┌───┴───┐   ┌───┴───┐
  user:100 user:123  user:200        user:100 user:123  user:200
   alice   WRONG_VAL  charlie         alice   john_doe   charlie
           ↑                                     ↑
        Inconsistent                          Correct
        
Steps:
1. Compare roots: XYZ789 ≠ ABC123 ❌
2. Compare children: H1 matches ✅, H2 differs ❌  
3. Traverse right subtree (H2)
4. Find leaf: user:123 has different value
5. Sync: Update "user:123" from "WRONG_VAL" → "john_doe"
6. Recalculate: H2: BAD999 → GHI789, Root: XYZ789 → ABC123
```

## 🔧 Getting Started

### Prerequisites

Before setting up MerkleKV, ensure you have the following dependencies installed:

#### Required Dependencies
- **🦀 Rust Toolchain**: Version 1.70.0 or later
  ```bash
  # Install via rustup (recommended)
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source ~/.cargo/env
  
  # Verify installation
  rustc --version
  cargo --version
  ```

- **📡 MQTT Broker**: A running instance of an MQTT broker
  
  **Option 1: Mosquitto (Recommended for production)**
  ```bash
  # Ubuntu/Debian
  sudo apt-get update
  sudo apt-get install mosquitto mosquitto-clients
  
  # macOS
  brew install mosquitto
  
  # Start the broker
  sudo systemctl start mosquitto  # Linux
  brew services start mosquitto   # macOS
  ```
  
  **Option 2: Docker (Quick setup)**
  ```bash
  # Run Mosquitto in Docker
  docker run -it -p 1883:1883 -p 9001:9001 eclipse-mosquitto
  ```
  
  **Option 3: Public test broker (Development only)**
  ```
  broker: test.mosquitto.org:1883
  ```

#### Optional Dependencies
- **🐳 Docker**: For containerized deployments
- **📊 Monitoring Tools**: Prometheus, Grafana (for production monitoring)
- **🐍 Python 3.8+**: For running integration tests

### Quick Start Guide

Get MerkleKV running in under 5 minutes:

#### 1. Clone and Build
```bash
# Clone the repository
git clone https://github.com/AI-Decenter/MerkleKV.git
cd MerkleKV

# Build the project (release mode for better performance)
cargo build --release
```

#### 2. Start MQTT Broker (if needed)
```bash
# Option A: Use Docker (easiest)
docker run -d --name mosquitto -p 1883:1883 eclipse-mosquitto

# Option B: Use system mosquitto
sudo systemctl start mosquitto

# Option C: Use public broker (no setup required)
# Just use test.mosquitto.org:1883 in configuration
```

#### 3. Quick Single Node Setup
```bash
# Create minimal configuration
cat > quickstart.toml << EOF
node_id = "quickstart-node"

[network]
bind_address = "127.0.0.1"
bind_port = 7379

[mqtt]
broker_address = "tcp://test.mosquitto.org:1883"
topic_prefix = "quickstart-merkle-kv"
EOF

# Start the server
cargo run --release -- --config quickstart.toml
```

#### 4. Test Your Setup
```bash
# In a new terminal, test basic operations
echo "SET hello world" | nc localhost 7379
# Expected: OK

echo "GET hello" | nc localhost 7379  
# Expected: world

echo "VERSION" | nc localhost 7379
# Expected: MerkleKV 1.0.0
```

#### 5. Run Integration Tests
```bash
# Install Python dependencies
cd tests/integration
pip install -r requirements.txt

# Run basic test suite
python run_tests.py --mode basic
# Expected: All tests should pass ✅
```

**🎉 Congratulations!** You now have a working MerkleKV instance. Continue reading for multi-node clusters and advanced features.

### Installation & Setup

#### Quick Start (Single Node)

1. **Clone the repository**
   ```bash
   git clone https://github.com/AI-Decenter/MerkleKV.git
   cd MerkleKV
   ```

2. **Build the project**
   ```bash
   # Debug build (faster compilation)
   cargo build
   
   # Release build (optimized)
   cargo build --release
   ```

3. **Create configuration file**
   ```bash
   # Create a basic configuration
   cat > node1.toml << EOF
   node_id = "local-node"
   
   [network]
   bind_address = "127.0.0.1"
   bind_port = 7379
   
   [mqtt]
   broker_address = "tcp://localhost:1883"
   topic_prefix = "dev-merkle-kv"
   EOF
   ```

4. **Start the node**
   ```bash
   # Run with debug build
   cargo run -- --config node1.toml
   
   # Or run with release build
   cargo run --release -- --config node1.toml
   ```

5. **Test the connection**
   ```bash
   # In another terminal
   nc localhost 7379
   
   # Try some commands
   SET hello world
   GET hello
   DEL hello
   ```

#### Multi-Node Cluster Setup

For a more realistic setup with replication and anti-entropy:

1. **Create multiple configuration files**
   ```bash
   # Node 1
   cat > node1.toml << EOF
   node_id = "node-1"
   
   [network]
   bind_address = "127.0.0.1"
   bind_port = 7379
   
   [mqtt]
   broker_address = "tcp://localhost:1883"
   topic_prefix = "cluster-merkle-kv"
   client_id = "node-1"
   
   [anti_entropy]
   enabled = true
   interval_seconds = 60
   peer_list = ["127.0.0.1:7879", "127.0.0.1:7880"]
   EOF
   
   # Node 2
   cat > node2.toml << EOF
   node_id = "node-2"
   
   [network]
   bind_address = "127.0.0.1"
   bind_port = 7879
   
   [mqtt]
   broker_address = "tcp://localhost:1883"
   topic_prefix = "cluster-merkle-kv"
   client_id = "node-2"
   
   [anti_entropy]
   enabled = true
   interval_seconds = 60
   peer_list = ["127.0.0.1:7379", "127.0.0.1:7880"]
   EOF
   
   # Node 3
   cat > node3.toml << EOF
   node_id = "node-3"
   
   [network]
   bind_address = "127.0.0.1"
   bind_port = 7880
   
   [mqtt]
   broker_address = "tcp://localhost:1883"
   topic_prefix = "cluster-merkle-kv"
   client_id = "node-3"
   
   [anti_entropy]
   enabled = true
   interval_seconds = 60
   peer_list = ["127.0.0.1:7379", "127.0.0.1:7879"]
   EOF
   ```

2. **Start all nodes**
   ```bash
   # Terminal 1: Start node 1
   cargo run --release -- --config node1.toml
   
   # Terminal 2: Start node 2
   cargo run --release -- --config node2.toml
   
   # Terminal 3: Start node 3
   cargo run --release -- --config node3.toml
   ```

3. **Test replication**
   ```bash
   # Connect to node 1
   echo "SET user:alice \"Alice Johnson\"" | nc localhost 7379
   
   # Connect to node 2 (different terminal)
   echo "GET user:alice" | nc localhost 7879  # Should return "Alice Johnson"
   
   # Connect to node 3 (different terminal)
   echo "GET user:alice" | nc localhost 7880  # Should return "Alice Johnson"
   ```

### Verification & Testing

#### Health Check
```bash
# Check if nodes are listening
netstat -tlnp | grep 787

# Test connectivity to each node
for port in 7379 7879 7880; do
  echo "Testing port $port..."
  echo "GET test" | nc localhost $port
done
```

#### MQTT Connectivity Test
```bash
# Subscribe to updates (requires mosquitto-clients)
mosquitto_sub -h localhost -t "cluster-merkle-kv/updates"

# In another terminal, make changes
echo "SET test:mqtt success" | nc localhost 7379

# Should see MQTT messages in the subscriber
```

#### Anti-Entropy Verification
```bash
# Stop node 2 temporarily
# Make changes to node 1
echo "SET offline:test value" | nc localhost 7379

# Restart node 2
cargo run --release -- --config node2.toml

# Wait for anti-entropy sync (60 seconds)
# Check if the change propagated
echo "GET offline:test" | nc localhost 7879
```

#### Comprehensive Testing Suite

MerkleKV includes a comprehensive Python-based integration test suite located in `tests/integration/`. These tests validate all server functionality including error handling, protocol compliance, and edge cases.

##### Prerequisites for Testing
```bash
cd tests/integration

# Install Python dependencies
pip install -r requirements.txt

# Or using a virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
```

##### Running Tests

**Quick Basic Tests**:
```bash
# Run essential functionality tests
python run_tests.py --mode basic

# Expected output:
# ✅ Basic Operations: PASSED
# ✅ Error Handling: PASSED  
# ✅ Numeric Operations: PASSED
# Summary: 3/3 tests passed
```

**Full Test Suite**:
```bash
# Run all integration tests
python run_tests.py --mode all

# Expected output:
# ✅ Basic Operations: PASSED
# ✅ Bulk Operations: PASSED
# ✅ Concurrency Tests: PASSED
# ✅ Error Handling: PASSED
# ✅ Numeric Operations: PASSED
# ✅ Statistical Commands: PASSED
# ✅ Benchmark Tests: PASSED
# Summary: 7/7 tests passed
```

**Verbose Test Output**:
```bash
# Run with detailed output
python run_tests.py --mode all --verbose

# Shows individual test cases and timing
# Useful for debugging specific issues
```

**Individual Test Modules**:
```bash
# Run specific test categories
pytest test_basic_operations.py -v     # Basic SET/GET/DEL
pytest test_error_handling.py -v       # Error conditions and edge cases
pytest test_numeric_operations.py -v   # INCR/DECR operations  
pytest test_concurrency.py -v          # Multi-client concurrent access
pytest test_bulk_operations.py -v      # Large dataset operations
pytest test_statistical_commands.py -v # Server info and statistics
pytest test_benchmark.py -v            # Performance benchmarks
```

##### Test Categories

**1. Basic Operations (`test_basic_operations.py`)**
- SET/GET/DEL command validation
- Data persistence verification
- Protocol compliance testing
- Response format validation

**2. Error Handling (`test_error_handling.py`)**
- Invalid command detection
- Malformed input handling
- Special character support (Unicode, newlines, tabs)
- Edge case validation
- Protocol error responses

**3. Numeric Operations (`test_numeric_operations.py`)**
- INCR/DECR functionality
- Custom increment/decrement amounts
- Non-existent key handling
- Non-numeric value error handling
- Boundary value testing

**4. String Operations (included in basic tests)**
- APPEND/PREPEND functionality
- String concatenation validation
- Empty string handling
- Unicode string support

**5. Concurrency Tests (`test_concurrency.py`)**
- Multiple client connections
- Concurrent read/write operations
- Race condition prevention
- Connection handling under load

**6. Bulk Operations (`test_bulk_operations.py`)**
- Large dataset handling
- Memory efficiency testing
- Performance under load
- Bulk data validation

**7. Server Information (`test_statistical_commands.py`)**
- VERSION command testing
- INFO command validation
- Server statistics accuracy
- Uptime tracking

**8. Benchmark Tests (`test_benchmark.py`)**
- Throughput measurements
- Latency testing
- Memory usage monitoring
- Performance regression detection

##### Test Configuration

Tests can be configured via `conftest.py`:

```python
# Default test configuration
SERVER_HOST = "127.0.0.1" 
SERVER_PORT = 7379
SERVER_START_TIMEOUT = 10
TEST_TIMEOUT = 30

# Custom configuration for CI/CD
pytest test_basic_operations.py --host=192.168.1.10 --port=7879
```

##### Continuous Integration

For automated testing in CI/CD pipelines:

```bash
# Run tests with JUnit XML output
python run_tests.py --mode all --output junit

# Generate test coverage report
python run_tests.py --mode all --coverage

# Run performance benchmarks
python run_tests.py --mode benchmark --threshold=1000ops
```

##### Test Development

When adding new features, include corresponding tests:

```python
# Example test structure
def test_new_feature():
    """Test new feature functionality."""
    client = connect_to_server()
    
    # Test normal operation
    response = send_command(client, "NEW_COMMAND arg1 arg2")
    assert response == "EXPECTED_RESULT"
    
    # Test error conditions
    response = send_command(client, "NEW_COMMAND invalid")
    assert "ERROR" in response
    
    client.close()
```

### Troubleshooting

#### Common Issues

**"Connection refused" errors**
```bash
# Check if the node is running
ps aux | grep merkle

# Check if the port is in use
sudo lsof -i :7379

# Check firewall settings
sudo ufw status
```

**MQTT connection failures**
```bash
# Test MQTT broker connectivity
mosquitto_pub -h localhost -t test -m "hello"

# Check broker logs
sudo journalctl -u mosquitto -f
```

**Anti-entropy not working**
```bash
# Check node logs for sync attempts
grep "anti-entropy" /var/log/merkle-kv/*.log

# Verify peer connectivity
nc -v <peer-ip> <peer-port>
```

### Performance Tuning

For production deployments:

```toml
[network]
max_connections = 10000
connection_timeout = 60

[storage]
memory_limit_mb = 8192  # 8GB

[anti_entropy]
interval_seconds = 300   # 5 minutes
max_concurrent_syncs = 5

[logging]
level = "warn"  # Reduce log verbosity
```

---

## 📚 Official Client Libraries

MerkleKV provides official client libraries for 9 popular programming languages, making it easy to integrate with your applications. All clients implement the MerkleKV TCP protocol with consistent behavior across languages.

### Python

**Installation:**
```bash
pip install merklekv
```

**Quick Start:**
```python
from merklekv import MerkleKVClient

with MerkleKVClient("localhost", 7379) as client:
    client.set("user:123", "john_doe")    # Store key-value
    value = client.get("user:123")         # Returns "john_doe"
    client.delete("user:123")              # Remove key
```

**Documentation:** [`clients/python/README.md`](clients/python/README.md)

### Node.js

**Installation:**
```bash
npm install @merklekv/client
```

**Quick Start:**
```javascript
const { MerkleKVClient } = require('@merklekv/client');

const client = new MerkleKVClient('localhost', 7379);
await client.connect();

await client.set('user:123', 'john_doe'); // Store key-value
const value = await client.get('user:123'); // Returns 'john_doe'
await client.delete('user:123');             // Remove key

await client.close();
```

**Documentation:** [`clients/nodejs/README.md`](clients/nodejs/README.md)

### Go

**Installation:**
```bash
go get github.com/AI-Decenter/MerkleKV/clients/go
```

**Quick Start:**
```go
import merklekv "github.com/AI-Decenter/MerkleKV/clients/go"

client := merklekv.New("localhost", 7379)
defer client.Close()

client.Connect()
client.Set("user:123", "john_doe")    // Store key-value
value, _ := client.Get("user:123")    // Returns "john_doe"
client.Delete("user:123")             // Remove key
```

**Documentation:** [`clients/go/README.md`](clients/go/README.md)

### Java

**Installation:**
```xml
<dependency>
    <groupId>io.merklekv</groupId>
    <artifactId>merklekv-client-java</artifactId>
    <version>1.0.0</version>
</dependency>
```

**Quick Start:**
```java
import io.merklekv.client.MerkleKVClient;

try (MerkleKVClient client = new MerkleKVClient("localhost", 7379)) {
    client.set("user:123", "john_doe");     // Store key-value
    String value = client.get("user:123");  // Returns "john_doe"
    client.delete("user:123");              // Remove key
}
```

**Documentation:** [`clients/java/README.md`](clients/java/README.md)

### Rust

**Installation:**
```toml
[dependencies]
merklekv-client = "1.0.0"
```

**Quick Start:**
```rust
use merklekv_client::Client;

let mut client = Client::connect("127.0.0.1:7379")?;
client.set("user:123", "john_doe")?;     // Store key-value
let value = client.get("user:123")?;     // Returns Some("john_doe")
client.delete("user:123")?;              // Remove key
```

**Documentation:** [`clients/rust/README.md`](clients/rust/README.md)

### C#/.NET

**Installation:**
```bash
dotnet add package MerkleKV.Client
```

**Quick Start:**
```csharp
using MerkleKV;

using var client = new MerkleKvClient("localhost", 7379);
await client.ConnectAsync();

await client.SetAsync("user:123", "john_doe"); // Store key-value
var value = await client.GetAsync("user:123");  // Returns "john_doe"
await client.DeleteAsync("user:123");           // Remove key
```

**Documentation:** [`clients/dotnet/README.md`](clients/dotnet/README.md)

### C++

**Installation:**
```bash
# CMake
find_package(MerkleKV REQUIRED)
target_link_libraries(your_target MerkleKV::Client)
```

**Quick Start:**
```cpp
#include <merklekv/client.hpp>

MerkleKvClient client("localhost", 7379);
client.connect();

client.set("user:123", "john_doe");      // Store key-value
auto value = client.get("user:123");     // Returns optional<string>
client.del("user:123");                  // Remove key
```

**Documentation:** [`clients/cpp/README.md`](clients/cpp/README.md)

### Ruby

**Installation:**
```bash
gem install merklekv
```

**Quick Start:**
```ruby
require 'merklekv'

client = MerkleKV::Client.new(host: "localhost", port: 7379)
client.set("user:123", "john_doe")       # Store key-value
value = client.get("user:123")           # Returns "john_doe"
client.delete("user:123")                # Remove key
client.close
```

**Documentation:** [`clients/ruby/README.md`](clients/ruby/README.md)

### PHP

**Installation:**
```bash
composer require merklekv/client
```

**Quick Start:**
```php
<?php
use MerkleKV\Client;

$client = new Client("localhost", 7379);
$client->set("user:123", "john_doe");    // Store key-value
$value = $client->get("user:123");       // Returns "john_doe"
$client->delete("user:123");             // Remove key
$client->close();
```

**Documentation:** [`clients/php/README.md`](clients/php/README.md)

### Protocol Notes

All clients implement consistent protocol behavior with observed server responses:

- **CRLF Termination**: Commands are terminated with `\r\n`
- **Unicode Support**: Full UTF-8 encoding for keys and values
- **GET Responses**: Returns `VALUE <data>` for existing keys, `(null)` for missing keys
- **DELETE Responses**: Returns `OK` for both existing and non-existing keys
- **Empty Values**: Empty strings are represented as `""`
- **Control Characters**: Tab (`\t`) and newline (`\n`) characters are rejected by the server
- **Error Handling**: Consistent error types across all languages
- **Timeouts**: Configurable operation timeouts (default 5 seconds)

### Known Issues

**Note**: The major server-side issues (large value parser corruption, DELETE response semantics, control character handling) have been resolved as of the latest version.

For complete API documentation and advanced usage examples, see the individual client library documentation in the [`clients/`](clients/) directory.

---

## 📡 Usage (Raw TCP Protocol)

MerkleKV uses a simple, text-based protocol similar to Memcached. You can interact with any `MerkleKV` node using standard TCP clients like `netcat` (nc) or custom applications.

### Protocol Overview

- **Connection**: TCP connection to any node in the cluster
- **Format**: All commands must end with `\r\n` (CRLF)
- **Responses**: Simple text responses for easy parsing
- **Encoding**: UTF-8 text encoding

### Connecting to a Node

```bash
# Basic connection
nc localhost 7379

# Using netcat with timeout
nc -w 5 localhost 7379

# Using netcat with verbose output
nc -v localhost 7379
```

### Available Commands

#### Basic Operations

##### SET Command
Store a key-value pair in the distributed store.

**Syntax**: `SET <key> <value>\r\n`

```bash
# Store a user record
SET user:100 jane_doe
OK

# Store JSON-like data (as string)
SET config:database {"host":"localhost","port":5432}
OK

# Store with spaces in value
SET message:1 Hello World from MerkleKV
OK
```

**Response**: `OK` on success, error message on failure.

##### GET Command
Retrieve a value by its key from the local node.

**Syntax**: `GET <key>\r\n`

```bash
# Get a user record
GET user:100
jane_doe

# Get non-existent key
GET user:999
(null)

# Get configuration
GET config:database
{"host":"localhost","port":5432}
```

**Response**: The value if found, `(null)` if key doesn't exist.

##### DEL Command
Delete a key and its associated value from the distributed store.

**Syntax**: `DEL <key>\r\n`

```bash
# Delete a user record
DEL user:100
DELETED

# Delete non-existent key
DEL user:999
NOT_FOUND
```

**Response**: `DELETED` on successful deletion, `NOT_FOUND` if key doesn't exist.

#### Numeric Operations

##### INCR Command
Increment a numeric value stored at a key.

**Syntax**: `INCR <key> [amount]\r\n`

```bash
# Increment by 1 (default)
SET counter 10
INCR counter
11

# Increment by custom amount
INCR counter 5
16

# Increment non-existent key (starts at 0)
INCR new_counter
1
```

**Response**: The new value after increment, or error if value is not numeric.

##### DECR Command
Decrement a numeric value stored at a key.

**Syntax**: `DECR <key> [amount]\r\n`

```bash
# Decrement by 1 (default)
SET counter 10
DECR counter
9

# Decrement by custom amount
DECR counter 3
6

# Decrement non-existent key (starts at 0)
DECR new_counter
-1
```

**Response**: The new value after decrement, or error if value is not numeric.

#### String Operations

##### APPEND Command
Append a value to an existing string.

**Syntax**: `APPEND <key> <value>\r\n`

```bash
# Append to existing string
SET greeting "Hello"
APPEND greeting " World!"
Hello World!

# Append to non-existent key (creates new)
APPEND new_message "Start"
Start
```

**Response**: The concatenated string value.

##### PREPEND Command
Prepend a value to an existing string.

**Syntax**: `PREPEND <key> <value>\r\n`

```bash
# Prepend to existing string
SET greeting "World!"
PREPEND greeting "Hello "
Hello World!

# Prepend to non-existent key (creates new)
PREPEND new_message "Start"
Start
```

**Response**: The concatenated string value.

#### Server Information Commands

##### VERSION Command
Get server version information.

**Syntax**: `VERSION\r\n`

```bash
VERSION
MerkleKV 1.0.0
```

##### INFO Command
Get detailed server information including uptime and configuration.

**Syntax**: `INFO\r\n`

```bash
INFO
# Server Information
version:1.0.0
uptime_seconds:3600
node_id:node-alpha
memory_usage_mb:128
total_keys:1500
```

##### FLUSH Command
Clear all data from the server (development/testing only).

**Syntax**: `FLUSH\r\n`

```bash
FLUSH
OK
```

**⚠️ Warning**: This command removes all data and should only be used in development environments.

##### SHUTDOWN Command
Gracefully shutdown the server.

**Syntax**: `SHUTDOWN\r\n`

```bash
SHUTDOWN
OK
```

**Response**: Server will close the connection and terminate.

### Interactive Session Example

```bash
$ nc -v localhost 7379
Connection to localhost 7379 port [tcp/*] succeeded!

# Store some data
SET user:alice Alice Smith
OK

SET user:bob Bob Johnson  
OK

SET counter:views 1500
OK

# Retrieve data
GET user:alice
Alice Smith

GET counter:views
1500

# Numeric operations
INCR counter:views
1501

INCR counter:views 10
1511

DECR counter:views 5
1506

# String operations
SET greeting Hello
OK

APPEND greeting " World!"
Hello World!

PREPEND greeting "Hi, "
Hi, Hello World!

# Server information
VERSION
MerkleKV 1.0.0

INFO
# Server Information
version:1.0.0
uptime_seconds:3600
node_id:node-alpha
memory_usage_mb:128
total_keys:5

# Update existing data
SET counter:views 2000
OK

GET counter:views
2000

# Delete data
DEL user:bob
DELETED

GET user:bob
(null)

# Close connection
# Use Ctrl+C or Ctrl+D to exit
Connection closed.
```

### Client Library Integration

While MerkleKV uses a simple text protocol, you can easily integrate it into applications:

#### Python Example
```python
import socket

def merkle_kv_client(host, port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((host, port))
    return sock

# Usage
client = merkle_kv_client('localhost', 7379)
client.send(b'SET user:123 john_doe\r\n')
response = client.recv(1024).decode().strip()
print(f"Response: {response}")  # Output: Response: OK
```

#### Bash Script Example
```bash
#!/bin/bash
exec 3<>/dev/tcp/localhost/7379
echo "SET script:status running" >&3
read response <&3
echo "Response: $response"
exec 3<&-
exec 3>&-
```

### Replication Demonstration

MerkleKV includes a comprehensive replication testing suite that demonstrates real-time synchronization between nodes.

#### Quick Replication Test
```bash
# Navigate to test directory
cd tests/integration

# Run simple replication verification
python run_replication_tests.py simple

# Expected output:
# 🧪 Running simple replication test...
# ✅ MQTT broker connectivity: PASSED
# ✅ Basic replication: PASSED
# ✅ All tests completed successfully!
```

#### Interactive Replication Demo
```bash
# Run interactive demonstration
python demo_replication.py

# Expected output:
# 🔥 MerkleKV Replication Demo
# 🚀 Starting Node 1 (Port 7600)...
# 🚀 Starting Node 2 (Port 7601)...
# ✅ Node 1 ready
# ✅ Node 2 ready
# 
# 📝 Demo 1: Basic SET operation replication
# → Setting 'user:alice' = 'Alice Johnson' on Node 1...
# → Waiting for replication (3 seconds)...
# → Getting 'user:alice' from Node 2...
# ✅ Replication successful!
```

#### Multi-Node Cluster Replication
```bash
# Test comprehensive replication scenarios
python run_replication_tests.py all

# Includes tests for:
# - SET/DELETE operation replication
# - Numeric operations (INCR/DECR) sync
# - String operations (APPEND/PREPEND) sync
# - Concurrent operations handling
# - Loop prevention validation
# - Malformed message handling
```

### Error Handling

Common error responses:
- `ERROR: Invalid command` - Unknown command sent
- `ERROR: Missing arguments` - Command missing required parameters  
- `ERROR: Connection failed` - Network or node issues
- `ERROR: Storage full` - Node storage capacity exceeded (if limits configured)
## ⚙️ Configuration

MerkleKV nodes are configured using TOML files, providing a simple and readable configuration format. Each node requires its own configuration file to specify network settings, identification, and operational parameters.

### Configuration File Format

#### Basic Configuration Example (`node1.toml`)

```toml
# Node Identity
node_id = "node-alpha"
description = "Primary MerkleKV node in us-east datacenter"

# Network Configuration
[network]
bind_address = "127.0.0.1"
bind_port = 7379
max_connections = 1000

# MQTT Broker Settings
[mqtt]
broker_address = "tcp://test.mosquitto.org:1883"
topic_prefix = "merkle-kv"
client_id = "merkle-node-alpha"
keep_alive = 60
clean_session = true

# Anti-Entropy Settings
[anti_entropy]
enabled = true
interval_seconds = 300  # 5 minutes
peer_discovery_interval = 600  # 10 minutes
max_concurrent_syncs = 3

# Storage Configuration
[storage]
memory_limit_mb = 1024  # 1GB memory limit
persistence_enabled = false
backup_interval_hours = 24

# Logging Configuration  
[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # json, text
file_path = "/var/log/merkle-kv/node-alpha.log"
```

#### Multi-Node Cluster Example

**Node 1 Configuration (`node1.toml`)**
```toml
node_id = "node-primary"

[network]
bind_address = "192.168.1.10"
bind_port = 7379

[mqtt]
broker_address = "tcp://192.168.1.100:1883"
topic_prefix = "prod-merkle-kv"
client_id = "primary-node"

[anti_entropy]
enabled = true
interval_seconds = 180
peer_list = ["192.168.1.11:7379", "192.168.1.12:7379"]
```

**Node 2 Configuration (`node2.toml`)**
```toml
node_id = "node-secondary"

[network]
bind_address = "192.168.1.11" 
bind_port = 7379

[mqtt]
broker_address = "tcp://192.168.1.100:1883"
topic_prefix = "prod-merkle-kv"
client_id = "secondary-node"

[anti_entropy]
enabled = true
interval_seconds = 180
peer_list = ["192.168.1.10:7379", "192.168.1.12:7379"]
```

**Node 3 Configuration (`node3.toml`)**
```toml
node_id = "node-tertiary"

[network]
bind_address = "192.168.1.12"
bind_port = 7379

[mqtt]
broker_address = "tcp://192.168.1.100:1883" 
topic_prefix = "prod-merkle-kv"
client_id = "tertiary-node"

[anti_entropy]
enabled = true
interval_seconds = 180
peer_list = ["192.168.1.10:7379", "192.168.1.11:7379"]
```

### Configuration Options Reference

#### Core Settings
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `node_id` | String | Required | Unique identifier for this node in the cluster |
| `description` | String | Optional | Human-readable description of the node |

#### Network Section `[network]`
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `bind_address` | String | "127.0.0.1" | IP address to bind the TCP listener |
| `bind_port` | Integer | 7379 | Port number for client connections |
| `max_connections` | Integer | 1000 | Maximum concurrent client connections |
| `connection_timeout` | Integer | 30 | Connection timeout in seconds |

#### MQTT Section `[mqtt]`
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `broker_address` | String | Required | MQTT broker connection string |
| `topic_prefix` | String | "merkle-kv" | Prefix for MQTT topics |
| `client_id` | String | node_id | MQTT client identifier |
| `username` | String | Optional | MQTT authentication username |
| `password` | String | Optional | MQTT authentication password |
| `keep_alive` | Integer | 60 | MQTT keep-alive interval in seconds |
| `clean_session` | Boolean | true | MQTT clean session flag |

#### Anti-Entropy Section `[anti_entropy]`
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `enabled` | Boolean | true | Enable/disable anti-entropy mechanism |
| `interval_seconds` | Integer | 300 | Interval between sync attempts |
| `peer_discovery_interval` | Integer | 600 | Interval for peer discovery |
| `max_concurrent_syncs` | Integer | 3 | Maximum concurrent synchronizations |
| `peer_list` | Array | [] | Static list of known peer addresses |

### Running with Configuration

```bash
# Start node with specific config
cargo run --release -- --config /path/to/node1.toml

# Start multiple nodes for testing
cargo run --release -- --config configs/node1.toml &
cargo run --release -- --config configs/node2.toml &
cargo run --release -- --config configs/node3.toml &

# Verify nodes are running
ps aux | grep merkle-kv
```

### Environment Variable Overrides

Configuration values can be overridden using environment variables:

```bash
# Override node ID
export MERKLE_KV_NODE_ID="production-node-1"

# Override MQTT broker
export MERKLE_KV_MQTT_BROKER_ADDRESS="tcp://production-mqtt.example.com:1883"

# Override bind port
export MERKLE_KV_NETWORK_BIND_PORT=8080

# Start with overrides
cargo run --release -- --config node.toml
```

### Configuration Validation

The system validates configuration on startup:

- ✅ **Required fields**: `node_id`, `mqtt.broker_address`
- ✅ **Port ranges**: 1-65535 for `bind_port`
- ✅ **IP addresses**: Valid IPv4/IPv6 for `bind_address`
- ✅ **MQTT URLs**: Valid broker connection strings
- ✅ **Node ID uniqueness**: Within the cluster (checked at runtime)
## 🗺️ Roadmap & Implementation Status

We have an exciting future planned for MerkleKV! Below is a comprehensive status list of implemented features and upcoming enhancements.

### Phase 1: Core Foundation ✅ COMPLETED
**Priority: High** - Essential components for basic functionality

- [x] **Issue #1: Project Structure & Build System** ✅
  - ✅ Set up Cargo workspace and project structure
  - ✅ Configure dependencies (tokio, serde, toml, etc.)
  - ✅ Set up CI/CD pipeline and testing framework
  - ✅ Create comprehensive configuration system

- [x] **Issue #2: Storage Engine Foundation** ✅
  - ✅ Implement in-memory key-value storage with HashMap
  - ✅ Add thread-safe access patterns using RwLock/Mutex
  - ✅ Create enhanced CRUD operations (Create, Read, Update, Delete)
  - ✅ Implement key-value serialization/deserialization

- [x] **Issue #3: Merkle Tree Implementation** ✅
  - ✅ Design and implement Merkle tree data structure
  - ✅ Add lexicographical key sorting functionality
  - ✅ Implement hash computation for leaf and internal nodes
  - ✅ Create root hash calculation and tree traversal methods

### Phase 2: Client Interface ✅ COMPLETED
**Priority: High** - User-facing functionality

- [x] **Issue #4: TCP Protocol Listener** ✅
  - ✅ Implement asynchronous TCP server using Tokio
  - ✅ Create enhanced command parser for all operations
  - ✅ Add comprehensive protocol validation and error handling
  - ✅ Implement response formatting and client communication

- [x] **Issue #5: Client Command Processing** ✅
  - ✅ Integrate TCP listener with storage engine
  - ✅ Add command execution logic for all operations
  - ✅ Implement proper error responses and status codes
  - ✅ Add comprehensive logging and monitoring capabilities

### Phase 2.5: Enhanced Protocol Support ✅ COMPLETED
**Priority: High** - Extended functionality beyond basic KV operations

- [x] **Issue #5.1: Numeric Operations** ✅
  - ✅ Implement `INCR` command with custom amounts
  - ✅ Implement `DECR` command with custom amounts
  - ✅ Add proper numeric validation and error handling
  - ✅ Handle non-existent keys (default to 0)

- [x] **Issue #5.2: String Operations** ✅
  - ✅ Implement `APPEND` command for string concatenation
  - ✅ Implement `PREPEND` command for string prefixing
  - ✅ Handle non-existent keys (create new entries)
  - ✅ Add Unicode and special character support

- [x] **Issue #5.3: Server Information Commands** ✅
  - ✅ Implement `VERSION` command
  - ✅ Implement `INFO` command with detailed server stats
  - ✅ Implement `FLUSH` command for data clearing
  - ✅ Implement `SHUTDOWN` command for graceful termination

- [x] **Issue #5.4: Enhanced Error Handling** ✅
  - ✅ Improve protocol parsing robustness
  - ✅ Add comprehensive input validation
  - ✅ Handle edge cases and malformed input gracefully
  - ✅ Enhance error message clarity and consistency

### Phase 3: Distributed System Core ✅ COMPLETED
**Priority: High** - Replication and consistency

- [x] **Issue #6: MQTT Integration** ✅
  - ✅ Integrated MQTT client library (rumqttc) for robust connectivity
  - ✅ Implemented connection management with automatic reconnection logic
  - ✅ Created message publishing system for local write operations
  - ✅ Added subscription handling for remote updates from other nodes
  - ✅ Implemented MQTT event loop with proper error handling
  - ✅ Added support for public MQTT brokers (test.mosquitto.org)

- [x] **Issue #7: Replication Module** ✅
  - ✅ Designed comprehensive ChangeEvent format with CBOR serialization
  - ✅ Implemented real-time change publishing for all write operations (SET, DELETE, INCR, DECR, APPEND, PREPEND)
  - ✅ Added remote change application logic with idempotency protection
  - ✅ Implemented Last-Write-Wins (LWW) conflict resolution strategy
  - ✅ Added loop prevention mechanism to avoid processing own messages
  - ✅ Created comprehensive Python test suite for replication validation
  - ✅ Verified bi-directional replication functionality with 3-5s latency

- [ ] **Issue #8: Anti-Entropy Mechanism**
  - Implement periodic peer discovery and selection
  - Create Merkle tree comparison protocol
  - Add efficient tree traversal for inconsistency detection
  - Implement repair operations for divergent data

### Phase 4: Advanced Features 📋 PLANNED
**Priority: Medium** - Performance and reliability improvements

- [x] **Issue #9: Configuration Management** ✅
  - ✅ Implement TOML configuration parsing
  - ✅ Add runtime configuration validation
  - [ ] Create configuration hot-reloading capability
  - ✅ Add environment variable override support

- [ ] **Issue #10: Persistent Storage Backend**
  - Integrate Sled embedded database for disk persistence
  - Implement write-ahead logging (WAL) for durability
  - Add database recovery and initialization logic
  - Create storage engine abstraction layer

- [x] **Issue #11: Performance Optimizations** ✅
  - ✅ Implement connection pooling and reuse
  - ✅ Add batching for multiple operations
  - ✅ Optimize Merkle tree updates and caching
  - ✅ Add metrics collection and performance monitoring

### Phase 5: Production Readiness 📋 PLANNED
**Priority: Low** - Enhanced robustness and features

- [ ] **Issue #12: Advanced Conflict Resolution**
  - Implement Vector Clocks for causality tracking
  - Add sophisticated conflict detection algorithms
  - Create conflict resolution strategies (manual/automatic)
  - Add conflict history and audit trails

- [ ] **Issue #13: Monitoring & Observability**
  - Implement structured logging with tracing
  - Add Prometheus metrics endpoint
  - Create health check endpoints
  - Add distributed tracing support

- [ ] **Issue #14: Security & Authentication**
  - Add TLS support for client connections
  - Implement MQTT authentication mechanisms
  - Add access control and authorization
  - Create secure configuration management

- [x] **Issue #15: Documentation & Tooling** ✅
  - ✅ Create comprehensive API documentation
  - ✅ Add deployment guides and examples
  - ✅ Create comprehensive integration test suite
  - ✅ Add benchmarking and testing tools

### Testing & Quality Assurance ✅ COMPLETED

- [x] **Comprehensive Test Suite** ✅
  - ✅ Basic operations testing (SET/GET/DEL)
  - ✅ Numeric operations testing (INCR/DECR)
  - ✅ String operations testing (APPEND/PREPEND)
  - ✅ Error handling and edge case validation
  - ✅ Concurrency and load testing
  - ✅ Protocol compliance verification
  - ✅ Performance benchmarking

- [x] **Replication Testing Infrastructure** ✅
  - ✅ MQTT connectivity validation with public brokers
  - ✅ Real-time replication testing between multiple nodes
  - ✅ Bi-directional synchronization verification
  - ✅ Loop prevention and idempotency testing
  - ✅ Interactive demonstration scripts for replication
  - ✅ Comprehensive test documentation and procedures

- [x] **Development Tools** ✅
  - ✅ Automated test runner with multiple modes
  - ✅ Integration with pytest framework
  - ✅ Continuous integration support
  - ✅ Performance regression detection

---

## Known Issues

### Recently Fixed Issues ✅

The following server-side issues have been resolved:

- **Large Value Parser Corruption**: ✅ **Fixed** - Server now handles values of arbitrary size using streaming line-based reading instead of fixed 1KB chunks.

- **DELETE Response Semantics**: ✅ **Fixed** - Server now returns `DELETED` for existing keys and `NOT_FOUND` for non-existing keys instead of always returning `OK`.

- **Control Character Handling**: ✅ **Fixed** - Server now allows tab (`\t`) characters in values while properly rejecting them in keys and commands. Note: Newline characters (`\n`) cannot be used in values due to the text protocol design (commands are terminated by CRLF).

### Protocol Behavior Notes

- **GET Response Format**: The server returns `VALUE <data>` instead of just `<data>`. Client libraries automatically strip the `VALUE ` prefix.

For a complete list of implementation details across all client libraries, see the [client implementation summary](clients/IMPLEMENTATION_SUMMARY.md).

---

## 🙌 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

### Ways to Contribute

- 🐛 **Report Bugs**: Open an issue with detailed reproduction steps
- 💡 **Suggest Features**: Share ideas for new functionality
- 📝 **Improve Documentation**: Help make docs clearer and more comprehensive
- 🔧 **Submit Code**: Fix bugs, implement features, or optimize performance
- 🧪 **Add Tests**: Improve test coverage and reliability
- 📊 **Performance Testing**: Benchmark and optimize the system

### Development Process

1. **Fork the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/MerkleKV.git
   cd MerkleKV
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   # or
   git checkout -b fix/bug-description
   ```

3. **Make your changes**
   ```bash
   # Follow Rust conventions
   cargo fmt
   cargo clippy
   cargo test
   ```

4. **Commit your changes**
   ```bash
   git commit -m "feat: add amazing feature"
   # Use conventional commit format:
   # feat: new feature
   # fix: bug fix
   # docs: documentation changes
   # test: add or update tests
   # refactor: code refactoring
   ```

5. **Push and create Pull Request**
   ```bash
   git push origin feature/amazing-feature
   ```

### Code Style & Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add tests for new functionality
- Update documentation for API changes
- Keep commits atomic and well-described

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_merkle_tree

# Run integration tests
cargo test --test integration_tests
```

### Getting Help

- 💬 **Discussions**: Use GitHub Discussions for questions
- 🐛 **Issues**: Report bugs and feature requests
- 📧 **Email**: Contact maintainers for sensitive issues

For major changes, please open an issue first to discuss what you would like to change.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2025 AI-Decenter

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
```

