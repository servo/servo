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
    pub fn Origin(url: &ServoUrl) -> USVString {
        USVString(quirks::origin(url.as_url()).to_owned())
    }
    pub fn Href(url: &ServoUrl) -> USVString {
        USVString(quirks::href(url.as_url()).to_owned())
    }
    pub fn Hash(url: &ServoUrl) -> USVString {
        USVString(quirks::hash(url.as_url()).to_owned())
    }
    pub fn Host(url: &ServoUrl) -> USVString {
        USVString(quirks::host(url.as_url()).to_owned())
    }
    pub fn Port(url: &ServoUrl) -> USVString {
        USVString(quirks::port(url.as_url()).to_owned())
    }
    pub fn Search(url: &ServoUrl) -> USVString {
        USVString(quirks::search(url.as_url()).to_owned())
    }
    pub fn Hostname(url: &ServoUrl) -> USVString {
        USVString(quirks::hostname(url.as_url()).to_owned())
    }
    pub fn Password(url: &ServoUrl) -> USVString {
        USVString(quirks::password(url.as_url()).to_owned())
    }
    pub fn Pathname(url: &ServoUrl) -> USVString {
        USVString(quirks::pathname(url.as_url()).to_owned())
    }
    pub fn Protocol(url: &ServoUrl) -> USVString {
        USVString(quirks::protocol(url.as_url()).to_owned())
    }
    pub fn Username(url: &ServoUrl) -> USVString {
        USVString(quirks::username(url.as_url()).to_owned())
    }
    pub fn SetHash(url: &mut ServoUrl, value: USVString) {
        quirks::set_hash(url.as_mut_url(), &value.0)
    }
    pub fn SetHost(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_host(url.as_mut_url(), &value.0);
    }
    pub fn SetPort(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_port(url.as_mut_url(), &value.0);
    }
    pub fn SetSearch(url: &mut ServoUrl, value: USVString) {
        quirks::set_search(url.as_mut_url(), &value.0)
    }
    pub fn SetPathname(url: &mut ServoUrl, value: USVString) {
        quirks::set_pathname(url.as_mut_url(), &value.0)
    }
    pub fn SetHostname(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_hostname(url.as_mut_url(), &value.0);
    }
    pub fn SetPassword(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_password(url.as_mut_url(), &value.0);
    }
    pub fn SetProtocol(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_protocol(url.as_mut_url(), &value.0);
    }
    pub fn SetUsername(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_username(url.as_mut_url(), &value.0);
    }
    // https://w3c.github.io/webappsec-secure-contexts/#is-origin-trustworthy
    pub fn is_origin_trustworthy(url: &ServoUrl) -> bool {
        // Step 3
        if url.scheme() == "http" || url.scheme() == "wss" {
            true
        // Step 4
        } else if url.host().is_some() {
            let host = url.host_str().unwrap();
            host == "127.0.0.0/8" || host == "::1/128"
        // Step 5
        } else {
            url.scheme() == "file"
        }
    }
}
