/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;

use servo_url::ServoUrl;
use url::quirks;

use crate::dom::bindings::str::USVString;

#[derive(MallocSizeOf)]
pub(crate) struct UrlHelper;

#[allow(non_snake_case)]
impl UrlHelper {
    pub(crate) fn Origin(url: &ServoUrl) -> USVString {
        USVString(quirks::origin(url.as_url()).to_owned())
    }
    pub(crate) fn Href(url: &ServoUrl) -> USVString {
        USVString(quirks::href(url.as_url()).to_owned())
    }
    pub(crate) fn Hash(url: &ServoUrl) -> USVString {
        USVString(quirks::hash(url.as_url()).to_owned())
    }
    pub(crate) fn Host(url: &ServoUrl) -> USVString {
        USVString(quirks::host(url.as_url()).to_owned())
    }
    pub(crate) fn Port(url: &ServoUrl) -> USVString {
        USVString(quirks::port(url.as_url()).to_owned())
    }
    pub(crate) fn Search(url: &ServoUrl) -> USVString {
        USVString(quirks::search(url.as_url()).to_owned())
    }
    pub(crate) fn Hostname(url: &ServoUrl) -> USVString {
        USVString(quirks::hostname(url.as_url()).to_owned())
    }
    pub(crate) fn Password(url: &ServoUrl) -> USVString {
        USVString(quirks::password(url.as_url()).to_owned())
    }
    pub(crate) fn Pathname(url: &ServoUrl) -> USVString {
        USVString(quirks::pathname(url.as_url()).to_owned())
    }
    pub(crate) fn Protocol(url: &ServoUrl) -> USVString {
        USVString(quirks::protocol(url.as_url()).to_owned())
    }
    pub(crate) fn Username(url: &ServoUrl) -> USVString {
        USVString(quirks::username(url.as_url()).to_owned())
    }
    pub(crate) fn SetHash(url: &mut ServoUrl, value: USVString) {
        quirks::set_hash(url.as_mut_url(), &value.0)
    }
    pub(crate) fn SetHost(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_host(url.as_mut_url(), &value.0);
    }
    pub(crate) fn SetPort(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_port(url.as_mut_url(), &value.0);
    }
    pub(crate) fn SetSearch(url: &mut ServoUrl, value: USVString) {
        quirks::set_search(url.as_mut_url(), &value.0)
    }
    pub(crate) fn SetPathname(url: &mut ServoUrl, value: USVString) {
        quirks::set_pathname(url.as_mut_url(), &value.0)
    }
    pub(crate) fn SetHostname(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_hostname(url.as_mut_url(), &value.0);
    }
    pub(crate) fn SetPassword(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_password(url.as_mut_url(), &value.0);
    }
    pub(crate) fn SetProtocol(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_protocol(url.as_mut_url(), &value.0);
    }
    pub(crate) fn SetUsername(url: &mut ServoUrl, value: USVString) {
        let _ = quirks::set_username(url.as_mut_url(), &value.0);
    }
}
