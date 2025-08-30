# MQTT Client ID and Password Configuration

This implementation adds support for MQTT client ID and password configuration using config files with environment variable overrides.

## Features Implemented

### 1. Configuration Structure Updates

- Added `client_password` field to `ReplicationConfig` structure
- Made `client_password` optional using `#[serde(default)]`
- Updated documentation to explain environment variable support

### 2. Environment Variable Support

The following environment variables are supported with **top priority**:

- `CLIENT_ID`: Overrides `replication.client_id` from config file
- `CLIENT_PASSWORD`: Overrides `replication.client_password` from config file
- `MQTT_BROKER`: Overrides `replication.mqtt_broker` from config file
- `MQTT_PORT`: Overrides `replication.mqtt_port` from config file
- `TOPIC_PREFIX`: Overrides `replication.topic_prefix` from config file

### 3. Priority Order

1. **Environment variables** (highest priority)
2. **Config file values** (medium priority)  
3. **Default values** (lowest priority)

## Configuration File Format

```toml
host = "127.0.0.1"
port = 7379
storage_path = "data"
engine = "rwlock"
sync_interval_seconds = 60

[replication]
enabled = true
mqtt_broker = "localhost"
mqtt_port = 1883
topic_prefix = "merkle_kv"
client_id = "node1"
client_password = "your_mqtt_password"  # Optional
```

## Usage Examples

### 1. Use Config File Values Only
```bash
cargo run --bin merkle_kv config.toml
```

### 2. Override Client ID
```bash
CLIENT_ID=my_node cargo run --bin merkle_kv config.toml
```

### 3. Override Multiple Values
```bash
CLIENT_ID=my_node CLIENT_PASSWORD=secret MQTT_BROKER=prod.mqtt.com cargo run --bin merkle_kv config.toml
```

### 4. Override MQTT Connection Details
```bash
MQTT_BROKER=mqtt.example.com MQTT_PORT=8883 TOPIC_PREFIX=production cargo run --bin merkle_kv config.toml
```

### 5. Use Defaults with Environment Overrides
```bash
CLIENT_ID=my_node CLIENT_PASSWORD=secret MQTT_BROKER=localhost MQTT_PORT=1883 TOPIC_PREFIX=dev cargo run --bin merkle_kv
```

## Implementation Details

### Code Changes

1. **src/config.rs**:
   - Added `client_password: Option<String>` field to `ReplicationConfig`
   - Updated `Config::load()` to apply environment variable overrides
   - Updated `Config::default()` to apply environment variable overrides
   - Added comprehensive documentation
   - Added test for environment variable functionality

2. **src/replication.rs**:
   - Updated `Replicator::new()` to handle environment variable overrides for all MQTT settings
   - Added MQTT credentials configuration when password is available
   - Used consistent resolution for client_id, mqtt_broker, mqtt_port, and topic_prefix

3. **config.toml**:
   - Added example `client_password` field (commented out)
   - Updated documentation

### MQTT Authentication

When a password is configured (either from config file or environment variable), the MQTT client will use authentication:

```rust
// Set credentials if password is available
if let Some(password) = client_password {
    mqtt_options.set_credentials(&client_id, &password);
}
```

### Environment Variable Resolution

Environment variables are checked and applied in both `Config::load()` and `Config::default()` methods:

```rust
// Override client_id from environment variable if present
if let Ok(client_id) = std::env::var("CLIENT_ID") {
    config.replication.client_id = client_id;
}

// Override client_password from environment variable if present
if let Ok(client_password) = std::env::var("CLIENT_PASSWORD") {
    config.replication.client_password = Some(client_password);
}

// Override mqtt_broker from environment variable if present
if let Ok(mqtt_broker) = std::env::var("MQTT_BROKER") {
    config.replication.mqtt_broker = mqtt_broker;
}

// Override mqtt_port from environment variable if present
if let Ok(mqtt_port) = std::env::var("MQTT_PORT") {
    if let Ok(port) = mqtt_port.parse::<u16>() {
        config.replication.mqtt_port = port;
    }
}

// Override topic_prefix from environment variable if present
if let Ok(topic_prefix) = std::env::var("TOPIC_PREFIX") {
    config.replication.topic_prefix = topic_prefix;
}
```

## Testing

The implementation includes comprehensive tests:

1. `test_config_load()`: Tests basic configuration loading
2. `test_environment_variable_overrides()`: Tests all environment variable override functionality
3. `test_mqtt_port_invalid_environment_variable()`: Tests that invalid MQTT_PORT values don't break the system

Run tests with:
```bash
cargo test config
```

## Security Considerations

- **Environment variables take priority**: This allows secure credential injection in production
- **Optional password**: Systems without authentication can omit the password
- **No logging of credentials**: Passwords are not logged in debug output
- **Memory safety**: Uses Rust's memory safety features for credential handling

## Backward Compatibility

- **Fully backward compatible**: Existing config files continue to work
- **Optional password field**: No breaking changes to existing configurations
- **Default behavior preserved**: Systems without authentication work as before

## Production Deployment

For production deployments, it's recommended to:

1. Use environment variables for credentials
2. Set appropriate file permissions on config files
3. Use secure MQTT brokers with TLS
4. Rotate credentials regularly

Example production deployment:
```bash
export CLIENT_ID=prod-node-001
export CLIENT_PASSWORD=$(cat /etc/secrets/mqtt-password)
export MQTT_BROKER=secure.mqtt.company.com
export MQTT_PORT=8883
export TOPIC_PREFIX=production_cluster
cargo run --bin merkle_kv --release production-config.toml
```
