/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie_rs;
use hyper::header::{Header, SetCookie};
use net::cookie::Cookie;
use net::cookie_storage::CookieStorage;
use net_traits::CookieSource;
use servo_url::ServoUrl;

#[test]
fn test_domain_match() {
    assert!(Cookie::domain_match("foo.com", "foo.com"));
    assert!(Cookie::domain_match("bar.foo.com", "foo.com"));
    assert!(Cookie::domain_match("baz.bar.foo.com", "foo.com"));

    assert!(!Cookie::domain_match("bar.foo.com", "bar.com"));
    assert!(!Cookie::domain_match("bar.com", "baz.bar.com"));
    assert!(!Cookie::domain_match("foo.com", "bar.com"));

    assert!(!Cookie::domain_match("bar.com", "bbar.com"));
    assert!(Cookie::domain_match("235.132.2.3", "235.132.2.3"));
    assert!(!Cookie::domain_match("235.132.2.3", "1.1.1.1"));
    assert!(!Cookie::domain_match("235.132.2.3", ".2.3"));
}

#[test]
fn test_path_match() {
    assert!(Cookie::path_match("/", "/"));
    assert!(Cookie::path_match("/index.html", "/"));
    assert!(Cookie::path_match("/w/index.html", "/"));
    assert!(Cookie::path_match("/w/index.html", "/w/index.html"));
    assert!(Cookie::path_match("/w/index.html", "/w/"));
    assert!(Cookie::path_match("/w/index.html", "/w"));

    assert!(!Cookie::path_match("/", "/w/"));
    assert!(!Cookie::path_match("/a", "/w/"));
    assert!(!Cookie::path_match("/", "/w"));
    assert!(!Cookie::path_match("/w/index.html", "/w/index"));
    assert!(!Cookie::path_match("/windex.html", "/w/"));
    assert!(!Cookie::path_match("/windex.html", "/w"));
}

#[test]
fn test_default_path() {
    assert!(&*Cookie::default_path("/foo/bar/baz/") == "/foo/bar/baz");
    assert!(&*Cookie::default_path("/foo/bar/baz") == "/foo/bar");
    assert!(&*Cookie::default_path("/foo/") == "/foo");
    assert!(&*Cookie::default_path("/foo") == "/");
    assert!(&*Cookie::default_path("/") == "/");
    assert!(&*Cookie::default_path("") == "/");
    assert!(&*Cookie::default_path("foo") == "/");
}

#[test]
fn fn_cookie_constructor() {
    use net_traits::CookieSource;

    let url = &ServoUrl::parse("http://example.com/foo").unwrap();

    let gov_url = &ServoUrl::parse("http://gov.ac/foo").unwrap();
    // cookie name/value test
    assert!(cookie_rs::Cookie::parse(" baz ").is_err());
    assert!(cookie_rs::Cookie::parse(" = bar  ").is_err());
    assert!(cookie_rs::Cookie::parse(" baz = ").is_ok());

    // cookie domains test
    let cookie = cookie_rs::Cookie::parse(" baz = bar; Domain =  ").unwrap();
    assert!(Cookie::new_wrapped(cookie.clone(), url, CookieSource::HTTP).is_some());
    let cookie = Cookie::new_wrapped(cookie, url, CookieSource::HTTP).unwrap();
    assert!(&**cookie.cookie.domain().as_ref().unwrap() == "example.com");

    // cookie public domains test
    let cookie = cookie_rs::Cookie::parse(" baz = bar; Domain =  gov.ac").unwrap();
    assert!(Cookie::new_wrapped(cookie.clone(), url, CookieSource::HTTP).is_none());
    assert!(Cookie::new_wrapped(cookie, gov_url, CookieSource::HTTP).is_some());

    // cookie domain matching test
    let cookie = cookie_rs::Cookie::parse(" baz = bar ; Secure; Domain = bazample.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let cookie = cookie_rs::Cookie::parse(" baz = bar ; Secure; Path = /foo/bar/").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_some());

    let cookie = cookie_rs::Cookie::parse(" baz = bar ; HttpOnly").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::NonHTTP).is_none());

    let cookie = cookie_rs::Cookie::parse(" baz = bar ; Secure; Path = /foo/bar/").unwrap();
    let cookie = Cookie::new_wrapped(cookie, url, CookieSource::HTTP).unwrap();
    assert!(cookie.cookie.value() == "bar");
    assert!(cookie.cookie.name() == "baz");
    assert!(cookie.cookie.secure());
    assert!(&cookie.cookie.path().as_ref().unwrap()[..] == "/foo/bar/");
    assert!(&cookie.cookie.domain().as_ref().unwrap()[..] == "example.com");
    assert!(cookie.host_only);

    let u = &ServoUrl::parse("http://example.com/foobar").unwrap();
    let cookie = cookie_rs::Cookie::parse("foobar=value;path=/").unwrap();
    assert!(Cookie::new_wrapped(cookie, u, CookieSource::HTTP).is_some());
}

