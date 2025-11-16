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

    // Tests for get_socket_path_from_url()

    #[test]
    fn test_get_socket_path_from_url_explicit() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp/sockets"));

        // Explicit Unix socket URL should extract the socket path directly
        let url = "http::unix///tmp/explicit.sock";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, Some(PathBuf::from("/tmp/explicit.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_with_path() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp/sockets"));

        // Unix socket URL with URL path
        let url = "http::unix///tmp/app.sock/api/data";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, Some(PathBuf::from("/tmp/app.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_hostname_mapped() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp/sockets"));
        mapping.add_mapping("localhost".to_string(), PathBuf::from("/var/run/local.sock"));

        // Standard URL should use hostname mapping
        let url = "http://localhost/path";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, Some(PathBuf::from("/var/run/local.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_hostname_default() {
        let mapping = SocketMapping::new(PathBuf::from("/custom/dir"));

        // Unmapped hostname should use default directory
        let url = "http://example.com/path";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, Some(PathBuf::from("/custom/dir/example.com.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_malformed() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp"));

        // Malformed URL should return None
        let url = "not-a-valid-url";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, None);
    }

    #[test]
    fn test_get_socket_path_from_url_priority() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp/default"));
        mapping.add_mapping("localhost".to_string(), PathBuf::from("/tmp/mapped.sock"));

        // Explicit path should have highest priority, even with hostname mapping
        let explicit_url = "http::unix///tmp/explicit.sock";
        let explicit_path = mapping.get_socket_path_from_url(explicit_url);
        assert_eq!(explicit_path, Some(PathBuf::from("/tmp/explicit.sock")));

        // Hostname mapping should be used when no explicit path
        let hostname_url = "http://localhost/path";
        let hostname_path = mapping.get_socket_path_from_url(hostname_url);
        assert_eq!(hostname_path, Some(PathBuf::from("/tmp/mapped.sock")));

        // Default should be used when neither explicit nor mapped
        let default_url = "http://unmapped.com/path";
        let default_path = mapping.get_socket_path_from_url(default_url);
        assert_eq!(
            default_path,
            Some(PathBuf::from("/tmp/default/unmapped.com.sock"))
        );
    }

    #[test]
    fn test_get_socket_path_from_url_relative_path() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp"));

        // Relative socket path
        let url = "http::unix//var/run/app.sock";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, Some(PathBuf::from("var/run/app.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_with_query() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp"));

        // URL with query parameters should still extract socket path
        let url = "http::unix///tmp/test.sock/path?param=value";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, Some(PathBuf::from("/tmp/test.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_tcp_transport() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp"));

        // Explicit TCP transport should use hostname mapping, not socket path
        let url = "http::tcp//localhost:8080/path";
        let path = mapping.get_socket_path_from_url(url);
        // Should use hostname mapping for localhost
        assert_eq!(path, Some(PathBuf::from("/tmp/localhost.sock")));
    }

    #[test]
    fn test_get_socket_path_from_url_empty_url() {
        let mapping = SocketMapping::new(PathBuf::from("/tmp"));

        // Empty URL should return None
        let url = "";
        let path = mapping.get_socket_path_from_url(url);
        assert_eq!(path, None);
    }
}
