/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of cookie storage as specified in
//! <http://tools.ietf.org/html/rfc6265>

use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::net::IpAddr;
use std::time::SystemTime;

use cookie::Cookie;
use hyper_serde::Serde;
use itertools::Itertools;
use log::{info, warn};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::pub_domains::reg_suffix;
use net_traits::{
    CookieAsyncResponse, CookieChange, CookieChangeType, CookieData, CookieSource, SiteDescriptor,
};
use serde::{Deserialize, Serialize};
use servo_base::generic_channel::GenericCallback;
use servo_base::id::CookieStoreId;
use servo_url::ServoUrl;

use crate::cookie::ServoCookie;

#[derive(Clone, Debug, Deserialize, Serialize, MallocSizeOf)]
pub struct CookieStorage {
    version: u32,
    cookies_map: HashMap<String, Vec<ServoCookie>>,
    max_per_host: usize,
    #[serde(skip)]
    #[ignore_malloc_size_of = "Channels"]
    listeners: HashMap<CookieStoreId, (ServoUrl, GenericCallback<CookieAsyncResponse>)>,
}

struct ServoCookieChange {
    cookie: ServoCookie,
    type_: CookieChangeType,
}

#[derive(Debug)]
pub enum RemoveCookieError {
    Overlapping,
    NonHTTP,
}

impl CookieStorage {
    pub fn new(max_cookies: usize) -> CookieStorage {
        CookieStorage {
            version: 1,
            cookies_map: HashMap::new(),
            max_per_host: max_cookies,
            listeners: HashMap::new(),
        }
    }

    pub fn add_listener(
        &mut self,
        store_id: CookieStoreId,
        url: ServoUrl,
        callback: GenericCallback<CookieAsyncResponse>,
    ) {
        self.listeners.insert(store_id, (url, callback));
    }

    pub fn remove_listener(&mut self, store_id: CookieStoreId) {
        self.listeners.remove(&store_id);
    }

    pub fn send_response(&self, store_id: CookieStoreId, data: CookieData) {
        let Some((_, callback)) = self.listeners.get(&store_id) else {
            warn!("Async cookie request made for store id that is non-existent {store_id:?}");
            return;
        };
        if callback.send(CookieAsyncResponse { data }).is_err() {
            warn!("Unable to send cookie response to script thread");
        }
    }