#[test]
fn test_cookie_secure_prefix() {
    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Secure-SID=12345").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("http://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Secure-SID=12345; Secure").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Secure-SID=12345; Secure").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_some());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Secure-SID=12345; Domain=example.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("http://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Secure-SID=12345; Secure; Domain=example.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Secure-SID=12345; Secure; Domain=example.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_some());
}

#[test]
fn test_cookie_host_prefix() {
    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("http://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Secure").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Secure").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Domain=example.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Domain=example.com; Path=/").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("http://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Secure; Domain=example.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Secure; Domain=example.com").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Secure; Domain=example.com; Path=/").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_none());

    let url = &ServoUrl::parse("https://example.com").unwrap();
    let cookie = cookie_rs::Cookie::parse("__Host-SID=12345; Secure; Path=/").unwrap();
    assert!(Cookie::new_wrapped(cookie, url, CookieSource::HTTP).is_some());
}

#[cfg(target_os = "windows")]
fn delay_to_ensure_different_timestamp() {
    use std::thread;
    use std::time::Duration;

    // time::now()'s resolution on some platforms isn't granular enought to ensure
    // that two back-to-back calls to Cookie::new_wrapped generate different timestamps .
    thread::sleep(Duration::from_millis(500));
}

#[cfg(not(target_os = "windows"))]
fn delay_to_ensure_different_timestamp() {}

#[test]
fn test_sort_order() {
    use std::cmp::Ordering;

    let url = &ServoUrl::parse("http://example.com/foo").unwrap();
    let a_wrapped = cookie_rs::Cookie::parse("baz=bar; Path=/foo/bar/").unwrap();
    let a = Cookie::new_wrapped(a_wrapped.clone(), url, CookieSource::HTTP).unwrap();
    delay_to_ensure_different_timestamp();
    let a_prime = Cookie::new_wrapped(a_wrapped, url, CookieSource::HTTP).unwrap();
    let b = cookie_rs::Cookie::parse("baz=bar;Path=/foo/bar/baz/").unwrap();
    let b = Cookie::new_wrapped(b, url, CookieSource::HTTP).unwrap();

    assert!(b.cookie.path().as_ref().unwrap().len() > a.cookie.path().as_ref().unwrap().len());
    assert!(CookieStorage::cookie_comparator(&a, &b) == Ordering::Greater);
    assert!(CookieStorage::cookie_comparator(&b, &a) == Ordering::Less);
    assert!(CookieStorage::cookie_comparator(&a, &a_prime) == Ordering::Less);
    assert!(CookieStorage::cookie_comparator(&a_prime, &a) == Ordering::Greater);
    assert!(CookieStorage::cookie_comparator(&a, &a) == Ordering::Equal);
}

fn add_cookie_to_storage(storage: &mut CookieStorage, url: &ServoUrl, cookie_str: &str)
{
    let source = CookieSource::HTTP;
    let cookie = cookie_rs::Cookie::parse(cookie_str.to_owned()).unwrap();
    let cookie = Cookie::new_wrapped(cookie, url, source).unwrap();
    storage.push(cookie, url, source);
}

