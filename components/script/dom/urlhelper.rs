/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::fmt::Write;
use url::urlutils::{UrlUtils, UrlUtilsWrapper};
use url::{SchemeData, Url, UrlParser};

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn Hash(url: &Url) -> String {
        match url.fragment {
            None => "".to_owned(),
            Some(ref hash) if hash.is_empty() => "".to_owned(),
            Some(ref hash) => format!("#{}", hash)
        }
    }

    pub fn SetHash(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_fragment(&value);
    }

    pub fn Host(url: &Url) -> String {
        match url.scheme_data {
            SchemeData::NonRelative(..) => "".to_owned(),
            SchemeData::Relative(ref scheme_data) => {
                let mut host = scheme_data.host.serialize();
                if let Some(port) = scheme_data.port {
                    write!(host, ":{}", port).unwrap();
                }
                host
            },
        }
    }

    pub fn SetHost(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_host(&value);
    }

    pub fn Hostname(url: &Url) -> String {
        url.serialize_host().unwrap_or_else(|| "".to_owned())
    }

    pub fn SetHostname(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_host_and_port(&value);
    }

    pub fn Href(url: &Url) -> String {
        url.serialize()
    }

    pub fn Password(url: &Url) -> String {
        url.password().unwrap_or("").to_owned()
    }

    pub fn SetPassword(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_password(&value);
    }

    pub fn Pathname(url: &Url) -> String {
        match url.scheme_data {
            SchemeData::NonRelative(ref scheme_data) => scheme_data.clone(),
            SchemeData::Relative(..) => url.serialize_path().unwrap()
        }
    }

    pub fn SetPathname(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_path(&value);
    }

    pub fn Port(url: &Url) -> String {
        match url.port() {
            None => "".to_owned(),
            Some(port) => port.to_string(),
        }
    }

    pub fn SetPort(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_port(&value);
    }

    pub fn Protocol(url: &Url) -> String {
        format!("{}:", url.scheme.clone())
    }

    pub fn SetProtocol(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_scheme(&value);
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
        true
    }

    pub fn Search(url: &Url) -> String {
        match url.query {
            None => "".to_owned(),
            Some(ref query) if query.is_empty() => "".to_owned(),
            Some(ref query) => format!("?{}", query)
        }
    }

    pub fn SetSearch(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_query(&value);
    }

    pub fn Username(url: &Url) -> String {
        url.username().unwrap_or("").to_owned()
    }

    pub fn SetUsername(url: &mut Url, value: String) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_username(&value);
    }
}
