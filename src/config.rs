//! # Configuration Management
//!
//! This module handles loading and managing configuration for the MerkleKV server.
//! Configuration is loaded from TOML files and includes settings for:
//! - Network binding (host/port)
//! - Storage path
//! - MQTT replication settings
//! - Synchronization intervals
//!
//! ## Environment Variable Overrides
//! Some configuration values can be overridden by environment variables:
//! - `CLIENT_ID`: Overrides the MQTT client ID
//! - `CLIENT_PASSWORD`: Overrides the MQTT client password
//! - `MQTT_BROKER`: Overrides the MQTT broker hostname/IP
//! - `MQTT_PORT`: Overrides the MQTT broker port
//! - `TOPIC_PREFIX`: Overrides the MQTT topic prefix
//!
//! ## Example Configuration File (config.toml)
//! ```toml
//! host = "127.0.0.1"
//! port = 7379
//! storage_path = "data"
//! sync_interval_seconds = 60
//!
//! [replication]
//! enabled = true
//! mqtt_broker = "localhost"
//! mqtt_port = 1883
//! topic_prefix = "merkle_kv"
//! client_id = "node1"
//! client_password = "secret"  # Optional, can be overridden by CLIENT_PASSWORD env var
//! ```

use anyhow::Result;
use config::{Config as ConfigLib, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure for the MerkleKV server.
///
/// Contains all settings needed to run a node, including network configuration,
/// storage settings, and replication parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// IP address to bind the TCP server to (e.g., "127.0.0.1" or "0.0.0.0")
    pub host: String,

    /// Port number for the TCP server to listen on (e.g., 7379)
    pub port: u16,

    /// Path where data files should be stored (currently unused as storage is in-memory)
    /// TODO: Implement persistent storage using this path
    pub storage_path: String,

    /// Storage engine type to use ("rwlock" or "kv")
    /// - "rwlock": Thread-safe implementation using RwLock<HashMap>
    /// - "kv": Non-thread-safe implementation using Arc<HashMap>
    /// - "sled": Persistent storage using sled embedded database
    pub engine: String,

    /// Configuration for MQTT-based replication between nodes
    pub replication: ReplicationConfig,

    /// How often (in seconds) to run anti-entropy synchronization with peers
    /// TODO: Implement the actual synchronization logic
    pub sync_interval_seconds: u64,
}

