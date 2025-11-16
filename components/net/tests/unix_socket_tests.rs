/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Unit tests for Unix socket configuration and connector

#[cfg(test)]
mod unix_config_tests {
    use net::unix_config::*;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_default_config() {
        let config = UnixSocketConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.socket_dir, PathBuf::from("/tmp/servo-sockets"));
        assert!(config.mappings.is_empty());
    }

    #[test]
    fn test_env_var_enabled() {
        unsafe {
            env::set_var("SERVO_USE_UNIX_SOCKETS", "true");
        }
        let config = UnixSocketConfig::from_env();
        assert!(config.enabled);
        unsafe {
            env::remove_var("SERVO_USE_UNIX_SOCKETS");
        }
    }

    #[test]
    fn test_env_var_enabled_numeric() {
        unsafe {
            env::set_var("SERVO_USE_UNIX_SOCKETS", "1");
        }
        let config = UnixSocketConfig::from_env();
        assert!(config.enabled);
        unsafe {
            env::remove_var("SERVO_USE_UNIX_SOCKETS");
        }
    }

    #[test]
    fn test_env_var_disabled() {
        unsafe {
            env::set_var("SERVO_USE_UNIX_SOCKETS", "false");
        }
        let config = UnixSocketConfig::from_env();
        assert!(!config.enabled);
        unsafe {
            env::remove_var("SERVO_USE_UNIX_SOCKETS");
        }
    }

    #[test]
    fn test_socket_dir_from_env() {
        unsafe {
            env::set_var("SERVO_SOCKET_DIR", "/custom/socket/dir");
        }
        let config = UnixSocketConfig::from_env();
        assert_eq!(config.socket_dir, PathBuf::from("/custom/socket/dir"));
        unsafe {
            env::remove_var("SERVO_SOCKET_DIR");
        }
    }

    #[test]
    fn test_socket_mappings_single() {
        unsafe {
            env::set_var("SERVO_SOCKET_MAPPINGS", "localhost:/tmp/test.sock");
        }
        let config = UnixSocketConfig::from_env();
        assert_eq!(config.mappings.len(), 1);
        assert_eq!(config.mappings[0].host, "localhost");
        assert_eq!(config.mappings[0].socket_path, PathBuf::from("/tmp/test.sock"));
        unsafe {
            env::remove_var("SERVO_SOCKET_MAPPINGS");
        }
    }

    #[test]
    fn test_socket_mappings_multiple() {
        unsafe {
            env::set_var(
                "SERVO_SOCKET_MAPPINGS",
                "localhost:/tmp/local.sock,example.com:/tmp/example.sock",
            );
        }
        let config = UnixSocketConfig::from_env();
        assert_eq!(config.mappings.len(), 2);
        assert_eq!(config.mappings[0].host, "localhost");
        assert_eq!(config.mappings[1].host, "example.com");
        unsafe {
            env::remove_var("SERVO_SOCKET_MAPPINGS");
        }
    }

    #[test]
    fn test_socket_mappings_with_colon_in_path() {
        // Test that paths with colons (like /path:with:colons.sock) work
        unsafe {
            env::set_var("SERVO_SOCKET_MAPPINGS", "localhost:/path:with:colons.sock");
        }
        let config = UnixSocketConfig::from_env();
        assert_eq!(config.mappings.len(), 1);
        assert_eq!(
            config.mappings[0].socket_path,
            PathBuf::from("/path:with:colons.sock")
        );
        unsafe {
            env::remove_var("SERVO_SOCKET_MAPPINGS");
        }
    }

    #[test]
    fn test_socket_mappings_invalid() {
        unsafe {
            env::set_var("SERVO_SOCKET_MAPPINGS", "invalid");
        }
        let config = UnixSocketConfig::from_env();
        assert_eq!(config.mappings.len(), 0);
        unsafe {
            env::remove_var("SERVO_SOCKET_MAPPINGS");
        }
    }
}

#[cfg(test)]
mod unix_connector_tests {
    use net::unix_connector::*;
    use std::path::PathBuf;

    #[test]
    fn test_socket_mapping_new() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp/sockets"));
        assert_eq!(
            mapping.get_socket_path("unmapped"),
            PathBuf::from("/tmp/sockets/unmapped.sock")
        );
    }

    #[test]
    fn test_socket_mapping_add_and_get() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp/sockets"));
        mapping.add_mapping("localhost".to_string(), PathBuf::from("/custom/path.sock"));

        assert_eq!(
            mapping.get_socket_path("localhost"),
            PathBuf::from("/custom/path.sock")
        );
    }

    #[test]
    fn test_socket_mapping_default_fallback() {
        let mapping = SocketMapping::new(PathBuf::from("/var/run/sockets"));

        // Should use default mapping for unmapped hosts
        assert_eq!(
            mapping.get_socket_path("example.com"),
            PathBuf::from("/var/run/sockets/example.com.sock")
        );
    }

    #[test]
    fn test_socket_mapping_multiple_hosts() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp"));
        mapping.add_mapping("host1".to_string(), PathBuf::from("/tmp/host1.sock"));
        mapping.add_mapping("host2".to_string(), PathBuf::from("/tmp/host2.sock"));

        assert_eq!(
            mapping.get_socket_path("host1"),
            PathBuf::from("/tmp/host1.sock")
        );
        assert_eq!(
            mapping.get_socket_path("host2"),
            PathBuf::from("/tmp/host2.sock")
        );
    }
}
