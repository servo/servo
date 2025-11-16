/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Unix domain socket connector for Servo networking

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use http::uri::Uri;
use hyperlocal::UnixConnector as HyperlocalConnector;
use log::debug;
use tower_service::Service;

/// Mapping configuration for URL hostnames to Unix socket paths
#[derive(Clone, Debug)]
pub struct SocketMapping {
    mappings: Arc<RwLock<HashMap<String, PathBuf>>>,
    default_socket_dir: PathBuf,
}

impl SocketMapping {
    /// Create a new socket mapping with a default directory
    pub fn new(default_dir: PathBuf) -> Self {
        Self {
            mappings: Arc::new(RwLock::new(HashMap::new())),
            default_socket_dir: default_dir,
        }
    }

    /// Add a custom mapping from hostname to socket path
    pub fn add_mapping(&self, host: String, socket_path: PathBuf) {
        debug!(
            "Adding UDS mapping: {} -> {}",
            host,
            socket_path.display()
        );
        self.mappings.write().unwrap().insert(host, socket_path);
    }

    /// Get the socket path for a given hostname
    pub fn get_socket_path(&self, host: &str) -> PathBuf {
        self.mappings
            .read()
            .unwrap()
            .get(host)
            .cloned()
            .unwrap_or_else(|| {
                // Default mapping: hostname to socket file in default directory
                let socket_path = self.default_socket_dir.join(format!("{}.sock", host));
                debug!(
                    "No explicit mapping for '{}', using default: {}",
                    host,
                    socket_path.display()
                );
                socket_path
            })
    }
}

/// A connector that uses Unix domain sockets instead of TCP
#[derive(Clone)]
pub struct ServoUnixConnector {
    inner: HyperlocalConnector,
    mapping: SocketMapping,
}

impl ServoUnixConnector {
    /// Create a new Unix socket connector with the given mapping
    pub fn new(mapping: SocketMapping) -> Self {
        Self {
            inner: HyperlocalConnector,
            mapping,
        }
    }
}

impl Service<Uri> for ServoUnixConnector {
    type Response = <HyperlocalConnector as Service<Uri>>::Response;
    type Error = <HyperlocalConnector as Service<Uri>>::Error;
    type Future = <HyperlocalConnector as Service<Uri>>::Future;

    fn call(&mut self, uri: Uri) -> Self::Future {
        // Convert HTTP URL to Unix socket URI
        let socket_uri = if let Some(auth) = uri.authority() {
            let host = auth.host();
            let socket_path = self.mapping.get_socket_path(host);

            // Get the path and query from the original URI
            let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

            debug!(
                "Converting {} to Unix socket: {} (path: {})",
                uri,
                socket_path.display(),
                path_and_query
            );

            // hyperlocal expects URIs like: unix://[socket_path][http_path]
            hyperlocal::Uri::new(&socket_path, path_and_query).into()
        } else {
            // If no authority, pass through as-is
            uri
        };

        self.inner.call(socket_uri)
    }

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
}