#[test]
fn test_insecure_cookies_cannot_evict_secure_cookie() {
    let mut storage = CookieStorage::new(5);
    let secure_url = ServoUrl::parse("https://home.example.org:8888/cookie-parser?0001").unwrap();
    let source = CookieSource::HTTP;
    let mut cookies = Vec::new();

    cookies.push(cookie_rs::Cookie::parse("foo=bar; Secure; Domain=home.example.org").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo2=bar; Secure; Domain=.example.org").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo3=bar; Secure; Path=/foo").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo4=bar; Secure; Path=/foo/bar").unwrap());

    for bare_cookie in cookies {
        let cookie = Cookie::new_wrapped(bare_cookie, &secure_url, source).unwrap();
        storage.push(cookie, &secure_url, source);
    }

    let insecure_url = ServoUrl::parse("http://home.example.org:8888/cookie-parser?0001").unwrap();

    add_cookie_to_storage(&mut storage, &insecure_url, "foo=value; Domain=home.example.org");
    add_cookie_to_storage(&mut storage, &insecure_url, "foo2=value; Domain=.example.org");
    add_cookie_to_storage(&mut storage, &insecure_url, "foo3=value; Path=/foo/bar");
    add_cookie_to_storage(&mut storage, &insecure_url, "foo4=value; Path=/foo");

    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&secure_url, source).unwrap(), "foo=bar; foo2=bar");

    let url = ServoUrl::parse("https://home.example.org:8888/foo/cookie-parser-result?0001").unwrap();
    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(), "foo3=bar; foo4=value; foo=bar; foo2=bar");

    let url = ServoUrl::parse("https://home.example.org:8888/foo/bar/cookie-parser-result?0001").unwrap();
    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(), "foo4=bar; foo3=bar; foo4=value; foo=bar; foo2=bar");
}

#[test]
fn test_secure_cookies_eviction() {
    let mut storage = CookieStorage::new(5);
    let url = ServoUrl::parse("https://home.example.org:8888/cookie-parser?0001").unwrap();
    let source = CookieSource::HTTP;
    let mut cookies = Vec::new();

    cookies.push(cookie_rs::Cookie::parse("foo=bar; Secure; Domain=home.example.org").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo2=bar; Secure; Domain=.example.org").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo3=bar; Secure; Path=/foo").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo4=bar; Secure; Path=/foo/bar").unwrap());

    for bare_cookie in cookies {
        let cookie = Cookie::new_wrapped(bare_cookie, &url, source).unwrap();
        storage.push(cookie, &url, source);
    }

    add_cookie_to_storage(&mut storage, &url, "foo=value; Domain=home.example.org");
    add_cookie_to_storage(&mut storage, &url, "foo2=value; Domain=.example.org");
    add_cookie_to_storage(&mut storage, &url, "foo3=value; Path=/foo/bar");
    add_cookie_to_storage(&mut storage, &url, "foo4=value; Path=/foo");

    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(), "foo2=value");

    let url = ServoUrl::parse("https://home.example.org:8888/foo/cookie-parser-result?0001").unwrap();
    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(), "foo3=bar; foo4=value; foo2=value");

    let url = ServoUrl::parse("https://home.example.org:8888/foo/bar/cookie-parser-result?0001").unwrap();
    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(),
               "foo4=bar; foo3=value; foo3=bar; foo4=value; foo2=value");
}

#[test]
fn test_secure_cookies_eviction_non_http_source() {
    let mut storage = CookieStorage::new(5);
    let url = ServoUrl::parse("https://home.example.org:8888/cookie-parser?0001").unwrap();
    let source = CookieSource::NonHTTP;
    let mut cookies = Vec::new();

    cookies.push(cookie_rs::Cookie::parse("foo=bar; Secure; Domain=home.example.org").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo2=bar; Secure; Domain=.example.org").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo3=bar; Secure; Path=/foo").unwrap());
    cookies.push(cookie_rs::Cookie::parse("foo4=bar; Secure; Path=/foo/bar").unwrap());

    for bare_cookie in cookies {
        let cookie = Cookie::new_wrapped(bare_cookie, &url, source).unwrap();
        storage.push(cookie, &url, source);
    }

    add_cookie_to_storage(&mut storage, &url, "foo=value; Domain=home.example.org");
    add_cookie_to_storage(&mut storage, &url, "foo2=value; Domain=.example.org");
    add_cookie_to_storage(&mut storage, &url, "foo3=value; Path=/foo/bar");
    add_cookie_to_storage(&mut storage, &url, "foo4=value; Path=/foo");

    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(), "foo2=value");

    let url = ServoUrl::parse("https://home.example.org:8888/foo/cookie-parser-result?0001").unwrap();
    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(), "foo3=bar; foo4=value; foo2=value");

    let url = ServoUrl::parse("https://home.example.org:8888/foo/bar/cookie-parser-result?0001").unwrap();
    let source = CookieSource::HTTP;
    assert_eq!(storage.cookies_for_url(&url, source).unwrap(),
               "foo4=bar; foo3=value; foo3=bar; foo4=value; foo2=value");
}


