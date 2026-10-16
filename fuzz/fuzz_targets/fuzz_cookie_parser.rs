/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![no_main]

use libfuzzer_sys::fuzz_target;
use net::cookie::{CookieSource, ServoCookie};
use servo_url::ServoUrl;

fuzz_target!(|data: &[u8]| {
    if let Ok(cookie_str) = std::str::from_utf8(data) {
        if let Ok(url) = ServoUrl::parse("https://example.com/test") {
            let _ = ServoCookie::from_cookie_string(cookie_str, &url, CookieSource::HTTP);
        }
    }
});
