/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//use resource_task::{Metadata, Payload, Done, LoadResponse, LoaderTask, start_sending};
//use time::Tm;
use url::Url;
use std::ascii::AsciiExt;
use time::{strptime, Tm, at, get_time, Timespec, now};
use std::i64;
use pub_domains::PUB_DOMAINS;
use std::io::net::ip::IpAddr;

#[derive(Clone, Show)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub expires: Option<Tm>,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub host_only: bool,
    pub persistent: bool,
    pub created_at: Tm,
    pub last_access: Tm,
    pub scheme: String
}

impl Cookie {
    pub fn new(header_value: String, url: &Url) -> Option<Cookie> {
        let mut secure = false;
        let mut http_only = false;
        let mut name = None;
        let mut value = None;
        let mut domain = "".to_string();
        let mut path = "".to_string();
        let mut expires = None;
        let mut max_age = None;

        let mut set_cookie_iter = header_value.as_slice().split(';');
        match set_cookie_iter.next() {
            Some(name_value_pair) => {
                if !name_value_pair.contains_char('=') {
                    return None;
                }
                let mut data = name_value_pair.split('=').map(|x| x.trim());
                name = data.next();
                value = data.next();
            }
            None => { return None }
        }

        if name.is_some() && name.unwrap() == "" {
            return None
        }

        for spl in set_cookie_iter {
            let cookie_av = spl.trim();
            if cookie_av.contains_char('=') {
                match cookie_av.split('=').map(|x| x.trim()).collect::<Vec<&str>>().as_slice() {
                    [attr, val] if attr.eq_ignore_ascii_case("domain") => {
                        if val == "" {
                            continue;
                        }
                        let cookie_domain;
                        if val.char_at(0) == '.' {
                            cookie_domain = val.slice_from(1);
                        } else {
                            cookie_domain = val;
                        }
                        domain = cookie_domain.to_ascii_lowercase();
                    }
                    [attr, val] if attr.eq_ignore_ascii_case("path") => {
                        if val == "" || val.char_at(0) != '/' {
                            match url.path() {
                                Some(x) => {
                                    let mut url_path = "".to_string();
                                    for v in x.iter() {
                                        url_path.push_str(v.as_slice())
                                    }
                                    path = Cookie::default_path(url_path.as_slice())
                                }
                                _ => {
                                    return None
                                }
                            }
                        } else {
                            path = val.to_string();
                        }

                    }
                    [attr, val] if attr.eq_ignore_ascii_case("expires") => {
                        // we try strptime with three date formats according to
                        // http://tools.ietf.org/html/rfc2616#section-3.3.1
                        match strptime(val, "%a, %d %b %Y %H:%M:%S %Z") {
                            Ok(x) => expires = Some(x),
                            Err(_) => {
                                match strptime(val, "%A, %d-%b-%y %H:%M:%S %Z") {
                                    Ok(x) => expires = Some(x),
                                    Err(_) => {
                                        match strptime(val, "%a %b %d %H:%M:%S %Y") {
                                            Ok(x) => expires = Some(x),
                                            Err(_) => continue
                                        }
                                    }
                                }
                            }
                        }
                    }
                    [attr, val] if attr.eq_ignore_ascii_case("max-age") => {
                        match val.parse() {
                            Some(x) if x > 0 => {
                                let mut expires = get_time();
                                expires.sec += x;
                                max_age = Some(at(expires));
                            }
                            Some(_) => {
                                max_age = Some(at(Timespec::new(0, 0)))
                            }
                            None => continue
                        }
                    }
                    x => { println!("Undefined cookie attr value: {:?}", x); }
                }
            } else if cookie_av.eq_ignore_ascii_case("secure") {
                secure = true;
            } else if cookie_av.eq_ignore_ascii_case("httponly") {
                http_only = true;
            } else {
                println!("Undefined cookie attr value: {}", cookie_av)
            }
        }

        let url_host = match url.host() {
            Some(x) => x.serialize(),
            None => "".to_string()
        };
        let mut cookie = Cookie {
            name: name.unwrap().to_string(),
            value: value.unwrap().to_string(),
            created_at: now(),
            last_access: now(),
            domain: url_host.clone(),
            expires: None,
            path: path,
            secure: secure,
            http_only: http_only,
            host_only: true,
            persistent: false,
            scheme: url.scheme.clone()
        };

        if max_age.is_some() {
            cookie.persistent = true;
            cookie.expires = max_age;
        } else if expires.is_some() {
            cookie.persistent = true;
            cookie.expires = expires;
        } else {
            cookie.expires = Some(at(Timespec::new(i64::MAX, 0)))
        }

        match PUB_DOMAINS.iter().find(|&x| domain == *x) {
            Some(val) if *val == url_host => domain = "".to_string(),
            Some(_) => return None,
            None => {}
        }
        if !domain.is_empty() {
            if !Cookie::domain_match(url_host.as_slice(), domain.as_slice()) {
                return None;
            } else {
                cookie.host_only = false;
                cookie.domain = domain;
            }
        }
        if cookie.http_only && !url.scheme.as_slice().starts_with("http") {
            return None;
        }

        Some(cookie)
    }

    pub fn touch(&mut self) {
        self.last_access = now();
    }

    fn default_path(request_path: &str) -> String {
        if request_path == "" || request_path.char_at(0) != '/' || request_path == "/" {
            return "/".to_string();
        }
        if request_path.ends_with("/") {
            return request_path.slice_to(request_path.len()-1).to_string();
        }
        return request_path.clone().to_string();
    }

    pub fn path_match(request_path: &str, cookie_path: &str) -> bool {
        request_path == cookie_path ||
        ( request_path.starts_with(cookie_path) &&
            ( request_path.ends_with("/") || request_path.char_at(cookie_path.len()) == '/' )
        )
    }

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

    pub fn appropriate_for_url(&self, url: Url) -> bool {
        let domain = url.host().unwrap().serialize();
        let mut result = if self.host_only {
            self.domain == domain
        } else {
            Cookie::domain_match(domain.as_slice(), self.domain.as_slice())
        };
        result = result && Cookie::path_match(url.serialize_path().unwrap().as_slice(), self.path.as_slice());

        if self.secure {
            result = result && url.scheme == "https".to_string()
        }
        if self.http_only {
            result = result && url.scheme.as_slice().starts_with("http")
        }
        result
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
        match self.cookies.iter().find(|x| x.domain == cookie.domain && x.name == cookie.name && x.path == cookie.path) {
            Some(c) => {
                if c.http_only && !cookie.scheme.as_slice().starts_with("http") {
                    return false
                }
            }
            None => {}
        }
        self.cookies.push(cookie.clone());
        true
    }
}
