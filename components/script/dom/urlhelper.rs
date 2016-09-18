/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;
use std::borrow::ToOwned;
use url::{Url, quirks};

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn same_origin(url_a: &Url, url_b: &Url) -> bool { url_a.origin() == url_b.origin() }
    pub fn origin(url: &Url) -> USVString { USVString(quirks::origin(url)) }
    pub fn href(url: &Url) -> USVString { USVString(quirks::href(url).to_owned()) }
    pub fn hash(url: &Url) -> USVString { USVString(quirks::hash(url).to_owned()) }
    pub fn host(url: &Url) -> USVString { USVString(quirks::host(url).to_owned()) }
    pub fn port(url: &Url) -> USVString { USVString(quirks::port(url).to_owned()) }
    pub fn search(url: &Url) -> USVString { USVString(quirks::search(url).to_owned()) }
    pub fn hostname(url: &Url) -> USVString { USVString(quirks::hostname(url).to_owned()) }
    pub fn password(url: &Url) -> USVString { USVString(quirks::password(url).to_owned()) }
    pub fn pathname(url: &Url) -> USVString { USVString(quirks::pathname(url).to_owned()) }
    pub fn protocol(url: &Url) -> USVString { USVString(quirks::protocol(url).to_owned()) }
    pub fn username(url: &Url) -> USVString { USVString(quirks::username(url).to_owned()) }
    pub fn set_hash(url: &mut Url, value: USVString) { quirks::set_hash(url, &value.0) }
    pub fn set_host(url: &mut Url, value: USVString) { let _ = quirks::set_host(url, &value.0); }
    pub fn set_port(url: &mut Url, value: USVString) { let _ = quirks::set_port(url, &value.0); }
    pub fn set_search(url: &mut Url, value: USVString) { quirks::set_search(url, &value.0) }
    pub fn set_pathname(url: &mut Url, value: USVString) { quirks::set_pathname(url, &value.0) }
    pub fn set_hostname(url: &mut Url, value: USVString) { let _ = quirks::set_hostname(url, &value.0); }
    pub fn set_password(url: &mut Url, value: USVString) { let _ = quirks::set_password(url, &value.0); }
    pub fn set_protocol(url: &mut Url, value: USVString) { let _ = quirks::set_protocol(url, &value.0); }
    pub fn set_username(url: &mut Url, value: USVString) { let _ = quirks::set_username(url, &value.0); }
}
