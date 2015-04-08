/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;

use url::{Url, SchemeData};

use std::borrow::ToOwned;

pub struct UrlHelper;

impl UrlHelper {
    pub fn Href(url: &Url) -> USVString {
        USVString(url.serialize())
    }

    pub fn Protocol(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-protocol
        // FIXME: Url null check is skipped for now
        USVString(format!("{}:", url.scheme))
    }

    pub fn Username(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-username
        // FIXME: Url null check is skipped for now
        USVString(url.username().unwrap().to_owned())
    }

    pub fn Password(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-password
        USVString(match url.password() {
            None => "".to_owned(),
            Some(_) => url.password().unwrap().to_owned()
        })
    }

    pub fn Host(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-host
        // FIXME: Url null check is skipped for now
        let host = url.host().unwrap().serialize();
        USVString(match url.port() {
            None => host,
            Some(port) => format!("{}:{}", host, port)
        })
    }

    pub fn Hostname(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-hostname
        // FIXME: Url null check is skipped for now
        USVString(url.host().unwrap().serialize())
    }

    pub fn Port(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-port
        // FIXME: Url null check is skipped for now
        USVString(format!("{}", url.port().unwrap()))
    }

    pub fn Search(url: &Url) -> USVString {
        USVString(match url.query {
            None => "".to_owned(),
            Some(ref query) if query.as_slice() == "" => "".to_owned(),
            Some(ref query) => format!("?{}", query)
        })
    }

    pub fn Hash(url: &Url) -> USVString {
        USVString(match url.fragment {
            None => "".to_owned(),
            Some(ref hash) if hash.as_slice() == "" => "".to_owned(),
            Some(ref hash) => format!("#{}", hash)
        })
    }

    pub fn Pathname(url: &Url) -> USVString {
        // https://url.spec.whatwg.org/#dom-urlutils-pathname
        // FIXME: Url null check is skipped for now
        USVString(match url.scheme_data {
            SchemeData::NonRelative(ref scheme_data) => scheme_data.clone(),
            SchemeData::Relative(..) => url.serialize_path().unwrap()
        })
    }

    /// https://html.spec.whatwg.org/multipage/browsers.html#same-origin
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
