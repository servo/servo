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
use time::Tm;

extern crate time;

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct CookieStorage {
    version: u32,
    cookies_map: HashMap<String, Vec<Cookie>>,
    max_per_host: usize,
}

impl CookieStorage {
    pub fn new(max_cookies: usize) -> CookieStorage {
        CookieStorage {
            version: 1,
            cookies_map: HashMap::new(),
            max_per_host: max_cookies,
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn remove(&mut self, cookie: &Cookie, url: &ServoUrl, source: CookieSource) -> Result<Option<Cookie>, ()> {
        let domain = reg_host(cookie.cookie.domain.as_ref().unwrap_or(&"".to_string()));
        let cookies = self.cookies_map.entry(domain).or_insert(vec![]);

        // https://www.ietf.org/id/draft-ietf-httpbis-cookie-alone-01.txt Step 2
        if !cookie.cookie.secure && !url.is_secure_scheme() {
            let new_domain = cookie.cookie.domain.as_ref().unwrap();
            let new_path = cookie.cookie.path.as_ref().unwrap();

            let any_overlapping = cookies.iter().any(|c| {
                let existing_domain = c.cookie.domain.as_ref().unwrap();
                let existing_path = c.cookie.path.as_ref().unwrap();

                c.cookie.name == cookie.cookie.name &&
                c.cookie.secure &&
                (Cookie::domain_match(new_domain, existing_domain) ||
                 Cookie::domain_match(existing_domain, new_domain)) &&
                Cookie::path_match(new_path, existing_path)
            });

            if any_overlapping {
                return Err(());
            }
        }

        // Step 11.1
        let position = cookies.iter().position(|c| {
            c.cookie.domain == cookie.cookie.domain &&
            c.cookie.path == cookie.cookie.path &&
            c.cookie.name == cookie.cookie.name
        });

        if let Some(ind) = position {
            // Step 11.4
            let c = cookies.remove(ind);

            // http://tools.ietf.org/html/rfc6265#section-5.3 step 11.2
            if c.cookie.httponly && source == CookieSource::NonHTTP {
                // Undo the removal.
                cookies.push(c);
                Err(())
            } else {
                Ok(Some(c))
            }
        } else {
            Ok(None)
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn push(&mut self, mut cookie: Cookie, url: &ServoUrl, source: CookieSource) {
        // https://www.ietf.org/id/draft-ietf-httpbis-cookie-alone-01.txt Step 1
        if cookie.cookie.secure && !url.is_secure_scheme() {
            return;
        }

        let old_cookie = self.remove(&cookie, url, source);
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
        let domain = reg_host(&cookie.cookie.domain.as_ref().unwrap_or(&"".to_string()));
        let mut cookies = self.cookies_map.entry(domain).or_insert(vec![]);

        if cookies.len() == self.max_per_host {
            let old_len = cookies.len();
            cookies.retain(|c| !is_cookie_expired(&c));
            let new_len = cookies.len();

            // https://www.ietf.org/id/draft-ietf-httpbis-cookie-alone-01.txt
            if new_len == old_len && !evict_one_cookie(cookie.cookie.secure, cookies) {
                return;
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
        let domain = reg_host(url.host_str().unwrap_or(""));
        let cookies = self.cookies_map.entry(domain).or_insert(vec![]);

        let mut url_cookies: Vec<&mut Cookie> = cookies.iter_mut().filter(filterer).collect();
        url_cookies.sort_by(|a, b| CookieStorage::cookie_comparator(*a, *b));

        let reducer = |acc: String, c: &mut &mut Cookie| -> String {
            // Step 3
            c.touch();

            // Step 4
            (match acc.len() {
                0 => acc,
                _ => acc + "; ",
            }) + &c.cookie.name + "=" + &c.cookie.value
        };
        let result = url_cookies.iter_mut().fold("".to_owned(), reducer);

        info!(" === COOKIES SENT: {}", result);
        match result.len() {
            0 => None,
            _ => Some(result),
        }
    }

    pub fn cookies_data_for_url<'a>(&'a mut self,
                                    url: &'a ServoUrl,
                                    source: CookieSource)
                                    -> Box<Iterator<Item = cookie_rs::Cookie> + 'a> {
        let domain = reg_host(url.host_str().unwrap_or(""));
        let cookies = self.cookies_map.entry(domain).or_insert(vec![]);

        Box::new(cookies.iter_mut().filter(move |c| c.appropriate_for_url(url, source)).map(|c| {
            c.touch();
            c.cookie.clone()
        }))
    }
}

fn reg_host<'a>(url: &'a str) -> String {
    reg_suffix(url).to_string()
}

fn is_cookie_expired(cookie: &Cookie) -> bool {
    match cookie.expiry_time {
        Some(t) => t.to_timespec() <= time::get_time(),
        None => false,
    }
}

fn evict_one_cookie(is_secure_cookie: bool, cookies: &mut Vec<Cookie>) -> bool {
    // Remove non-secure cookie with oldest access time
    let oldest_accessed: Option<(usize, Tm)> = get_oldest_accessed(false, cookies);

    if let Some((index, _)) = oldest_accessed {
        cookies.remove(index);
    } else {
        // All secure cookies were found
        if !is_secure_cookie {
            return false;
        }
        let oldest_accessed: Option<(usize, Tm)> = get_oldest_accessed(true, cookies);
        if let Some((index, _)) = oldest_accessed {
            cookies.remove(index);
        }
    }
    return true;
}

fn get_oldest_accessed(is_secure_cookie: bool, cookies: &mut Vec<Cookie>) -> Option<(usize, Tm)> {
    let mut oldest_accessed: Option<(usize, Tm)> = None;
    for (i, c) in cookies.iter().enumerate() {
        if (c.cookie.secure == is_secure_cookie) &&
           oldest_accessed.as_ref().map_or(true, |a| c.last_access < a.1) {
            oldest_accessed = Some((i, c.last_access));
        }
    }
    oldest_accessed
}