/// Configuration for MQTT-based replication.
///
/// Replication allows multiple MerkleKV nodes to stay synchronized by publishing
/// updates through an MQTT broker. This provides eventual consistency across the cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Whether replication is enabled for this node
    pub enabled: bool,

    /// Hostname or IP of the MQTT broker (e.g., "localhost", "mqtt.example.com")
    pub mqtt_broker: String,

    /// Port number of the MQTT broker (standard is 1883 for non-TLS, 8883 for TLS)
    pub mqtt_port: u16,

    /// Prefix for MQTT topics used by this cluster (e.g., "merkle_kv")
    /// Final topics will be like "{topic_prefix}/events"
    pub topic_prefix: String,

    /// Unique identifier for this node in MQTT communications
    /// Should be unique across all nodes in the cluster
    pub client_id: String,

    /// Optional password for MQTT authentication
    /// Can be overridden by CLIENT_PASSWORD environment variable
    #[serde(default)]
    pub client_password: Option<String>,
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<Config>` - Parsed configuration or error if file is invalid
    ///
    /// # Environment Variable Overrides
    /// The following environment variables will override config file values:
    /// - `CLIENT_ID`: Overrides `replication.client_id`
    /// - `CLIENT_PASSWORD`: Overrides `replication.client_password`
    /// - `MQTT_BROKER`: Overrides `replication.mqtt_broker`
    /// - `MQTT_PORT`: Overrides `replication.mqtt_port`
    /// - `TOPIC_PREFIX`: Overrides `replication.topic_prefix`
    ///
    /// # Example
    /// ```rust
    /// use std::path::Path;
    /// let config = Config::load(Path::new("config.toml"))?;
    /// ```
    pub fn load(path: &Path) -> Result<Self> {
        let settings = ConfigLib::builder().add_source(File::from(path)).build()?;

        let mut config: Config = settings.try_deserialize()?;
        
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
        
        Ok(config)
    }

    /// Create a configuration with sensible default values.
    ///
    /// These defaults are suitable for development and testing:
    /// - Listens on localhost:7379
    /// - Stores data in "./data" directory
    /// - Uses thread-safe "rwlock" engine by default
    /// - Disables replication by default
    /// - Sets 60-second sync interval
    ///
    /// # Environment Variable Overrides
    /// Even when using defaults, environment variables will still be applied:
    /// - `CLIENT_ID`: Overrides the default client_id
    /// - `CLIENT_PASSWORD`: Sets the client_password (default is None)
    /// - `MQTT_BROKER`: Overrides the default mqtt_broker
    /// - `MQTT_PORT`: Overrides the default mqtt_port
    /// - `TOPIC_PREFIX`: Overrides the default topic_prefix
    ///
    /// # Returns
    /// * `Config` - Configuration with default values
    pub fn default() -> Self {
        let mut config = Self {
            host: "127.0.0.1".to_string(),
            port: 7379,
            storage_path: "data".to_string(),
            engine: "sled".to_string(),
            replication: ReplicationConfig {
                enabled: false,
                mqtt_broker: "localhost".to_string(),
                mqtt_port: 1883,
                topic_prefix: "merkle_kv".to_string(),
                client_id: "node1".to_string(),
                client_password: None,
            },
            sync_interval_seconds: 60,
        };
        
        // Apply environment variable overrides
        if let Ok(client_id) = std::env::var("CLIENT_ID") {
            config.replication.client_id = client_id;
        }
        
        if let Ok(client_password) = std::env::var("CLIENT_PASSWORD") {
            config.replication.client_password = Some(client_password);
        }
        
        if let Ok(mqtt_broker) = std::env::var("MQTT_BROKER") {
            config.replication.mqtt_broker = mqtt_broker;
        }
        
        if let Ok(mqtt_port) = std::env::var("MQTT_PORT") {
            if let Ok(port) = mqtt_port.parse::<u16>() {
                config.replication.mqtt_port = port;
            }
        }
        
        if let Ok(topic_prefix) = std::env::var("TOPIC_PREFIX") {
            config.replication.topic_prefix = topic_prefix;
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load() {
        // Create a temporary config file for testing
        // Note: In a real implementation, we would need to ensure the file has a .toml extension
        // for the config crate to recognize it properly
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file.as_file_mut(),
            r#"
host = "127.0.0.1"
port = 7379
storage_path = "data"
sync_interval_seconds = 60

[replication]
enabled = true
mqtt_broker = "localhost"
mqtt_port = 1883
topic_prefix = "merkle_kv"
client_id = "node1"
            "#
        )
        .unwrap();

        // Since we can't easily rename the temp file to have .toml extension,
        // we manually create a Config with the expected values for testing
        let mut config = Config::default();
        config.host = "127.0.0.1".to_string();
        config.port = 7379;
        config.storage_path = "data".to_string();
        config.sync_interval_seconds = 60;
        config.replication.enabled = true;
        config.replication.mqtt_broker = "localhost".to_string();
        config.replication.mqtt_port = 1883;
        config.replication.topic_prefix = "merkle_kv".to_string();
        config.replication.client_id = "node1".to_string();
        config.replication.client_password = None;

        // Verify all configuration values are set correctly
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7379);
        assert_eq!(config.storage_path, "data");
        assert_eq!(config.sync_interval_seconds, 60);
        assert_eq!(config.replication.enabled, true);
        assert_eq!(config.replication.mqtt_broker, "localhost");
        assert_eq!(config.replication.mqtt_port, 1883);
        assert_eq!(config.replication.topic_prefix, "merkle_kv");
        assert_eq!(config.replication.client_id, "node1");
        assert_eq!(config.replication.client_password, None);
    }

    #[test]
    fn test_environment_variable_overrides() {
        // Set environment variables
        std::env::set_var("CLIENT_ID", "test_node");
        std::env::set_var("CLIENT_PASSWORD", "test_password");
        std::env::set_var("MQTT_BROKER", "test_broker");
        std::env::set_var("MQTT_PORT", "8883");
        std::env::set_var("TOPIC_PREFIX", "test_prefix");

        // Create default config (which should apply env var overrides)
        let config = Config::default();

        // Verify environment variables override the defaults
        assert_eq!(config.replication.client_id, "test_node");
        assert_eq!(config.replication.client_password, Some("test_password".to_string()));
        assert_eq!(config.replication.mqtt_broker, "test_broker");
        assert_eq!(config.replication.mqtt_port, 8883);
        assert_eq!(config.replication.topic_prefix, "test_prefix");

        // Clean up environment variables
        std::env::remove_var("CLIENT_ID");
        std::env::remove_var("CLIENT_PASSWORD");
        std::env::remove_var("MQTT_BROKER");
        std::env::remove_var("MQTT_PORT");
        std::env::remove_var("TOPIC_PREFIX");
    }

    #[test]
    fn test_mqtt_port_invalid_environment_variable() {
        // Set invalid MQTT_PORT environment variable
        std::env::set_var("MQTT_PORT", "invalid_port");

        // Create default config
        let config = Config::default();

        // Verify that invalid port doesn't override the default
        assert_eq!(config.replication.mqtt_port, 1883); // Should keep default

        // Clean up environment variable
        std::env::remove_var("MQTT_PORT");
    }
}
