/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! http://tools.ietf.org/html/rfc6265

use cookie_rs;
use net_traits::CookieSource;
use net_traits::pub_domains::is_pub_domain;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::net::{Ipv4Addr, Ipv6Addr};
use time::{Tm, now, at, Duration};

/// A stored cookie that wraps the definition in cookie-rs. This is used to implement
/// various behaviours defined in the spec that rely on an associated request URL,
/// which cookie-rs and hyper's header parsing do not support.
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct Cookie {
    pub cookie: cookie_rs::Cookie,
    pub host_only: bool,
    pub persistent: bool,
    pub creation_time: Tm,
    pub last_access: Tm,
    pub expiry_time: Option<Tm>,
}

impl Cookie {
    /// http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn new_wrapped(mut cookie: cookie_rs::Cookie, request: &ServoUrl, source: CookieSource)
                       -> Option<Cookie> {
        // Step 3
        let (persistent, expiry_time) = match (&cookie.max_age, &cookie.expires) {
            (&Some(max_age), _) => {
                (true, Some(at(now().to_timespec() + Duration::seconds(max_age as i64))))
            }
            (_, &Some(expires)) => (true, Some(expires)),
            _ => (false, None)
        };

        let url_host = request.host_str().unwrap_or("").to_owned();

        // Step 4
        let mut domain = cookie.domain.clone().unwrap_or("".to_owned());

        // Step 5
        if is_pub_domain(&domain) {
            if domain == url_host {
                domain = "".to_owned();
            } else {
                return None
            }
        }

        // Step 6
        let host_only = if !domain.is_empty() {
            if !Cookie::domain_match(&url_host, &domain) {
                return None;
            } else {
                cookie.domain = Some(domain);
                false
            }
        } else {
            cookie.domain = Some(url_host);
            true
        };

        // Step 7
        let mut path = cookie.path.unwrap_or("".to_owned());
        if path.chars().next() != Some('/') {
            path = Cookie::default_path(request.path()).to_owned();
        }
        cookie.path = Some(path);


        // Step 10
        if cookie.httponly && source == CookieSource::NonHTTP {
            return None;
        }

        Some(Cookie {
            cookie: cookie,
            host_only: host_only,
            persistent: persistent,
            creation_time: now(),
            last_access: now(),
            expiry_time: expiry_time,
        })
    }

    pub fn touch(&mut self) {
        self.last_access = now();
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.4
    pub fn default_path(request_path: &str) -> &str {
        // Step 2
        if request_path.chars().next() != Some('/') {
            return "/";
        }

        // Step 3
        let rightmost_slash_idx = request_path.rfind("/").unwrap();
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

        (request_path.starts_with(cookie_path) && (
            // The cookie-path is a prefix of the request-path, and the last
            // character of the cookie-path is %x2F ("/").
            cookie_path.ends_with("/") ||
            // The cookie-path is a prefix of the request-path, and the first
            // character of the request-path that is not included in the cookie-
            // path is a %x2F ("/") character.
            request_path[cookie_path.len()..].starts_with("/")
        ))
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.3
    pub fn domain_match(string: &str, domain_string: &str) -> bool {
        string == domain_string ||
        (string.ends_with(domain_string) &&
         string.as_bytes()[string.len()-domain_string.len()-1] == b'.' &&
         string.parse::<Ipv4Addr>().is_err() &&
         string.parse::<Ipv6Addr>().is_err())
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4 step 1
    pub fn appropriate_for_url(&self, url: &ServoUrl, source: CookieSource) -> bool {
        let domain = url.host_str();
        if self.host_only {
            if self.cookie.domain.as_ref().map(String::as_str) != domain {
                return false;
            }
        } else {
            if let (Some(domain), &Some(ref cookie_domain)) = (domain, &self.cookie.domain) {
                if !Cookie::domain_match(domain, cookie_domain) {
                    return false;
                }
            }
        }

        if let Some(ref cookie_path) = self.cookie.path {
            if !Cookie::path_match(url.path(), cookie_path) {
                return false;
            }
        }

        if self.cookie.secure && !url.is_secure_scheme() {
            return false;
        }
        if self.cookie.httponly && source == CookieSource::NonHTTP {
            return false;
        }

        true
    }
}
