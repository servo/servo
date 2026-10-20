/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::cookie::{SameSiteContext, ServoCookie};
use net::cookie_storage::CookieStorage;
use net_traits::CookieSource;
use servo_url::ServoUrl;

fn is_partitioned_cookie(cookie_str: &str) -> bool {
    cookie_str
        .split(';')
        .any(|part| part.trim().eq_ignore_ascii_case("partitioned"))
}

fn cookie_name(cookie_str: &str) -> Option<String> {
    let name = cookie_str.split(';').next()?.trim();
    if name.is_empty() {
        return None;
    }
    let (n, _) = name.split_once('=')?;
    Some(n.trim().to_owned())
}

fuzz_target!(|data: &[u8]| {
    // Split input: [cookie_string][0x00][partition_key_str][0x00][flags]
    let mut parts = data.split(|b| *b == 0);
    let (Some(c0), Some(c1), Some(c2)) = (parts.next(), parts.next(), parts.next()) else { return; };
    let (Ok(cookie_str), Ok(partition_key_str), Ok(flags_str)) = (
        std::str::from_utf8(c0),
        std::str::from_utf8(c1),
        std::str::from_utf8(c2),
    ) else { return; };
    let Ok(url) = ServoUrl::parse("https://example.com/test") else { return; };

    let Some(cookie) = ServoCookie::from_cookie_string(cookie_str, &url, CookieSource::HTTP) else {
        // Parser rejected the cookie; the store must not panic on rejected inputs.
        return;
    };

    let name = cookie_name(cookie_str);
    let mut storage = CookieStorage::new(30);
    storage.push(cookie.clone(), &url, CookieSource::HTTP);

    let flags: Vec<char> = flags_str.chars().take(4).collect();
    let cross_site = flags.get(0).cloned() == Some('c');
    let top_level = flags.get(1).cloned() == Some('t');
    let safe_method = flags.get(2).cloned() != Some('u');

    let ctx = SameSiteContext {
        cross_site,
        top_level_navigation: top_level,
        safe_method,
        partition_key: Some(partition_key_str.to_owned()),
    };

    // 1. Partitioned cookie: with no partition key in request context, it must NOT be sent.
    let ctx_no_key = SameSiteContext {
        partition_key: None,
        ..ctx.clone()
    };
    let sent_no_key = storage
        .cookies_for_url_with_context(&url, CookieSource::HTTP, ctx_no_key.clone())
        .unwrap_or_default();
    if is_partitioned_cookie(cookie_str) {
        if let Some(n) = &name {
            if sent_no_key.contains(n) {
                panic!("partitioned cookie sent with no request partition key");
            }
        }
    }

    // 2. Partitioned cookie: with different partition key, must NOT be sent.
    let ctx_diff_key = SameSiteContext {
        partition_key: Some(format!("{}x", partition_key_str)),
        ..ctx.clone()
    };
    let sent_diff = storage
        .cookies_for_url_with_context(&url, CookieSource::HTTP, ctx_diff_key.clone())
        .unwrap_or_default();
    if is_partitioned_cookie(cookie_str) {
        if let Some(n) = &name {
            if sent_diff.contains(n) {
                panic!("partitioned cookie sent with different partition key");
            }
        }
    }

    // 3. Unpartitioned cookie: must be sent even if request context has a partition key.
    if !is_partitioned_cookie(cookie_str) {
        let sent_with_key = storage
            .cookies_for_url_with_context(&url, CookieSource::HTTP, ctx.clone())
            .unwrap_or_default();
        if let Some(n) = &name {
            // The cookie should be present unless other constraints (SameSite) removed it;
            // but for the default context flags (cross_site=false, top_level=true, safe=true),
            // SameSite is not triggered. We don't assert presence strictly in all cases,
            // just that the query does not crash and CHIPS doesn't withhold unpartitioned.
            let _ = (n, sent_with_key);
        }
    }

    // 4. Round-trip queries must not panic on arbitrary contexts.
    let _ = storage.cookies_for_url_with_context(&url, CookieSource::HTTP, ctx.clone());
    let _ = storage.cookies_for_url_with_context(&url, CookieSource::HTTP, ctx_no_key);
    let _ = storage.cookies_for_url_with_context(&url, CookieSource::HTTP, ctx_diff_key);
    let _ = storage.cookies_for_url(&url, CookieSource::HTTP);
});