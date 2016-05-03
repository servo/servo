/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;
use std::borrow::ToOwned;
use url::{Url, quirks};

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn SameOrigin(urlA: &Url, urlB: &Url) -> bool { urlA.origin() == urlB.origin() }
    pub fn Origin(url: &Url) -> USVString { USVString(quirks::origin(url)) }
    pub fn Href(url: &Url) -> USVString { USVString(quirks::href(url).to_owned()) }
    pub fn Hash(url: &Url) -> USVString { USVString(quirks::hash(url).to_owned()) }
    pub fn Host(url: &Url) -> USVString { USVString(quirks::host(url).to_owned()) }
    pub fn Port(url: &Url) -> USVString { USVString(quirks::port(url).to_owned()) }
    pub fn Search(url: &Url) -> USVString { USVString(quirks::search(url).to_owned()) }
    pub fn Hostname(url: &Url) -> USVString { USVString(quirks::hostname(url).to_owned()) }
    pub fn Password(url: &Url) -> USVString { USVString(quirks::password(url).to_owned()) }
    pub fn Pathname(url: &Url) -> USVString { USVString(quirks::pathname(url).to_owned()) }
    pub fn Protocol(url: &Url) -> USVString { USVString(quirks::protocol(url).to_owned()) }
    pub fn Username(url: &Url) -> USVString { USVString(quirks::username(url).to_owned()) }
    pub fn SetHash(url: &mut Url, value: USVString) { quirks::set_hash(url, &value.0) }
    pub fn SetHost(url: &mut Url, value: USVString) { let _ = quirks::set_host(url, &value.0); }
    pub fn SetPort(url: &mut Url, value: USVString) { let _ = quirks::set_port(url, &value.0); }
    pub fn SetSearch(url: &mut Url, value: USVString) { quirks::set_search(url, &value.0) }
    pub fn SetPathname(url: &mut Url, value: USVString) { quirks::set_pathname(url, &value.0) }
    pub fn SetHostname(url: &mut Url, value: USVString) { let _ = quirks::set_hostname(url, &value.0); }
    pub fn SetPassword(url: &mut Url, value: USVString) { let _ = quirks::set_password(url, &value.0); }
    pub fn SetProtocol(url: &mut Url, value: USVString) { let _ = quirks::set_protocol(url, &value.0); }
    pub fn SetUsername(url: &mut Url, value: USVString) { let _ = quirks::set_username(url, &value.0); }
}
