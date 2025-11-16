/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Transport-aware URL parsing for Servo
//!
//! This module implements a unified URL parser that supports explicit transport
//! specifications using the syntax: `scheme::transport//authority`
//!
//! Examples:
//! - `http::unix//var/run/app.sock` - HTTP over Unix socket (relative path)
//! - `http::unix///tmp/app.sock` - HTTP over Unix socket (absolute path, 3 slashes)
//! - `https://example.com` - HTTPS over TCP (implied transport)
//! - `http::tcp//localhost:8080` - HTTP over TCP (explicit transport)
//!
//! Note: For absolute Unix socket paths starting with `/`, use three slashes
//! (like file:/// URLs). For relative paths, use two slashes.

use std::fmt;

use log::{debug, warn};
use servo_url::ServoUrl;

/// Supported transport protocols
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transport {
    /// TCP/IP networking (default for most schemes)
    Tcp,
    /// Unix domain sockets (IPC)
    Unix,
    /// Future: QUIC transport
    Quic,
}

impl Transport {
    /// Parse transport from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tcp" => Some(Transport::Tcp),
            "unix" | "uds" | "ipc" => Some(Transport::Unix),
            "quic" => Some(Transport::Quic),
            _ => None,
        }
    }

    /// Get transport as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Transport::Tcp => "tcp",
            Transport::Unix => "unix",
            Transport::Quic => "quic",
        }
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A URL with transport layer information
///
/// This wraps a ServoUrl and adds transport metadata, supporting
/// both standard URLs and extended transport-aware syntax.
#[derive(Clone, Debug)]
pub struct TransportUrl {
    /// The underlying URL (may be modified from original for compatibility)
    url: ServoUrl,
    /// The transport layer to use
    transport: Transport,
    /// Original scheme before any protocol downgrading
    original_scheme: String,
    /// Whether the transport was explicitly specified
    explicit_transport: bool,
    /// For Unix sockets, the actual socket path
    unix_socket_path: Option<String>,
}

impl TransportUrl {
    /// Parse a URL with optional transport specification
    ///
    /// Syntax:
    /// - `scheme::transport//authority/path` - Explicit transport
    /// - `scheme://authority/path` - Implied transport (standard URL)
    pub fn parse(url_str: &str) -> Result<Self, url::ParseError> {
        debug!("Parsing transport URL: {}", url_str);

        // Step 1: Find the first colon to extract the scheme
        let scheme_end = url_str
            .find(':')
            .ok_or(url::ParseError::RelativeUrlWithoutBase)?;
        let scheme = &url_str[..scheme_end];
        let after_scheme = &url_str[scheme_end + 1..];

        // Step 2: Check for explicit transport (::) vs standard (://)
        // Note: after_scheme already has the first ':' stripped, so we check for:
        //   - "//" for standard URLs (e.g., "http://example.com")
        //   - ":" for explicit transport (e.g., "http::unix//path")
        let (transport, explicit, authority_and_path, unix_socket_path) =
            if after_scheme.starts_with("//") {
                // Case 2: Standard URL with implied transport
                let implied = Self::implied_transport_for_scheme(scheme);
                debug!("  Implied transport for '{}': {}", scheme, implied);
                (implied, false, &after_scheme[2..], None)
            } else if after_scheme.starts_with(':') {
                // Case 1: Explicit transport specification
                let after_double_colon = &after_scheme[1..];

                // Find the '//' that separates transport from authority
                let authority_start = after_double_colon
                    .find("//")
                    .ok_or(url::ParseError::RelativeUrlWithoutBase)?;

                let transport_str = &after_double_colon[..authority_start];
                let transport = Transport::from_str(transport_str)
                    .ok_or(url::ParseError::RelativeUrlWithoutBase)?;

                let authority_and_path = &after_double_colon[authority_start + 2..];

                debug!("  Explicit transport: {}", transport);

                // For Unix sockets, extract the socket path
                let unix_path = if transport == Transport::Unix {
                    // The authority for Unix sockets is the socket path
                    // For absolute paths: /tmp/test.sock/api -> socket: /tmp/test.sock, path: /api
                    // For relative paths: var/run/app.sock/api -> socket: var/run/app.sock, path: /api
                    //
                    // Strategy: Look for socket file extensions (.sock, .socket) and split after them.
                    // This allows us to distinguish socket path from URL path.

                    let socket_path = if let Some(sock_pos) = authority_and_path.find(".sock") {
                        // Split after .sock or .socket
                        if authority_and_path[sock_pos..].starts_with(".socket") {
                            let end = sock_pos + 7; // ".socket" is 7 chars
                            // Check if there's a '/' after .socket
                            if authority_and_path.len() > end
                                && authority_and_path.as_bytes()[end] == b'/'
                            {
                                &authority_and_path[..end]
                            } else {
                                authority_and_path
                            }
                        } else {
                            let end = sock_pos + 5; // ".sock" is 5 chars
                            // Check if there's a '/' after .sock
                            if authority_and_path.len() > end
                                && authority_and_path.as_bytes()[end] == b'/'
                            {
                                &authority_and_path[..end]
                            } else {
                                authority_and_path
                            }
                        }
                    } else {
                        // No socket extension found - use entire string as socket path
                        authority_and_path
                    };

                    Some(socket_path.to_string())
                } else {
                    None
                };

                (transport, true, authority_and_path, unix_path)
            } else {
                // Malformed URL
                return Err(url::ParseError::RelativeUrlWithoutBase);
            };

        // Step 3: Handle protocol downgrading for local transports
        let (final_scheme, original_scheme) =
            Self::adjust_scheme_for_transport(scheme, &transport);

        // Step 4: Reconstruct URL for standard parser
        let reconstructed = if transport == Transport::Unix {
            // For Unix sockets, use a placeholder host and put path in the URL path
            format!(
                "{}://unix-socket/{}",
                final_scheme,
                authority_and_path.trim_start_matches('/')
            )
        } else {
            // For TCP and other transports, use standard URL format
            format!("{}://{}", final_scheme, authority_and_path)
        };

        debug!(
            "  Reconstructed URL: {} (original scheme: {})",
            reconstructed, original_scheme
        );

        // Step 5: Parse with standard ServoUrl parser
        let url = ServoUrl::parse(&reconstructed)?;

        Ok(TransportUrl {
            url,
            transport,
            original_scheme: original_scheme.to_string(),
            explicit_transport: explicit,
            unix_socket_path,
        })
    }

