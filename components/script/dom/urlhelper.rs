/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;
use std::borrow::ToOwned;
use url::{Url, WebIdl};

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn SameOrigin(urlA: &Url, urlB: &Url) -> bool { urlA.origin() == urlB.origin() }
    pub fn Origin(url: &Url) -> USVString { USVString(WebIdl::origin(url).to_owned()) }
    pub fn Href(url: &Url) -> USVString { USVString(WebIdl::href(url).to_owned()) }
    pub fn Hash(url: &Url) -> USVString { USVString(WebIdl::hash(url).to_owned()) }
    pub fn Host(url: &Url) -> USVString { USVString(WebIdl::host(url).to_owned()) }
    pub fn Port(url: &Url) -> USVString { USVString(WebIdl::port(url).to_owned()) }
    pub fn Search(url: &Url) -> USVString { USVString(WebIdl::search(url).to_owned()) }
    pub fn Hostname(url: &Url) -> USVString { USVString(WebIdl::hostname(url).to_owned()) }
    pub fn Password(url: &Url) -> USVString { USVString(WebIdl::password(url).to_owned()) }
    pub fn Pathname(url: &Url) -> USVString { USVString(WebIdl::pathname(url).to_owned()) }
    pub fn Protocol(url: &Url) -> USVString { USVString(WebIdl::protocol(url).to_owned()) }
    pub fn Username(url: &Url) -> USVString { USVString(WebIdl::username(url).to_owned()) }
    pub fn SetHash(url: &mut Url, value: USVString) { WebIdl::set_hash(url, &value.0) }
    pub fn SetHost(url: &mut Url, value: USVString) { WebIdl::set_host(url, &value.0) }
    pub fn SetPort(url: &mut Url, value: USVString) { WebIdl::set_port(url, &value.0) }
    pub fn SetSearch(url: &mut Url, value: USVString) { WebIdl::set_search(url, &value.0) }
    pub fn SetHostname(url: &mut Url, value: USVString) { WebIdl::set_hostname(url, &value.0) }
    pub fn SetPassword(url: &mut Url, value: USVString) { WebIdl::set_password(url, &value.0) }
    pub fn SetPathname(url: &mut Url, value: USVString) { WebIdl::set_pathname(url, &value.0) }
    pub fn SetProtocol(url: &mut Url, value: USVString) { WebIdl::set_protocol(url, &value.0) }
    pub fn SetUsername(url: &mut Url, value: USVString) { WebIdl::set_username(url, &value.0) }
}
