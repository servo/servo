/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;

use url::{Url, SchemeData};

use std::borrow::ToOwned;

pub struct UrlHelper;

impl UrlHelper {
    // https://url.spec.whatwg.org/#dom-urlutils-href
    pub fn Href(url: &Url) -> USVString {
        USVString(url.serialize())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    pub fn Search(url: &Url) -> USVString {
        USVString(match url.query {
            None => "".to_owned(),
            Some(ref query) if query.is_empty() => "".to_owned(),
            Some(ref query) => format!("?{}", query)
        })
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hash
    pub fn Hash(url: &Url) -> USVString {
        USVString(match url.fragment {
            None => "".to_owned(),
            Some(ref hash) if hash.is_empty() => "".to_owned(),
            Some(ref hash) => format!("#{}", hash)
        })
    }

    pub fn Protocol(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-protocol
        // FIXME: Url null check is skipped for now
        USVString(format!("{}:", url.scheme))
    }

    pub fn Username(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-username
        // FIXME: Url null check is skipped for now
        USVString(match url.username() {
            None => "".to_owned(),
            Some(username) => username.to_owned()
        })
    }

    pub fn Password(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-password
        USVString(match url.password() {
            None => "".to_owned(),
            Some(password) => password.to_owned()
        })
    }

    pub fn Host(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-host
        // FIXME: Url null check is skipped for now
        let host = match url.host() {
            None => ""
            Some(host) => host
        };
        USVString(match url.port() {
            None => host.to_owned(),
            Some(port) => format!("{}:{}", host, port)
        })
    }

    pub fn Hostname(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-hostname
        // FIXME: Url null check is skipped for now
        USVString(match url.host() {
            None => "".to_owned(),
            Some(host) => host.serialize()
        })
    }
    pub fn Port(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-port
        // FIXME: Url null check is skipped for now
        USVString(match url.port() {
            None => "".to_owned(),
            Some(port) => format!("{}", port)
        })
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    pub fn Pathname(url: &Url) -> USVString {
        // FIXME: Url null check is skipped for now
        USVString(match url.scheme_data {
            SchemeData::NonRelative(ref scheme_data) => scheme_data.clone(),
            SchemeData::Relative(..) => url.serialize_path().unwrap()
        })
    }

    // https://html.spec.whatwg.org/multipage/#same-origin
    pub fn SameOrigin(urlA: &Url, urlB: &Url) -> bool {
        if urlA.host() != urlB.host() {
            return false
        }
        if urlA.scheme != urlB.scheme {
            return false
        }
        if urlA.port() != urlB.port() {
            return false
        }
        return true
    }
}