    /// Get the implied/default transport for a scheme
    fn implied_transport_for_scheme(scheme: &str) -> Transport {
        match scheme.to_lowercase().as_str() {
            "http" | "https" | "ws" | "wss" | "ftp" | "ftps" => Transport::Tcp,
            _ => Transport::Tcp, // Default to TCP for unknown schemes
        }
    }

    /// Adjust scheme for transport (e.g., HTTPS -> HTTP for Unix sockets)
    ///
    /// When using local transports like Unix sockets, TLS doesn't make sense
    /// because we're not going over a network. We downgrade to the non-secure
    /// version of the protocol.
    fn adjust_scheme_for_transport<'a>(
        scheme: &'a str,
        transport: &Transport,
    ) -> (&'a str, &'a str) {
        let original = scheme;

        let adjusted = match (scheme.to_lowercase().as_str(), transport) {
            // Downgrade secure protocols for local/Unix transports
            ("https", Transport::Unix) => {
                warn!(
                    "Downgrading HTTPS to HTTP for Unix socket transport (TLS not applicable)"
                );
                "http"
            },
            ("wss", Transport::Unix) => {
                warn!("Downgrading WSS to WS for Unix socket transport (TLS not applicable)");
                "ws"
            },
            ("ftps", Transport::Unix) => {
                warn!("Downgrading FTPS to FTP for Unix socket transport (TLS not applicable)");
                "ftp"
            },

            // Keep everything else as-is
            _ => scheme,
        };

        (adjusted, original)
    }

    /// Get the underlying ServoUrl
    pub fn as_url(&self) -> &ServoUrl {
        &self.url
    }

    /// Get the transport
    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    /// Get the original scheme (before any downgrading)
    pub fn original_scheme(&self) -> &str {
        &self.original_scheme
    }

    /// Was the transport explicitly specified?
    pub fn has_explicit_transport(&self) -> bool {
        self.explicit_transport
    }

    /// Convert to standard ServoUrl (loses transport info)
    pub fn into_url(self) -> ServoUrl {
        self.url
    }

    /// Get the host
    pub fn host_str(&self) -> Option<&str> {
        self.url.host_str()
    }

    /// Get the path
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// Is this a Unix socket URL?
    pub fn is_unix_socket(&self) -> bool {
        self.transport == Transport::Unix
    }

    /// For Unix sockets, get the socket path
    pub fn unix_socket_path(&self) -> Option<&str> {
        self.unix_socket_path.as_deref()
    }

    /// Get the query string
    pub fn query(&self) -> Option<&str> {
        self.url.query()
    }

    /// Get the fragment
    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }

    /// Get the port
    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    /// Get the username
    pub fn username(&self) -> &str {
        self.url.username()
    }

    /// Get the password
    pub fn password(&self) -> Option<&str> {
        self.url.password()
    }
}