    /// <https://cookiestore.spec.whatwg.org/#process-cookie-changes>
    fn process_cookie_changes(&self, changes: &[ServoCookieChange]) {
        // 1. For every Window window, run the following steps:
        for (url, callback) in self.listeners.values() {
            // 1.1. Let url be window’s relevant settings object’s creation URL.
            // 1.2. Let changes be the observable changes for url.
            let changes = observable_changes(url, changes);

            // 1.3. If changes is empty, then continue.
            if changes.is_empty() {
                continue;
            }

            // 1.4. Queue a global task on the DOM manipulation task source given window to fire a
            // change event named "change" with changes at window’s CookieStore.
            //
            // This is implemented in CookieListener::handle.
            if callback
                .send(CookieAsyncResponse {
                    data: CookieData::Change(changes),
                })
                .is_err()
            {
                warn!("Unable to send cookie change to script thread");
            }
        }

        // TODO: 2. For every service worker registration registration, fire a functional event
        // named "cookiechange" for the changes matching its cookie change subscription list.
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn remove(
        &mut self,
        cookie: &ServoCookie,
        url: &ServoUrl,
        source: CookieSource,
    ) -> Result<Option<ServoCookie>, RemoveCookieError> {
        let domain = reg_host(cookie.cookie.domain().as_ref().unwrap_or(&""));
        let cookies = self.cookies_map.entry(domain).or_default();

        // https://www.ietf.org/id/draft-ietf-httpbis-cookie-alone-01.txt Step 2
        if !cookie.cookie.secure().unwrap_or(false) && !url.is_secure_scheme() {
            let new_domain = cookie.cookie.domain().as_ref().unwrap().to_owned();
            let new_path = cookie.cookie.path().as_ref().unwrap().to_owned();

            let any_overlapping = cookies.iter().any(|c| {
                let existing_domain = c.cookie.domain().as_ref().unwrap().to_owned();
                let existing_path = c.cookie.path().as_ref().unwrap().to_owned();

                c.cookie.name() == cookie.cookie.name() &&
                    c.cookie.secure().unwrap_or(false) &&
                    (ServoCookie::domain_match(new_domain, existing_domain) ||
                        ServoCookie::domain_match(existing_domain, new_domain)) &&
                    ServoCookie::path_match(new_path, existing_path)
            });

            if any_overlapping {
                return Err(RemoveCookieError::Overlapping);
            }
        }

        // Step 11.1
        let position = cookies.iter().position(|c| {
            c.cookie.domain() == cookie.cookie.domain() &&
                c.cookie.path() == cookie.cookie.path() &&
                c.cookie.name() == cookie.cookie.name()
        });

        if let Some(ind) = position {
            // Step 11.4
            let c = cookies.remove(ind);

            // http://tools.ietf.org/html/rfc6265#section-5.3 step 11.2
            if c.cookie.http_only().unwrap_or(false) && source == CookieSource::NonHTTP {
                // Undo the removal.
                cookies.push(c);
                Err(RemoveCookieError::NonHTTP)
            } else {
                Ok(Some(c))
            }
        } else {
            Ok(None)
        }
    }

    pub fn delete_cookies_for_sites(&mut self, sites: &Vec<String>) {
        // Note: We assume the number of sites is smaller than the number of
        // entries in the cookies map. If this assumption stops holding in
        // practice, this implementation can be revised to use `retain`
        // together with a temporary `HashSet` of sites.
        for site in sites {
            // TODO: We currently mark cookies as expired instead of removing
            // them immediately (same behavior as in the functions below).
            // This is safe because higher-level cookie accessors always call
            // `remove_expired_cookies_for_url` / `remove_all_expired_cookies`.
            // Consider whether we should instead delete the entries directly.
            if let Some(cookies) = self.cookies_map.get_mut(site) {
                for cookie in cookies.iter_mut() {
                    cookie.set_expiry_time_in_past();
                }
            }
        }
    }

    pub fn clear_session_cookies(&mut self) {
        self.cookies_map
            .values_mut()
            .flat_map(|cookies| cookies.iter_mut())
            .filter(|cookie| !cookie.persistent)
            .for_each(|cookie| cookie.set_expiry_time_in_past());
    }

    pub fn clear_storage(&mut self, url: Option<&ServoUrl>) {
        if let Some(url) = url {
            let domain = reg_host(url.host_str().unwrap_or(""));
            if let Some(cookies) = self.cookies_map.get_mut(&domain) {
                for cookie in cookies.iter_mut() {
                    cookie.set_expiry_time_in_past();
                }
            }
        } else {
            self.cookies_map.clear();
        }
    }

    pub fn delete_cookie_with_name(&mut self, url: &ServoUrl, name: String) {
        let domain = reg_host(url.host_str().unwrap_or(""));
        if let Some(cookies) = self.cookies_map.get_mut(&domain) {
            for cookie in cookies.iter_mut() {
                if cookie.cookie.name() == name {
                    cookie.set_expiry_time_in_past();
                }
            }
        }
    }

    // http://tools.ietf.org/html/rfc6265#section-5.3
    pub fn push(&mut self, mut cookie: ServoCookie, url: &ServoUrl, source: CookieSource) {
        // https://www.ietf.org/id/draft-ietf-httpbis-cookie-alone-01.txt Step 1
        if cookie.cookie.secure().unwrap_or(false) && !url.is_secure_scheme() {
            return;
        }

        let Ok(old_cookie) = self.remove(&cookie, url, source) else {
            // This new cookie is not allowed to overwrite an existing one.
            return;
        };

        // Step 11
        if let Some(old_cookie) = &old_cookie {
            // Step 11.3
            cookie.creation_time = old_cookie.creation_time;
        }

        // https://cookiestore.spec.whatwg.org/#cookie-change
        // > A newly-created cookie which is immediately evicted is considered deleted.
        let mut changes = vec![];
        if is_cookie_expired(&cookie) {
            cookie.immediately_evicted = true;
            // NOTE: We do not process a change if the immediately evicted cookie does not overwrite
            // an existing cookie, as that cookie would not be observable by the browser. This
            // mirrors the behaviour of other browsers and WPT expectations.
            if old_cookie.is_some() {
                changes.push(ServoCookieChange {
                    cookie: cookie.clone(),
                    type_: CookieChangeType::Deleted,
                });
            }
        }

        // Step 12
        let domain = reg_host(cookie.cookie.domain().as_ref().unwrap_or(&""));
        let cookies = self.cookies_map.entry(domain).or_default();

        if cookies.len() == self.max_per_host {
            let old_len = cookies.len();
            changes.extend(evict_expired_cookies(cookies));

            // https://www.ietf.org/id/draft-ietf-httpbis-cookie-alone-01.txt
            if cookies.len() == old_len {
                let Some(evicted) =
                    evict_one_cookie(cookie.cookie.secure().unwrap_or(false), cookies)
                else {
                    return;
                };

                // https://cookiestore.spec.whatwg.org/#cookie-change
                // > A cookie which is otherwise evicted or removed is considered deleted
                changes.push(ServoCookieChange {
                    cookie: evicted,
                    type_: CookieChangeType::Deleted,
                });
            }
        }

        // https://cookiestore.spec.whatwg.org/#cookie-change
        // > A newly-created cookie which is not immediately evicted is considered changed.
        // NOTE: We do not process a change if the new cookie is identical to the existing one, as
        // that cookie would not be observable by the browser. This mirrors the behaviour of other
        // browsers and WPT expectations.
        let is_identical = old_cookie.is_some_and(|old_cookie| {
            old_cookie.cookie == cookie.cookie &&
                old_cookie.cookie.same_site() == cookie.cookie.same_site() &&
                old_cookie.expiry_time == cookie.expiry_time
        });
        if !cookie.immediately_evicted && !is_identical {
            changes.push(ServoCookieChange {
                cookie: cookie.clone(),
                type_: CookieChangeType::Changed,
            });
        }

        cookies.push(cookie);

        // https://cookiestore.spec.whatwg.org/#ref-for-process-cookie-changes
        // > When any of the following conditions occur for a cookie store, perform the steps to
        // > process cookie changes.
        // > - A newly-created cookie is inserted into the cookie store.
        // > - A user agent evicts expired cookies from the cookie store.
        // > - A user agent removes excess cookies from the cookie store.
        self.process_cookie_changes(&changes);
    }

    pub fn cookie_comparator(a: &ServoCookie, b: &ServoCookie) -> Ordering {
        let a_path_len = a.cookie.path().as_ref().map_or(0, |p| p.len());
        let b_path_len = b.cookie.path().as_ref().map_or(0, |p| p.len());
        match a_path_len.cmp(&b_path_len) {
            Ordering::Equal => a.creation_time.cmp(&b.creation_time),
            // Ensure that longer paths are sorted earlier than shorter paths
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }

    pub fn remove_expired_cookies_for_url(&mut self, url: &ServoUrl) {
        let domain = reg_host(url.host_str().unwrap_or(""));
        if let Entry::Occupied(mut entry) = self.cookies_map.entry(domain) {
            let cookies = entry.get_mut();
            let expired = evict_expired_cookies(cookies);
            if cookies.is_empty() {
                entry.remove_entry();
            }

            // https://cookiestore.spec.whatwg.org/#ref-for-process-cookie-changes
            // > When any of the following conditions occur for a cookie store, perform the steps to
            // > process cookie changes.
            // > - A user agent evicts expired cookies from the cookie store.
            self.process_cookie_changes(&expired);
        }
    }

    pub fn remove_all_expired_cookies(&mut self) {
        let expired: Vec<_> = self
            .cookies_map
            .values_mut()
            .flat_map(evict_expired_cookies)
            .collect();
        self.cookies_map.retain(|_, cookies| !cookies.is_empty());

        // https://cookiestore.spec.whatwg.org/#ref-for-process-cookie-changes
        // > When any of the following conditions occur for a cookie store, perform the steps to
        // > process cookie changes.
        // > - A user agent evicts expired cookies from the cookie store.
        self.process_cookie_changes(&expired);
    }

    // http://tools.ietf.org/html/rfc6265#section-5.4
    pub fn cookies_for_url(&mut self, url: &ServoUrl, source: CookieSource) -> Option<String> {
        // Let cookie-list be the set of cookies from the cookie store
        let cookie_list = self.cookies_data_for_url(url, source);

        let reducer = |acc: String, cookie: Cookie<'static>| -> String {
            // Serialize the cookie-list into a cookie-string by processing each cookie in the cookie-list in order:
            // If the cookies' name is not empty, output the cookie's name followed by the %x3D ("=") character.
            // If the cookies' value is not empty, output the cookie's value.
            // If there is an unprocessed cookie in the cookie-list, output the characters %x3B and %x20 ("; ").
            // Security: the above steps allow for "nameless" cookies which have proved to be a security footgun
            // especially with the new cookie name prefix proposals
            (match acc.len() {
                0 => acc,
                _ => acc + "; ",
            }) + cookie.name() +
                "=" +
                cookie.value()
        };

        // Serialize the cookie-list into a cookie-string by processing each cookie in the cookie-list in order
        let result = cookie_list.fold(String::new(), reducer);

        info!(" === COOKIES SENT: {}", result);
        match result.len() {
            0 => None,
            _ => Some(result),
        }
    }

    /// <https://cookiestore.spec.whatwg.org/#query-cookies>
    pub fn query_cookies(&mut self, url: &ServoUrl, name: Option<String>) -> Vec<Cookie<'static>> {
        // 1. Retrieve cookie-list given request-uri and "non-HTTP" source
        let cookie_list = self.cookies_data_for_url(url, CookieSource::NonHTTP);

        // 3. For each cookie in cookie-list, run these steps:
        // 3.2. If name is given, then run these steps:
        if let Some(name) = name {
            // Let cookieName be the result of running UTF-8 decode without BOM on cookie’s name.
            // If cookieName does not equal name, then continue.
            cookie_list.filter(|cookie| cookie.name() == name).collect()
        } else {
            cookie_list.collect()
        }

        // Note: we do not convert the list into CookieListItem's here, we do that in script to not not have to define
        // the binding types in net.

        // Return list
    }

