/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::cookie::{ServoCookie, SameSiteContext};
use net_traits::CookieSource;
use servo_url::ServoUrl;

fuzz_target!(|data: &[u8]| {
    // Split input: [host_str][0x00][domain_str]
    let mut parts = data.splitn(2, |b| *b == 0);
    let (Some(h), Some(d)) = (parts.next(), parts.next()) else { return; };
    let (Ok(host_str), Ok(domain_str)) = (std::str::from_utf8(h), std::str::from_utf8(d)) else { return; };

    // 1. Reflexivity: a string should always domain-match itself (case-insensitively).
    assert_eq!(ServoCookie::domain_match(host_str, host_str), true);

    // 2. No panic / basic shape: if matched but not equal, host is longer.
    if ServoCookie::domain_match(host_str, domain_str) && !host_str.eq_ignore_ascii_case(domain_str) {
        assert!(host_str.len() > domain_str.len());
    }

    // 3. IP literals only match themselves.
    if host_str.parse::<std::net::Ipv4Addr>().is_ok() || host_str.parse::<std::net::Ipv6Addr>().is_ok() {
        if ServoCookie::domain_match(host_str, domain_str) {
            assert!(host_str.eq_ignore_ascii_case(domain_str));
        }
    }

    // 4. Parser round-trip with a Domain attribute against a stable request URL.
    // `from_cookie_string` validates the domain-attribute against the request host;
    // if the cookie is accepted, its domain must domain-match the request host or
    // be empty in a specific public-suffix case (which the predicate above checks).
    if let Ok(url) = ServoUrl::parse("https://example.com/test") {
        let cookie_str = format!("n=v; Domain={}", domain_str);
        if let Some(cookie) = ServoCookie::from_cookie_string(&cookie_str, &url, CookieSource::HTTP) {
            let cdom = cookie.cookie.domain().unwrap_or("");
            // After acceptance, domain must domain-match request host or be canonicalized.
            assert!(ServoCookie::domain_match("example.com", cdom) || cdom.eq_ignore_ascii_case("example.com"));
            let _ = cookie.appropriate_for_url(&url, CookieSource::HTTP);
            let _ = cookie.appropriate_for_url_with_context(&url, CookieSource::HTTP, SameSiteContext::SAME_SITE);
        }
    }

    // 5. Ensure other helpers do not panic on arbitrary strings.
    let _ = ServoCookie::path_match("/test", domain_str);
    let _ = ServoCookie::path_match(host_str, "/");
    let _ = ServoCookie::default_path(host_str);
});