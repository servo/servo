/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of cookie creation and matching as specified by
//! http://tools.ietf.org/html/rfc6265

use pub_domains::PUB_DOMAINS;

use cookie_rs;
use time::{Tm, now, at, Timespec};
use url::Url;
use std::borrow::ToOwned;
use std::i64;
use std::io::net::ip::IpAddr;
use std::time::Duration;

/// A stored cookie that wraps the definition in cookie-rs. This is used to implement
/// various behaviours defined in the spec that rely on an associated request URL,
/// which cookie-rs and hyper's header parsing do not support.
#[derive(Clone, Show)]
pub struct Cookie {
    pub cookie: cookie_rs::Cookie,
    pub host_only: bool,
    pub persistent: bool,
    pub created_at: Tm,
    pub last_access: Tm,
    pub scheme: String,
    pub expiry_time: Tm,
}

impl Cookie {
    /// http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn new_wrapped(mut cookie: cookie_rs::Cookie, request: &Url) -> Option<Cookie> {
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
            if !Cookie::domain_match(url_host.as_slice(), domain.as_slice()) {
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
            let mut url_path: String = "".to_owned();
            if let Some(paths) = request.path() {
                for path in paths.iter() {
                    url_path.extend(path.chars());
                }
            }
            path = Cookie::default_path(url_path.as_slice());
        }
        cookie.path = Some(path);


        // Step 10
        if cookie.httponly && !request.scheme.as_slice().starts_with("http") {
            return None;
        }

        Some(Cookie {
            cookie: cookie,
            host_only: host_only,
            persistent: persistent,
            created_at: now(),
            last_access: now(),
            scheme: request.scheme.clone(),
            expiry_time: expiry_time,
        })
    }

    pub fn touch(&mut self) {
        self.last_access = now();
    }

    // http://tools.ietf.org/html/rfc6265#section-5.1.4
    fn default_path(request_path: &str) -> String {
        if request_path == "" || request_path.char_at(0) != '/' || request_path == "/" {
            return "/".to_owned();
        }
        if request_path.ends_with("/") {
            return request_path.slice_to(request_path.len()-1).to_string();
        }
        return request_path.to_owned();
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
            && string.parse::<IpAddr>().is_none() {
            return true;
        }
        false
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4 step 1
    pub fn appropriate_for_url(&self, url: Url) -> bool {
        let domain = url.host().map(|host| host.serialize());
        if self.host_only {
            if self.cookie.domain != domain {
                return false;
            }
        } else {
            if let (Some(ref domain), &Some(ref cookie_domain)) = (domain, &self.cookie.domain) {
                if !Cookie::domain_match(domain.as_slice(), cookie_domain.as_slice()) {
                    return false;
                }
            }
        }

        if let (Some(ref path), &Some(ref cookie_path)) = (url.serialize_path(), &self.cookie.path) {
            if !Cookie::path_match(path.as_slice(), cookie_path.as_slice()) {
                return false;
            }
        }

        if self.cookie.secure && url.scheme != "https".to_string() {
            return false;
        }
        if self.cookie.httponly && !url.scheme.as_slice().starts_with("http") {
            return false;
        }

        return true;
    }
}

#[test]
fn test_domain_match() {
    assert!(Cookie::domain_match("foo.com", "foo.com"));
    assert!(Cookie::domain_match("bar.foo.com", "foo.com"));
    assert!(Cookie::domain_match("baz.bar.foo.com", "foo.com"));

    assert!(!Cookie::domain_match("bar.foo.com", "bar.com"));
    assert!(!Cookie::domain_match("bar.com", "baz.bar.com"));
    assert!(!Cookie::domain_match("foo.com", "bar.com"));
}

#[test]
fn test_default_path() {
    assert!(Cookie::default_path("/foo/bar/baz/").as_slice() == "/foo/bar/baz");
    assert!(Cookie::default_path("/foo").as_slice() == "/foo");
    assert!(Cookie::default_path("/").as_slice() == "/");
    assert!(Cookie::default_path("").as_slice() == "/");
}

#[test]
fn fn_cookie_constructor() {
    let url = &Url::parse("http://example.com/foo").unwrap();

    let gov_url = &Url::parse("http://gov.ac/foo").unwrap();
    // cookie name/value test
    assert!(Cookie::new(" baz ".to_string(), url).is_none());
    assert!(Cookie::new(" = bar  ".to_string(), url).is_none());
    assert!(Cookie::new(" baz = ".to_string(), url).is_some());

    // cookie domains test
    assert!(Cookie::new(" baz = bar; Domain =  ".to_string(), url).is_some());
    assert!(Cookie::new(" baz = bar; Domain =  ".to_string(), url).unwrap().domain.as_slice() == "example.com");

    // cookie public domains test
    assert!(Cookie::new(" baz = bar; Domain =  gov.ac".to_string(), url).is_none());
    assert!(Cookie::new(" baz = bar; Domain =  gov.ac".to_string(), gov_url).is_some());

    // cookie domain matching test
    assert!(Cookie::new(" baz = bar ; Secure; Domain = bazample.com".to_string(), url).is_none());

    assert!(Cookie::new(" baz = bar ; Secure; Path = /foo/bar/".to_string(), url).is_some());

    let cookie = Cookie::new(" baz = bar ; Secure; Path = /foo/bar/".to_string(), url).unwrap();
    assert!(cookie.value.as_slice() == "bar");
    assert!(cookie.name.as_slice() == "baz");
    assert!(cookie.secure);
    assert!(cookie.path.as_slice() == "/foo/bar/");
    assert!(cookie.domain.as_slice() == "example.com");
    assert!(cookie.host_only);

    let u = &Url::parse("http://example.com/foobar").unwrap();
    assert!(Cookie::new("foobar=value;path=/".to_string(), u).is_some());
}

#[deriving(Clone)]
pub struct CookieManager {
    cookies: Vec<Cookie>,
}

impl CookieManager {
    pub fn new() -> CookieManager {
        CookieManager {
            cookies: Vec::new()
        }
    }

    pub fn add(&mut self, cookie: &Cookie) -> bool {
        match self.cookies.iter().find(|x| {
            x.cookie.domain == cookie.cookie.domain &&
            x.cookie.name == cookie.cookie.name &&
            x.cookie.path == cookie.cookie.path
        }) {
            Some(c) => {
                if c.cookie.httponly && !cookie.scheme.as_slice().starts_with("http") {
                    return false
                }
            }
            None => {}
        }
        self.cookies.push(cookie.clone());
        true
    }
}