    pub fn cookies_data_for_url<'a>(
        &'a mut self,
        url: &'a ServoUrl,
        source: CookieSource,
    ) -> impl Iterator<Item = cookie::Cookie<'static>> + 'a {
        let domain = reg_host(url.host_str().unwrap_or(""));
        let cookies = self.cookies_map.entry(domain).or_default();

        cookies
            .iter_mut()
            .filter(move |c| c.appropriate_for_url(url, source))
            .sorted_by(|a: &&mut ServoCookie, b: &&mut ServoCookie| {
                // The user agent SHOULD sort the cookie-list
                CookieStorage::cookie_comparator(a, b)
            })
            .map(|c| {
                // Update the last-access-time of each cookie in the cookie-list to the current date and time
                c.touch();
                c.cookie.clone()
            })
    }

    pub fn cookie_site_descriptors(&self) -> Vec<SiteDescriptor> {
        self.cookies_map
            .keys()
            .cloned()
            .map(SiteDescriptor::new)
            .collect()
    }
}

/// <https://cookiestore.spec.whatwg.org/#observable-changes>
fn observable_changes(url: &ServoUrl, changes: &[ServoCookieChange]) -> Vec<CookieChange> {
    // > The observable changes for url are the set of cookie changes to cookies in a cookie store
    // > which meet the requirements in step 1 of Cookies § Retrieval Algorithm’s steps to compute
    // > the "cookie-string from a given cookie store" with url as request-uri, for a "non-HTTP" API.
    changes
        .iter()
        .filter(|change| {
            change
                .cookie
                .appropriate_for_url(url, CookieSource::NonHTTP)
        })
        .map(|change| CookieChange {
            cookie: Serde(change.cookie.cookie.clone()),
            type_: change.type_.clone(),
        })
        .collect()
}

