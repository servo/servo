/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! http://tools.ietf.org/html/rfc6265

use cookie_rs;
use net_traits::CookieSource;
use pub_domains::PUB_DOMAINS;
use rustc_serialize::{Encodable, Encoder};
use std::borrow::ToOwned;
use std::net::{Ipv4Addr, Ipv6Addr};
use time::{Tm, now, at, Duration};
use url::Url;

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
    pub expiry_time: Option<Tm>,
}

impl Cookie {
    /// http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn new_wrapped(mut cookie: cookie_rs::Cookie, request: &Url, source: CookieSource)
                       -> Option<Cookie> {
        // Step 3
        let (persistent, expiry_time) = match (&cookie.max_age, &cookie.expires) {
            (&Some(max_age), _) => {
                (true, Some(at(now().to_timespec() + Duration::seconds(max_age as i64))))
            }
            (_, &Some(expires)) => (true, Some(expires)),
            _ => (false, None)
        };

        let url_host = request.host().map_or("".to_owned(), |host| host.serialize());

        // Step 4
        let mut domain = cookie.domain.clone().unwrap_or("".to_owned());

        // Step 5
        match PUB_DOMAINS.iter().find(|&x| domain == *x) {
            Some(val) if *val == url_host => domain = "".to_owned(),
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
        if path.chars().next() != Some('/') {
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
        if string == domain_string {
            return true;
        }
        if string.ends_with(domain_string) &&
            string.as_bytes()[string.len()-domain_string.len()-1] == b'.' &&
            string.parse::<Ipv4Addr>().is_err() &&
            string.parse::<Ipv6Addr>().is_err() {
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

        if self.cookie.secure && url.scheme != "https" {
            return false;
        }
        if self.cookie.httponly && source == CookieSource::NonHTTP {
            return false;
        }

        true
    }
}

impl Encodable for Cookie {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("Cookie", 6, |e| {
            try!(e.emit_struct_field("cookie", 0, |e| RsCookie(self.cookie.clone()).encode(e)));
            try!(e.emit_struct_field("host_only", 1, |e| self.host_only.encode(e)));
            try!(e.emit_struct_field("persistent", 2, |e| self.persistent.encode(e)));
            try!(e.emit_struct_field("creation_time", 3, |e| Time(self.creation_time).encode(e)));
            try!(e.emit_struct_field("last_access", 4, |e| Time(self.last_access).encode(e)));
            match self.expiry_time {
                Some(time) => try!(e.emit_struct_field("expiry_time", 5, |e| Time(time).encode(e))),
                None => {},
            }
            Ok(())
        })
    }
}

struct Time(Tm);

impl Encodable for Time {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let Time(time) = *self;
        s.emit_struct("Time", 11, |e| {
            try!(e.emit_struct_field("tm_sec", 0, |e| time.tm_sec.encode(e)));
            try!(e.emit_struct_field("tm_min", 1, |e| time.tm_min.encode(e)));
            try!(e.emit_struct_field("tm_hour", 2, |e| time.tm_hour.encode(e)));
            try!(e.emit_struct_field("tm_mday", 3, |e| time.tm_mday.encode(e)));
            try!(e.emit_struct_field("tm_mon", 4, |e| time.tm_mon.encode(e)));
            try!(e.emit_struct_field("tm_year", 5, |e| time.tm_year.encode(e)));
            try!(e.emit_struct_field("tm_wday", 6, |e| time.tm_wday.encode(e)));
            try!(e.emit_struct_field("tm_yday", 7, |e| time.tm_yday.encode(e)));
            try!(e.emit_struct_field("tm_isdst", 8, |e| time.tm_isdst.encode(e)));
            try!(e.emit_struct_field("tm_utcoff", 9, |e| time.tm_utcoff.encode(e)));
            try!(e.emit_struct_field("tm_nsec", 10, |e| time.tm_nsec.encode(e)));
            Ok(())
        })
    }
}

struct RsCookie(cookie_rs::Cookie);

impl Encodable for RsCookie {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let RsCookie(ref rs_cookie) = *self;
        s.emit_struct("RsCookie", 9, |e| {
            try!(e.emit_struct_field("name", 0, |e| rs_cookie.name.encode(e)));
            try!(e.emit_struct_field("value", 1, |e| rs_cookie.value.encode(e)));
            match rs_cookie.expires {
                Some(time) => try!(e.emit_struct_field("expires", 2, |e| Time(time).encode(e))),
                None => {},
            }
            try!(e.emit_struct_field("max_age", 3, |e| rs_cookie.max_age.encode(e)));
            try!(e.emit_struct_field("domain", 4, |e| rs_cookie.domain.encode(e)));
            try!(e.emit_struct_field("path", 5, |e| rs_cookie.path.encode(e)));
            try!(e.emit_struct_field("secure", 6, |e| rs_cookie.secure.encode(e)));
            try!(e.emit_struct_field("httponly", 7, |e| rs_cookie.httponly.encode(e)));
            try!(e.emit_struct_field("custom", 8, |e| rs_cookie.custom.encode(e)));
            Ok(())
        })
    }
}
