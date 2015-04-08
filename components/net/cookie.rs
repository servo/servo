/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! http://tools.ietf.org/html/rfc6265

use net_traits::CookieSource;
use pub_domains::PUB_DOMAINS;

use cookie_rs;
use time::{Tm, now, at, Timespec};
use url::Url;
use std::borrow::ToOwned;
use std::i64;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use std::str::FromStr;

/// A stored cookie that wraps the definition in cookie-rs. This is used to implement
/// various behaviours defined in the spec that rely on an associated request URL,
/// which cookie-rs and hyper's header parsing do not support.
#[derive(Clone, Debug)]
pub struct Cookie {
    pub cookie: cookie_rs::Cookie,
    pub host_only: bool,
    pub persistent: bool,
    pub creation_time: Tm,
    pub last_access: Tm,
    pub expiry_time: Tm,
}

impl Cookie {
    /// http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn new_wrapped(mut cookie: cookie_rs::Cookie, request: &Url, source: CookieSource)
                       -> Option<Cookie> {
        // Step 3
        let (persistent, expiry_time) = match (&cookie.max_age, &cookie.expires) {
            (&Some(max_age), _) => (true, at(now().to_timespec() + Duration::seconds(max_age as i64))),
            (_, &Some(expires)) => (true, expires),
            _ => (false, at(Timespec::new(i64::MAX, 0)))
        };

        let url_host = request.host().map(|host| host.serialize()).unwrap_or("".to_owned());

        // Step 4
        let mut domain = cookie.domain.clone().unwrap_or("".to_owned());

        // Step 5
        match PUB_DOMAINS.iter().find(|&x| domain == *x) {
            Some(val) if *val == url_host => domain = "".to_string(),
            Some(_) => return None,
            None => {}
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
        if path.is_empty() || path.char_at(0) != '/' {
            let url_path = request.serialize_path();
            let url_path = url_path.as_ref().map(|path| &**path);
            path = Cookie::default_path(url_path.unwrap_or("")).to_owned();
        }
        cookie.path = Some(path);


        // Step 10
        if cookie.httponly && source != CookieSource::HTTP {
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
        if request_path.is_empty() || !request_path.starts_with("/") {
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
        request_path == cookie_path ||
        ( request_path.starts_with(cookie_path) &&
            ( request_path.ends_with("/") || request_path.char_at(cookie_path.len() - 1) == '/' )
        )
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.3
    pub fn domain_match(string: &str, domain_string: &str) -> bool {
        if string == domain_string {
            return true;
        }
        if string.ends_with(domain_string)
            && string.char_at(string.len()-domain_string.len()-1) == '.'
            && Ipv4Addr::from_str(string).is_err()
            && Ipv6Addr::from_str(string).is_err() {
            return true;
        }
        false
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4 step 1
    pub fn appropriate_for_url(&self, url: &Url, source: CookieSource) -> bool {
        let domain = url.host().map(|host| host.serialize());
        if self.host_only {
            if self.cookie.domain != domain {
                return false;
            }
        } else {
            if let (Some(ref domain), &Some(ref cookie_domain)) = (domain, &self.cookie.domain) {
                if !Cookie::domain_match(domain, cookie_domain) {
                    return false;
                }
            }
        }

        if let (Some(ref path), &Some(ref cookie_path)) = (url.serialize_path(), &self.cookie.path) {
            if !Cookie::path_match(path, cookie_path) {
                return false;
            }
        }

        if self.cookie.secure && url.scheme != "https".to_string() {
            return false;
        }
        if self.cookie.httponly && source == CookieSource::NonHTTP {
            return false;
        }

        return true;
    }
}
