/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Configuration for Unix domain socket networking

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for Unix domain socket connections
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnixSocketConfig {
    /// Whether Unix domain socket mode is enabled
    pub enabled: bool,
    /// Default directory for socket files
    pub socket_dir: PathBuf,
    /// Custom hostname-to-socket mappings
    pub mappings: Vec<HostSocketMapping>,
}

/// Mapping from a hostname to a Unix socket path
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HostSocketMapping {
    /// The hostname to match (e.g., "localhost", "example.com")
    pub host: String,
    /// The path to the Unix socket
    pub socket_path: PathBuf,
}

impl Default for UnixSocketConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            socket_dir: PathBuf::from("/tmp/servo-sockets"),
            mappings: vec![
                HostSocketMapping {
                    host: "localhost".to_string(),
                    socket_path: PathBuf::from("/tmp/servo-sockets/localhost.sock"),
                },
            ],
        }
    }
}

impl UnixSocketConfig {
    /// Create a new configuration from environment variables
    ///
    /// Supports:
    /// - `SERVO_USE_UNIX_SOCKETS=true|false|1|0` - Enable/disable UDS mode
    /// - `SERVO_SOCKET_DIR=/path/to/sockets` - Default socket directory
    /// - `SERVO_SOCKET_MAPPINGS=host1:/path1,host2:/path2` - Custom mappings
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("SERVO_USE_UNIX_SOCKETS") {
            config.enabled = val.to_lowercase() == "true" || val == "1";
        }

        if let Ok(dir) = std::env::var("SERVO_SOCKET_DIR") {
            config.socket_dir = PathBuf::from(dir);
        }

        // Parse mappings from SERVO_SOCKET_MAPPINGS="host1:/path/to/socket1,host2:/path/to/socket2"
        if let Ok(mappings) = std::env::var("SERVO_SOCKET_MAPPINGS") {
            config.mappings = mappings
                .split(',')
                .filter_map(|mapping| {
                    let parts: Vec<&str> = mapping.split(':').collect();
                    if parts.len() >= 2 {
                        // Join all parts after the first colon back together
                        // to support absolute paths like /path/to/socket
                        let host = parts[0].trim().to_string();
                        let socket_path = PathBuf::from(parts[1..].join(":").trim());
                        Some(HostSocketMapping { host, socket_path })
                    } else {
                        None
                    }
                })
                .collect();
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UnixSocketConfig::default();
        assert!(config.enabled);
        assert_eq!(config.socket_dir, PathBuf::from("/tmp/servo-sockets"));
        assert_eq!(config.mappings.len(), 1);
        assert_eq!(config.mappings[0].host, "localhost");
        assert_eq!(
            config.mappings[0].socket_path,
            PathBuf::from("/tmp/servo-sockets/localhost.sock")
        );
    }
}
