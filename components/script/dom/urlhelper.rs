/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use url::quirks;

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn SameOrigin(url_a: &ServoUrl, url_b: &ServoUrl) -> bool {
        url_a.origin() == url_b.origin()
    }
    pub fn Origin(url: &ServoUrl) -> USVString {
        USVString(quirks::origin(url.as_url().unwrap()).to_owned())
    }
    pub fn Href(url: &ServoUrl) -> USVString {
        USVString(quirks::href(url.as_url().unwrap()).to_owned())
    }
    pub fn Hash(url: &ServoUrl) -> USVString {
        USVString(quirks::hash(url.as_url().unwrap()).to_owned())
    }
    pub fn Host(url: &ServoUrl) -> USVString {
        USVString(quirks::host(url.as_url().unwrap()).to_owned())
    }
    pub fn Port(url: &ServoUrl) -> USVString {
        USVString(quirks::port(url.as_url().unwrap()).to_owned())
    }
    pub fn Search(url: &ServoUrl) -> USVString {
        USVString(quirks::search(url.as_url().unwrap()).to_owned())
    }
    pub fn Hostname(url: &ServoUrl) -> USVString {
        USVString(quirks::hostname(url.as_url().unwrap()).to_owned())
    }
    pub fn Password(url: &ServoUrl) -> USVString {
        USVString(quirks::password(url.as_url().unwrap()).to_owned())
    }
    pub fn Pathname(url: &ServoUrl) -> USVString {
        USVString(quirks::pathname(url.as_url().unwrap()).to_owned())
    }
    pub fn Protocol(url: &ServoUrl) -> USVString {
        USVString(quirks::protocol(url.as_url().unwrap()).to_owned())
    }
    pub fn Username(url: &ServoUrl) -> USVString {
        USVString(quirks::username(url.as_url().unwrap()).to_owned())
    }
    pub fn SetHash(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            quirks::set_hash(url, &value.0)
        }
    }
    pub fn SetHost(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            let _ = quirks::set_host(url, &value.0);
        }
    }
    pub fn SetPort(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            let _ = quirks::set_port(url, &value.0);
        }
    }
    pub fn SetSearch(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            quirks::set_search(url, &value.0)
        }
    }
    pub fn SetPathname(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            quirks::set_pathname(url, &value.0)
        }
    }
    pub fn SetHostname(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            let _ = quirks::set_hostname(url, &value.0);
        }
    }
    pub fn SetPassword(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            let _ = quirks::set_password(url, &value.0);
        }
    }
    pub fn SetProtocol(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            let _ = quirks::set_protocol(url, &value.0);
        }
    }
    pub fn SetUsername(url: &mut ServoUrl, value: USVString) {
        if let Some(ref mut url) = url.as_mut_url() {
            let _ = quirks::set_username(url, &value.0);
        }
    }
}
