/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! <http://tools.ietf.org/html/rfc6265>

use std::borrow::ToOwned;
use std::net::{Ipv4Addr, Ipv6Addr};

use cookie::Cookie;
use net_traits::pub_domains::is_pub_domain;
use net_traits::CookieSource;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use time::{now, Tm};
use time_03::OffsetDateTime;

/// A stored cookie that wraps the definition in cookie-rs. This is used to implement
/// various behaviours defined in the spec that rely on an associated request URL,
/// which cookie-rs and hyper's header parsing do not support.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServoCookie {
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    pub cookie: Cookie<'static>,
    pub host_only: bool,
    pub persistent: bool,
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    pub creation_time: Tm,
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    pub last_access: Tm,
    pub expiry_time: Option<OffsetDateTime>,
}

impl ServoCookie {
    pub fn from_cookie_string(
        cookie_str: String,
        request: &ServoUrl,
        source: CookieSource,
    ) -> Option<ServoCookie> {
        Cookie::parse(cookie_str)
            .ok()
            .map(|cookie| ServoCookie::new_wrapped(cookie, request, source))
            .unwrap_or(None)
    }

    /// <http://tools.ietf.org/html/rfc6265#section-5.3>
    pub fn new_wrapped(
        mut cookie: Cookie<'static>,
        request: &ServoUrl,
        source: CookieSource,
    ) -> Option<ServoCookie> {
        // Step 3
        let (persistent, expiry_time) = match (cookie.max_age(), cookie.expires_datetime()) {
            (Some(max_age), _) => (true, Some(time_03::OffsetDateTime::now_utc() + max_age)),
            (_, Some(date_time)) => (true, Some(date_time)),
            _ => (false, None),
        };

        let url_host = request.host_str().unwrap_or("").to_owned();

        // Step 4
        let mut domain = cookie.domain().unwrap_or("").to_owned();

        // Step 5
        if is_pub_domain(&domain) {
            if domain == url_host {
                domain = "".to_string();
            } else {
                return None;
            }
        }

        // Step 6
        let host_only = if !domain.is_empty() {
            if !ServoCookie::domain_match(&url_host, &domain) {
                return None;
            } else {
                cookie.set_domain(domain);
                false
            }
        } else {
            cookie.set_domain(url_host);
            true
        };

        // Step 7
        let mut has_path_specified = true;
        let mut path = cookie
            .path()
            .unwrap_or_else(|| {
                has_path_specified = false;
                ""
            })
            .to_owned();
        if !path.starts_with('/') {
            path = ServoCookie::default_path(request.path()).to_string();
        }
        cookie.set_path(path);

        // Step 10
        if cookie.http_only().unwrap_or(false) && source == CookieSource::NonHTTP {
            return None;
        }

        // https://tools.ietf.org/html/draft-west-cookie-prefixes-04#section-4
        // Step 1 of cookie prefixes
        if (cookie.name().starts_with("__Secure-") || cookie.name().starts_with("__Host-")) &&
            !(cookie.secure().unwrap_or(false) && request.is_secure_scheme())
        {
            return None;
        }

        // Step 2 of cookie prefixes
        if cookie.name().starts_with("__Host-") &&
            !(host_only && has_path_specified && cookie.path().unwrap() == "/")
        {
            return None;
        }

        Some(ServoCookie {
            cookie,
            host_only,
            persistent,
            creation_time: now(),
            last_access: now(),
            expiry_time,
        })
    }

    pub fn touch(&mut self) {
        self.last_access = now();
    }

    pub fn set_expiry_time_in_past(&mut self) {
        self.expiry_time = Some(time_03::OffsetDateTime::UNIX_EPOCH);
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.4
    pub fn default_path(request_path: &str) -> &str {
        // Step 2
        if !request_path.starts_with('/') {
            return "/";
        }

        // Step 3
        let rightmost_slash_idx = request_path.rfind('/').unwrap();
        if rightmost_slash_idx == 0 {
            // There's only one slash; it's the first character
            return "/";
        }

        // Step 4
        &request_path[..rightmost_slash_idx]
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.4
    pub fn path_match(request_path: &str, cookie_path: &str) -> bool {
        // A request-path path-matches a given cookie-path if at least one of
        // the following conditions holds:

        // The cookie-path and the request-path are identical.
        request_path == cookie_path ||
            (request_path.starts_with(cookie_path) &&
                (
                    // The cookie-path is a prefix of the request-path, and the last
                    // character of the cookie-path is %x2F ("/").
                    cookie_path.ends_with('/') ||
            // The cookie-path is a prefix of the request-path, and the first
            // character of the request-path that is not included in the cookie-
            // path is a %x2F ("/") character.
            request_path[cookie_path.len()..].starts_with('/')
                ))
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.3
    pub fn domain_match(string: &str, domain_string: &str) -> bool {
        let string = &string.to_lowercase();
        let domain_string = &domain_string.to_lowercase();

        string == domain_string ||
            (string.ends_with(domain_string) &&
                string.as_bytes()[string.len() - domain_string.len() - 1] == b'.' &&
                string.parse::<Ipv4Addr>().is_err() &&
                string.parse::<Ipv6Addr>().is_err())
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4 step 1
    pub fn appropriate_for_url(&self, url: &ServoUrl, source: CookieSource) -> bool {
        let domain = url.host_str();
        if self.host_only {
            if self.cookie.domain() != domain {
                return false;
            }
        } else if let (Some(domain), Some(cookie_domain)) = (domain, &self.cookie.domain()) {
            if !ServoCookie::domain_match(domain, cookie_domain) {
                return false;
            }
        }

        if let Some(cookie_path) = self.cookie.path() {
            if !ServoCookie::path_match(url.path(), cookie_path) {
                return false;
            }
        }

        if self.cookie.secure().unwrap_or(false) && !url.is_secure_scheme() {
            return false;
        }
        if self.cookie.http_only().unwrap_or(false) && source == CookieSource::NonHTTP {
            return false;
        }

        true
    }
}
