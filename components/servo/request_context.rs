/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

/// Proxy configuration for a request context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProxyConfig {
    /// The proxy mode to use.
    pub mode: ProxyMode,
    /// The proxy server address in "host:port" format.
    pub server: Option<String>,
    /// Comma-separated list of hosts to bypass the proxy.
    pub bypass_list: Option<String>,
}

/// Proxy mode constants matching CEF's proxy mode values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum ProxyMode {
    /// Direct connection, no proxy.
    #[default]
    Direct = 0,
    /// Auto-detect proxy settings (WPAD).
    AutoDetect = 1,
    /// Use a PAC script (not yet implemented).
    PacScript = 2,
    /// Use fixed proxy servers.
    FixedServers = 3,
    /// Use system proxy settings.
    System = 4,
}

impl ProxyMode {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ProxyMode::Direct),
            1 => Some(ProxyMode::AutoDetect),
            2 => Some(ProxyMode::PacScript),
            3 => Some(ProxyMode::FixedServers),
            4 => Some(ProxyMode::System),
            _ => None,
        }
    }
}

/// Request context configuration for profile/cookie isolation, proxy config,
/// cache path selection, and accept language.
///
/// Each `RequestContext` can be used to create isolated browsing profiles.
/// Multiple `WebView`s can share a `RequestContext` to share cookies, cache,
/// and other site data.
#[derive(Clone, Debug, Default)]
pub struct RequestContext {
    /// Path to the cookie store directory.
    /// If `None`, cookies are stored in memory only.
    pub cookie_store_path: Option<PathBuf>,
    /// Path to the HTTP cache directory.
    /// If `None`, cache is stored in memory only.
    pub cache_path: Option<PathBuf>,
    /// Path to the general config directory (for HSTS, auth cache, etc.).
    /// If `None`, uses the global config directory.
    pub config_dir: Option<PathBuf>,
    /// Proxy configuration.
    pub proxy_config: ProxyConfig,
    /// Accept-Language header value to send with requests.
    pub accept_language: Option<String>,
    /// Whether to use temporary storage (data not persisted across restarts).
    pub temporary_storage: bool,
}

impl RequestContext {
    /// Creates a new `RequestContext` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the cookie store path.
    pub fn cookie_store_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.cookie_store_path = Some(path.into());
        self
    }

    /// Sets the cache path.
    pub fn cache_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_path = Some(path.into());
        self
    }

    /// Sets the config directory path.
    pub fn config_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_dir = Some(path.into());
        self
    }

    /// Sets the proxy configuration.
    pub fn proxy_config(mut self, config: ProxyConfig) -> Self {
        self.proxy_config = config;
        self
    }

    /// Sets the accept language.
    pub fn accept_language(mut self, lang: impl Into<String>) -> Self {
        self.accept_language = Some(lang.into());
        self
    }

    /// Sets whether to use temporary storage.
    pub fn temporary_storage(mut self, temporary: bool) -> Self {
        self.temporary_storage = temporary;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_default() {
        let ctx = RequestContext::new();
        assert!(ctx.cookie_store_path.is_none());
        assert!(ctx.cache_path.is_none());
        assert!(ctx.config_dir.is_none());
        assert_eq!(ctx.proxy_config.mode, ProxyMode::Direct);
        assert!(ctx.accept_language.is_none());
        assert!(!ctx.temporary_storage);
    }

    #[test]
    fn test_request_context_builder() {
        let ctx = RequestContext::new()
            .cookie_store_path("/tmp/cookies")
            .cache_path("/tmp/cache")
            .config_dir("/tmp/config")
            .proxy_config(ProxyConfig {
                mode: ProxyMode::FixedServers,
                server: Some("proxy.example.com:8080".to_string()),
                bypass_list: Some("localhost,127.0.0.1".to_string()),
            })
            .accept_language("en-US,en;q=0.9")
            .temporary_storage(true);

        assert_eq!(ctx.cookie_store_path, Some("/tmp/cookies".into()));
        assert_eq!(ctx.cache_path, Some("/tmp/cache".into()));
        assert_eq!(ctx.config_dir, Some("/tmp/config".into()));
        assert_eq!(ctx.proxy_config.mode, ProxyMode::FixedServers);
        assert_eq!(ctx.proxy_config.server, Some("proxy.example.com:8080".to_string()));
        assert_eq!(ctx.accept_language, Some("en-US,en;q=0.9".to_string()));
        assert!(ctx.temporary_storage);
    }
}