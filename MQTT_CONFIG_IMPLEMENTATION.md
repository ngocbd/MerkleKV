# MQTT Configuration with Environment Variable Support

## Overview

The MerkleKV project now supports MQTT client ID and password configuration through both configuration files and environment variables. Environment variables take top priority when present.

## Implementation Details

### Configuration Structure

The `ReplicationConfig` struct in `src/config.rs` now includes:

```rust
pub struct ReplicationConfig {
    // ... existing fields ...
    
    /// Unique identifier for this node in MQTT communications
    /// Can be overridden by CLIENT_ID environment variable
    pub client_id: String,

    /// Optional password for MQTT broker authentication
    /// Can be overridden by CLIENT_PASSWORD environment variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_password: Option<String>,
}
```

### Environment Variable Priority

The `Config::load()` method now checks for environment variables and applies them with top priority:

1. **CLIENT_ID**: Overrides `replication.client_id` from config file
2. **CLIENT_PASSWORD**: Overrides `replication.client_password` from config file

### Configuration File Examples

#### Basic config.toml
```toml
[replication]
enabled = false
mqtt_broker = "localhost"
mqtt_port = 1883
topic_prefix = "merkle_kv"
client_id = "node1"
# client_password = "your_mqtt_password"  # Optional MQTT password
```

#### Config with password
```toml
[replication]
enabled = true
mqtt_broker = "test.mosquitto.org"
mqtt_port = 1883
topic_prefix = "merkle_kv_demo"
client_id = "demo_node"
client_password = "demo_password"
```

## Usage Examples

### 1. Default Configuration (no environment variables)
```bash
./target/debug/merkle_kv
```
Output:
```
MQTT Client ID: node1
MQTT Client Password: [NOT SET]
```

### 2. CLIENT_ID Environment Variable Override
```bash
CLIENT_ID=my_custom_node ./target/debug/merkle_kv
```
Output:
```
MQTT Client ID: my_custom_node
MQTT Client Password: [NOT SET]
```

### 3. CLIENT_PASSWORD Environment Variable Override
```bash
CLIENT_PASSWORD=secret123 ./target/debug/merkle_kv
```
Output:
```
MQTT Client ID: node1
MQTT Client Password: [SET] (length: 9)
```

### 4. Both Environment Variables
```bash
CLIENT_ID=secure_node CLIENT_PASSWORD=secure_pass ./target/debug/merkle_kv
```
Output:
```
MQTT Client ID: secure_node
MQTT Client Password: [SET] (length: 11)
```

### 5. With Configuration File
```bash
CLIENT_ID=override_node ./target/debug/merkle_kv --config examples/config_demo.toml
```
Output:
```
MQTT Client ID: override_node
MQTT Client Password: [SET] (length: 13)
```

## Security Considerations

1. **Password Protection**: Passwords are never displayed in plain text in logs or output
2. **Environment Variables**: Environment variables take precedence for security-sensitive deployments
3. **Optional Password**: The password field is optional and marked with `skip_serializing_if` to avoid serializing `None` values

## MQTT Client Integration

The replication module (`src/replication.rs`) has been updated to use the password when connecting to MQTT:

```rust
// Set password if configured
if let Some(ref password) = config.replication.client_password {
    // Note: rumqttc requires both username and password, but some MQTT brokers
    // allow using just a password with an empty username
    mqtt_options.set_credentials("", password);
}
```

## Testing

The implementation includes comprehensive tests:

1. **Unit Tests**: Configuration loading and environment variable overrides
2. **Integration Tests**: Real-world scenarios with different configuration combinations
3. **Examples**: Demo configuration files showing various use cases

### Running Tests

```bash
# Run configuration tests
cargo test config

# Test with environment variables
CLIENT_ID=test_node CLIENT_PASSWORD=test_pass cargo run
```

## Files Modified

1. **src/config.rs**: Added `client_password` field and environment variable override logic
2. **src/replication.rs**: Updated MQTT client to use password authentication
3. **config.toml**: Added commented example for password configuration
4. **examples/config_demo.toml**: Example configuration with password
5. **src/main.rs**: Added debug output to show configuration values

## Backward Compatibility

The implementation is fully backward compatible:
- Existing configurations without passwords continue to work
- The `client_password` field is optional
- Environment variables are only applied when present