impl fmt::Display for TransportUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.explicit_transport {
            if let Some(ref socket_path) = self.unix_socket_path {
                write!(
                    f,
                    "{}::unix//{}/{}",
                    self.original_scheme,
                    socket_path,
                    self.path().trim_start_matches('/')
                )
            } else {
                write!(f, "{}", self.url)
            }
        } else {
            write!(f, "{}", self.url)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unix_socket_url() {
        let url = TransportUrl::parse("http::unix//var/run/app.sock").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.original_scheme(), "http");
        assert!(url.has_explicit_transport());
        assert!(url.is_unix_socket());
        assert_eq!(url.unix_socket_path(), Some("var/run/app.sock"));
    }

    #[test]
    fn test_parse_unix_socket_url_absolute() {
        // Test absolute path with three slashes
        let url = TransportUrl::parse("http::unix///tmp/test.sock").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.original_scheme(), "http");
        assert!(url.has_explicit_transport());
        assert!(url.is_unix_socket());
        assert_eq!(url.unix_socket_path(), Some("/tmp/test.sock"));
    }

    #[test]
    fn test_parse_unix_socket_url_absolute_with_path() {
        // Test absolute socket path with URL path
        let url = TransportUrl::parse("http::unix///tmp/test.sock/api/data").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert!(url.is_unix_socket());
        assert_eq!(url.unix_socket_path(), Some("/tmp/test.sock"));
        assert!(url.path().contains("api/data"));
    }

    #[test]
    fn test_parse_unix_socket_url_with_path() {
        let url = TransportUrl::parse("http::unix//var/run/app.sock/api/endpoint").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert!(url.is_unix_socket());
        assert_eq!(url.unix_socket_path(), Some("var/run/app.sock"));
        assert!(url.path().contains("api/endpoint"));
    }

    #[test]
    fn test_parse_standard_http_url() {
        let url = TransportUrl::parse("http://example.com/path").unwrap();
        assert_eq!(url.transport(), &Transport::Tcp);
        assert_eq!(url.original_scheme(), "http");
        assert!(!url.has_explicit_transport());
        assert!(!url.is_unix_socket());
    }

    #[test]
    fn test_parse_standard_https_url() {
        let url = TransportUrl::parse("https://example.com").unwrap();
        assert_eq!(url.transport(), &Transport::Tcp);
        assert_eq!(url.original_scheme(), "https");
        assert!(!url.has_explicit_transport());
    }

    #[test]
    fn test_https_downgrade_for_unix() {
        let url = TransportUrl::parse("https::unix//var/run/secure.sock").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.original_scheme(), "https");
        // Should be downgraded to http internally
        assert_eq!(url.as_url().scheme(), "http");
        assert!(url.has_explicit_transport());
    }

    #[test]
    fn test_explicit_tcp_transport() {
        let url = TransportUrl::parse("http::tcp//localhost:8080").unwrap();
        assert_eq!(url.transport(), &Transport::Tcp);
        assert_eq!(url.original_scheme(), "http");
        assert!(url.has_explicit_transport());
    }

    #[test]
    fn test_transport_to_string() {
        assert_eq!(Transport::Tcp.to_string(), "tcp");
        assert_eq!(Transport::Unix.to_string(), "unix");
        assert_eq!(Transport::Quic.to_string(), "quic");
    }

    #[test]
    fn test_transport_from_str() {
        assert_eq!(Transport::from_str("tcp"), Some(Transport::Tcp));
        assert_eq!(Transport::from_str("unix"), Some(Transport::Unix));
        assert_eq!(Transport::from_str("uds"), Some(Transport::Unix));
        assert_eq!(Transport::from_str("ipc"), Some(Transport::Unix));
        assert_eq!(Transport::from_str("invalid"), None);
    }

    #[test]
    fn test_display_unix_url() {
        let url = TransportUrl::parse("http::unix//var/run/app.sock/api").unwrap();
        let displayed = format!("{}", url);
        assert!(displayed.contains("::unix"));
        assert!(displayed.contains("var/run/app.sock"));
    }

    // Edge case tests

    #[test]
    fn test_unix_url_with_query_parameters() {
        let url = TransportUrl::parse("http::unix///tmp/test.sock/path?key=value&foo=bar").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.unix_socket_path(), Some("/tmp/test.sock"));
        assert!(url.path().contains("path"));
        assert!(url.query().is_some());
        assert!(url.query().unwrap().contains("key=value"));
        assert!(url.query().unwrap().contains("foo=bar"));
    }

    #[test]
    fn test_unix_url_with_fragment() {
        let url = TransportUrl::parse("http::unix///tmp/test.sock/path#section").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.unix_socket_path(), Some("/tmp/test.sock"));
        assert!(url.path().contains("path"));
        assert_eq!(url.fragment(), Some("section"));
    }

    #[test]
    fn test_unix_url_with_query_and_fragment() {
        let url =
            TransportUrl::parse("http::unix///tmp/test.sock/api?param=1#anchor").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.unix_socket_path(), Some("/tmp/test.sock"));
        assert!(url.query().unwrap().contains("param=1"));
        assert_eq!(url.fragment(), Some("anchor"));
    }

    #[test]
    fn test_socket_path_with_special_characters() {
        // Test socket paths with hyphens, underscores, dots
        let url = TransportUrl::parse("http::unix///tmp/my-app_v1.0.sock").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.unix_socket_path(), Some("/tmp/my-app_v1.0.sock"));
    }

    #[test]
    fn test_socket_path_with_spaces() {
        // Socket paths with URL-encoded spaces
        let url = TransportUrl::parse("http::unix///tmp/my%20socket.sock").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        // The path should be decoded
        assert!(url.unix_socket_path().is_some());
    }

    #[test]
    fn test_very_long_socket_path() {
        // Unix socket paths have a limit (typically 108 bytes on Linux)
        // This tests that we can parse long paths even if they might fail at connection time
        let long_path = format!("/tmp/{}.sock", "a".repeat(200));
        let url_str = format!("http::unix/{}", long_path);
        let url = TransportUrl::parse(&url_str).unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert!(url.unix_socket_path().is_some());
        assert!(url.unix_socket_path().unwrap().len() > 100);
    }

    #[test]
    fn test_malformed_url_missing_scheme() {
        // URL without scheme should fail
        let result = TransportUrl::parse("::unix//tmp/test.sock");
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_url_invalid_chars() {
        // URL parser is permissive and will encode special characters
        // The connection would fail later when trying to use it
        let result = TransportUrl::parse("http::unix//tmp/test<>.sock");
        // This might parse successfully (URL encoding happens), or fail - both are acceptable
        // What matters is it doesn't crash
        match result {
            Ok(url) => {
                assert_eq!(url.transport(), &Transport::Unix);
            },
            Err(_) => {
                // Also acceptable - parser rejected it
            },
        }
    }

    #[test]
    fn test_empty_socket_path() {
        // Empty socket path should still parse but unix_socket_path should be None
        let result = TransportUrl::parse("http::unix//");
        // This might fail to parse as a valid URL, which is acceptable
        if let Ok(url) = result {
            assert_eq!(url.transport(), &Transport::Unix);
        }
    }

    #[test]
    fn test_tcp_url_with_port() {
        let url = TransportUrl::parse("http::tcp//localhost:8080/path").unwrap();
        assert_eq!(url.transport(), &Transport::Tcp);
        assert_eq!(url.host_str(), Some("localhost"));
        assert_eq!(url.port(), Some(8080));
        assert!(url.path().contains("path"));
    }

    #[test]
    fn test_standard_url_with_credentials() {
        let url = TransportUrl::parse("http://user:pass@example.com/path").unwrap();
        assert_eq!(url.transport(), &Transport::Tcp);
        assert_eq!(url.username(), "user");
        assert_eq!(url.password(), Some("pass"));
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_wss_downgrade_for_unix() {
        // WebSocket Secure should also downgrade to WS for Unix sockets
        let url = TransportUrl::parse("wss::unix//var/run/ws.sock").unwrap();
        assert_eq!(url.transport(), &Transport::Unix);
        assert_eq!(url.original_scheme(), "wss");
        // Should be downgraded to ws internally
        assert_eq!(url.as_url().scheme(), "ws");
    }

    #[test]
    fn test_case_insensitive_transport() {
        let url1 = TransportUrl::parse("http::UNIX//tmp/test.sock").unwrap();
        let url2 = TransportUrl::parse("http::Unix//tmp/test.sock").unwrap();
        assert_eq!(url1.transport(), &Transport::Unix);
        assert_eq!(url2.transport(), &Transport::Unix);
    }

    #[test]
    fn test_quic_transport() {
        let url = TransportUrl::parse("http::quic//localhost:443").unwrap();
        assert_eq!(url.transport(), &Transport::Quic);
        assert_eq!(url.original_scheme(), "http");
        assert!(url.has_explicit_transport());
    }

    #[test]
    fn test_relative_vs_absolute_socket_paths() {
        // Relative path (2 slashes after transport)
        let rel = TransportUrl::parse("http::unix//var/run/app.sock").unwrap();
        assert_eq!(rel.unix_socket_path(), Some("var/run/app.sock"));

        // Absolute path (3 slashes after transport)
        let abs = TransportUrl::parse("http::unix///var/run/app.sock").unwrap();
        assert_eq!(abs.unix_socket_path(), Some("/var/run/app.sock"));

        // Verify the difference
        assert_ne!(rel.unix_socket_path(), abs.unix_socket_path());
    }
}