fn reg_host(url: &str) -> String {
    let host_for_ip_parse = url
        .strip_prefix('[')
        .and_then(|url| url.strip_suffix(']'))
        .unwrap_or(url);
    if let Ok(address) = host_for_ip_parse.parse::<IpAddr>() {
        debug_assert!(address.to_string() == address.to_string().to_lowercase());
        return address.to_string();
    }

    reg_suffix(url).to_lowercase()
}

fn is_cookie_expired(cookie: &ServoCookie) -> bool {
    matches!(cookie.expiry_time, Some(date_time) if date_time <= SystemTime::now())
}

fn evict_one_cookie(is_secure_cookie: bool, cookies: &mut Vec<ServoCookie>) -> Option<ServoCookie> {
    // Remove non-secure cookie with oldest access time
    if let Some((index, _)) = get_oldest_accessed(false, cookies) {
        return Some(cookies.remove(index));
    }

    // All secure cookies were found
    if !is_secure_cookie {
        return None;
    }
    let (index, _) = get_oldest_accessed(true, cookies)?;
    Some(cookies.remove(index))
}

fn evict_expired_cookies(cookies: &mut Vec<ServoCookie>) -> Vec<ServoCookieChange> {
    cookies
        .extract_if(.., |c| is_cookie_expired(c))
        // https://cookiestore.spec.whatwg.org/#cookie-change
        // > A cookie which is otherwise evicted or removed is considered deleted.
        // NOTE: An immediately evicted cookie has already been processed on insertion.
        .filter(|c| !c.immediately_evicted)
        .map(|cookie| ServoCookieChange {
            cookie,
            type_: CookieChangeType::Deleted,
        })
        .collect()
}

fn get_oldest_accessed(
    is_secure_cookie: bool,
    cookies: &mut [ServoCookie],
) -> Option<(usize, SystemTime)> {
    let mut oldest_accessed = None;
    for (i, c) in cookies.iter().enumerate() {
        if (c.cookie.secure().unwrap_or(false) == is_secure_cookie) &&
            oldest_accessed
                .as_ref()
                .is_none_or(|(_, current_oldest_time)| c.last_access < *current_oldest_time)
        {
            oldest_accessed = Some((i, c.last_access));
        }
    }
    oldest_accessed
}
