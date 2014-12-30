/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of cookie storage as specified in
//! http://tools.ietf.org/html/rfc6265

use url::Url;
use cookie::Cookie;

pub struct CookieStorage {
    cookies: Vec<Cookie>
}

impl CookieStorage {
    pub fn new() -> CookieStorage {
        CookieStorage {
            cookies: Vec::new()
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn remove(&mut self, cookie: &Cookie) -> Option<Cookie> {
        // Step 1
        let position = self.cookies.iter().position(|c| {
            c.cookie.domain == cookie.cookie.domain &&
            c.cookie.path == cookie.cookie.path &&
            c.cookie.name == cookie.cookie.name
        });

        if let Some(ind) = position {
            Some(self.cookies.remove(ind))
        } else {
            None
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn push(&mut self, mut cookie: Cookie, request: &Url) {
        // Step 11
        if let Some(old_cookie) = self.remove(&cookie) {
            // Step 11.2
            if old_cookie.cookie.httponly && !request.scheme.starts_with("http") {
                self.cookies.push(old_cookie);
            } else {
                // Step 11.3
                cookie.created_at = old_cookie.created_at;
                // Step 12
                self.cookies.push(cookie);
            }
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4
    pub fn cookies_for_url(&mut self, url: Url) -> Option<String> {
        let filterer = |&:c: &&mut Cookie| -> bool {
            info!(" === SENT COOKIE : {} {} {:?} {:?}", c.cookie.name, c.cookie.value, c.cookie.domain, c.cookie.path);
            info!(" === SENT COOKIE RESULT {}", c.appropriate_for_url(url.clone()));
            // Step 1
            c.appropriate_for_url(url.clone())
        };

        let mut url_cookies = self.cookies.iter_mut().filter(filterer);

        // TODO Step 2

        let reducer = |&:acc: String, c: &mut Cookie| -> String {
            // Step 3
            c.touch();

            // Step 4
            (match acc.len() {
                0 => acc,
                _ => acc + ";"
            }) + c.cookie.name.as_slice() + "=" + c.cookie.value.as_slice()
        };
        let result = url_cookies.fold("".to_string(), reducer);

        info!(" === COOKIES SENT: {}", result);
        match result.len() {
            0 => None,
            _ => Some(result)
        }
    }
}
