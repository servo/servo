/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implementation of cookie storage as specified in
//! http://tools.ietf.org/html/rfc6265

use cookie::Cookie;
use cookie_rs;
use net_traits::CookieSource;
use net_traits::pub_domains::reg_suffix;
use servo_url::ServoUrl;
use std::cmp::Ordering;
use std::collections::HashMap;

extern crate time;

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct CookieStorage {
    version: u32,
    cookies_map: HashMap<String, Vec<Cookie>>,
    max_per_host: usize,
}

impl CookieStorage {
    pub fn new() -> CookieStorage {
        CookieStorage {
            version: 1,
            cookies_map: HashMap::new(),
            max_per_host: 50,
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn remove(&mut self, cookie: &Cookie, source: CookieSource) -> Result<Option<Cookie>, ()> {
        // Step 0
        let domain = reg_host(&cookie.cookie.domain.clone().unwrap_or("".to_owned())).unwrap_or("".to_owned());
        if self.cookies_map.contains_key(&domain) {
        let mut cookies = self.cookies_map.get_mut(&domain).unwrap();

        // Step 1
        let position = cookies.iter().position(|c| {
            c.cookie.domain == cookie.cookie.domain && c.cookie.path == cookie.cookie.path &&
            c.cookie.name == cookie.cookie.name
        });

        if let Some(ind) = position {
            let c = cookies.remove(ind);

            // http://tools.ietf.org/html/rfc6265#section-5.3 step 11.2
            if !c.cookie.httponly || source == CookieSource::HTTP {
                Ok(Some(c))
            } else {
                // Undo the removal.
                cookies.push(c);
                Err(())
            }
        } else {
            Ok(None)
        }
        }else {
            Ok(None)
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn push(&mut self, mut cookie: Cookie, source: CookieSource) {
        let old_cookie = self.remove(&cookie, source);
        if old_cookie.is_err() {
            // This new cookie is not allowed to overwrite an existing one.
            return;
        }

        // Step 11
        if let Some(old_cookie) = old_cookie.unwrap() {
            // Step 11.3
            cookie.creation_time = old_cookie.creation_time;
        }

        // Step 12
        let domain = reg_host(&cookie.cookie.domain.clone().unwrap_or("".to_owned())).unwrap_or("".to_owned());
        if !self.cookies_map.contains_key(&domain) {
        let tempdomain = domain.clone();
            let cookies: Vec<Cookie> = Vec::new();
            self.cookies_map.insert(tempdomain, cookies);
        }
        let mut cookies = self.cookies_map.get_mut(&domain).unwrap();

        if cookies.len() == self.max_per_host {
            // Step 12.1
            let old_len = cookies.len();
            cookies.retain(|c| !check_cookie_expired(&c));
            let new_len = cookies.len();

            // https://datatracker.ietf.org/doc/draft-ietf-httpbis-cookie-alone
            if new_len == old_len {
                // Remove non-secure cookie with oldest access time
                let mut is_first = true;
                let mut index = 0;
                let mut acc_time = time::now();
                for i in 0..cookies.len() {
                    let c = cookies.get(i).unwrap();
                    if !c.cookie.secure && (is_first || c.last_access < acc_time) {
                        acc_time = c.last_access;
                        index = i;

                        is_first = false;
                    }
                }

                // All secure cookies were found
                if is_first {
                        if !cookie.cookie.secure {
                            return;
                        } else {
                            let mut is_first = true;
                            let mut index = 0;
                            let mut acc_time = time::now();
                            // Get secure cookie with the oldest access time
                            for i in 0..cookies.len() {
                                let c = cookies.get(i).unwrap();
                                if c.cookie.secure && (is_first || c.last_access < acc_time) {
                                    acc_time = c.last_access;
                                    index = i;
                                    is_first = false;
                                }
                            }
                            cookies.remove(index);
                        }
                } else {
                    cookies.remove(index);
                }
            }
        }
        cookies.push(cookie);
    }

    pub fn cookie_comparator(a: &Cookie, b: &Cookie) -> Ordering {
        let a_path_len = a.cookie.path.as_ref().map_or(0, |p| p.len());
        let b_path_len = b.cookie.path.as_ref().map_or(0, |p| p.len());
        match a_path_len.cmp(&b_path_len) {
            Ordering::Equal => {
                let a_creation_time = a.creation_time.to_timespec();
                let b_creation_time = b.creation_time.to_timespec();
                a_creation_time.cmp(&b_creation_time)
            }
            // Ensure that longer paths are sorted earlier than shorter paths
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4
    pub fn cookies_for_url(&mut self, url: &ServoUrl, source: CookieSource) -> Option<String> {
        let filterer = |c: &&mut Cookie| -> bool {
            info!(" === SENT COOKIE : {} {} {:?} {:?}",
                  c.cookie.name,
                  c.cookie.value,
                  c.cookie.domain,
                  c.cookie.path);
            info!(" === SENT COOKIE RESULT {}",
                  c.appropriate_for_url(url, source));
            // Step 1
            c.appropriate_for_url(url, source)
        };

        // Step 2
        let domain = reg_host(&url.host_str().unwrap_or("").to_owned()).unwrap();
        let host = domain.clone();
        if !self.cookies_map.contains_key(&domain) {
            let cookies: Vec<Cookie> = Vec::new();
            self.cookies_map.insert(domain, cookies);
        }
        let mut cookies = self.cookies_map.get_mut(&host).unwrap();
        let mut url_cookies: Vec<&mut Cookie> = cookies.iter_mut().filter(filterer).collect();
        url_cookies.sort_by(|a, b| CookieStorage::cookie_comparator(*a, *b));

        let reducer = |acc: String, c: &mut &mut Cookie| -> String {
            // Step 3
            c.touch();

            // Step 4
            (match acc.len() {
                0 => acc,
                _ => acc + "; "
            }) + &c.cookie.name + "=" + &c.cookie.value
        };
        let result = url_cookies.iter_mut().fold("".to_owned(), reducer);

        info!(" === COOKIES SENT: {}", result);
        match result.len() {
            0 => None,
            _ => Some(result)
        }
    }

    pub fn cookies_data_for_url<'a>(&'a mut self,
                                    url: &'a ServoUrl,
                                    source: CookieSource)
                                    -> Box<Iterator<Item = cookie_rs::Cookie> + 'a> {
        let domain = reg_host(&url.host_str().unwrap_or("").to_owned()).unwrap();
        let host = domain.clone();
        if !self.cookies_map.contains_key(&domain) {
            let cookies: Vec<Cookie> = Vec::new();
            self.cookies_map.insert(domain, cookies);
        }
        let mut cookies = self.cookies_map.get_mut(&host).unwrap();

        Box::new(cookies.iter_mut().filter(move |c| c.appropriate_for_url(url, source)).map(|c| {
            c.touch();
            c.cookie.clone()
        }))
    }
}
    fn reg_host<'a>(url: &'a str) -> Option<String> {
       Some(reg_suffix(url).to_string())
    }

    fn check_cookie_expired(cookie: &Cookie) -> bool {
        if let Some(cookie_expiry_time_) = cookie.expiry_time {
            let cookie_expiry_time = cookie_expiry_time_.to_timespec();
            let cur_time = time::get_time();

            if cookie_expiry_time <= cur_time {
                return true;
            } else {
                return false;
            }

        } else {
            return false;
        }
    }