fn add_retrieve_cookies(set_location: &str,
                        set_cookies: &[String],
                        final_location: &str)
                        -> String {
    let mut storage = CookieStorage::new(5);
    let url = ServoUrl::parse(set_location).unwrap();
    let source = CookieSource::HTTP;

    // Add all cookies to the store
    for str_cookie in set_cookies {
        let bytes = str_cookie.to_string().into_bytes();
        let header = Header::parse_header(&[bytes]).unwrap();
        let SetCookie(cookies) = header;
        for bare_cookie in cookies {
            let cookie = Cookie::from_cookie_string(bare_cookie, &url, source).unwrap();
            storage.push(cookie, &url, source);
        }
    }

    // Get cookies for the test location
    let url = ServoUrl::parse(final_location).unwrap();
    storage.cookies_for_url(&url, source).unwrap_or("".to_string())
}


#[test]
fn test_cookie_eviction_expired() {
    let mut vec = Vec::new();
    for i in 1..6 {
        let st = format!("extra{}=bar; Secure; expires=Sun, 18-Apr-2000 21:06:29 GMT",
                         i);
        vec.push(st);
    }
    vec.push("foo=bar; Secure; expires=Sun, 18-Apr-2027 21:06:29 GMT".to_owned());
    let r = add_retrieve_cookies("https://home.example.org:8888/cookie-parser?0001",
                                 &vec, "https://home.example.org:8888/cookie-parser-result?0001");
    assert_eq!(&r, "foo=bar");
}


#[test]
fn test_cookie_eviction_all_secure_one_nonsecure() {
    let mut vec = Vec::new();
    for i in 1..5 {
        let st = format!("extra{}=bar; Secure; expires=Sun, 18-Apr-2026 21:06:29 GMT",
                         i);
        vec.push(st);
    }
    vec.push("foo=bar; expires=Sun, 18-Apr-2026 21:06:29 GMT".to_owned());
    vec.push("foo2=bar; Secure; expires=Sun, 18-Apr-2028 21:06:29 GMT".to_owned());
    let r = add_retrieve_cookies("https://home.example.org:8888/cookie-parser?0001",
                                 &vec, "https://home.example.org:8888/cookie-parser-result?0001");
    assert_eq!(&r, "extra1=bar; extra2=bar; extra3=bar; extra4=bar; foo2=bar");
}


#[test]
fn test_cookie_eviction_all_secure_new_nonsecure() {
    let mut vec = Vec::new();
    for i in 1..6 {
        let st = format!("extra{}=bar; Secure; expires=Sun, 18-Apr-2026 21:06:29 GMT",
                         i);
        vec.push(st);
    }
    vec.push("foo=bar; expires=Sun, 18-Apr-2077 21:06:29 GMT".to_owned());
    let r = add_retrieve_cookies("https://home.example.org:8888/cookie-parser?0001",
                                 &vec, "https://home.example.org:8888/cookie-parser-result?0001");
    assert_eq!(&r, "extra1=bar; extra2=bar; extra3=bar; extra4=bar; extra5=bar");
}


#[test]
fn test_cookie_eviction_all_nonsecure_new_secure() {
    let mut vec = Vec::new();
    for i in 1..6 {
        let st = format!("extra{}=bar; expires=Sun, 18-Apr-2026 21:06:29 GMT", i);
        vec.push(st);
    }
    vec.push("foo=bar; Secure; expires=Sun, 18-Apr-2077 21:06:29 GMT".to_owned());
    let r = add_retrieve_cookies("https://home.example.org:8888/cookie-parser?0001",
                                 &vec, "https://home.example.org:8888/cookie-parser-result?0001");
    assert_eq!(&r, "extra2=bar; extra3=bar; extra4=bar; extra5=bar; foo=bar");
}


#[test]
fn test_cookie_eviction_all_nonsecure_new_nonsecure() {
    let mut vec = Vec::new();
    for i in 1..6 {
        let st = format!("extra{}=bar; expires=Sun, 18-Apr-2026 21:06:29 GMT", i);
        vec.push(st);
    }
    vec.push("foo=bar; expires=Sun, 18-Apr-2077 21:06:29 GMT".to_owned());
    let r = add_retrieve_cookies("https://home.example.org:8888/cookie-parser?0001",
                                 &vec, "https://home.example.org:8888/cookie-parser-result?0001");
    assert_eq!(&r, "extra2=bar; extra3=bar; extra4=bar; extra5=bar; foo=bar");
}
