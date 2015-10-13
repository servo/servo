/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;
use std::borrow::ToOwned;
use std::fmt::Write;
use url::urlutils::{UrlUtils, UrlUtilsWrapper};
use url::{SchemeData, Url, UrlParser};

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn Hash(url: &Url) -> USVString {
        USVString(match url.fragment {
            None => "".to_owned(),
            Some(ref hash) if hash.is_empty() => "".to_owned(),
            Some(ref hash) => format!("#{}", hash)
        })
    }

    pub fn SetHash(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_fragment(&value.0);
    }

    pub fn Host(url: &Url) -> USVString {
        USVString(match url.scheme_data {
            SchemeData::NonRelative(..) => "".to_owned(),
            SchemeData::Relative(ref scheme_data) => {
                let mut host = scheme_data.host.serialize();
                if let Some(port) = scheme_data.port {
                    write!(host, ":{}", port).unwrap();
                }
                host
            },
        })
    }

    pub fn SetHost(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_host(&value.0);
    }

    pub fn Hostname(url: &Url) -> USVString {
        USVString(url.serialize_host().unwrap_or_else(|| "".to_owned()))
    }

    pub fn SetHostname(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_host_and_port(&value.0);
    }

    pub fn Href(url: &Url) -> USVString {
        USVString(url.serialize())
    }

    pub fn Password(url: &Url) -> USVString {
        USVString(url.password().unwrap_or("").to_owned())
    }

    pub fn SetPassword(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_password(&value.0);
    }

    pub fn Pathname(url: &Url) -> USVString {
        USVString(match url.scheme_data {
            SchemeData::NonRelative(ref scheme_data) => scheme_data.clone(),
            SchemeData::Relative(..) => url.serialize_path().unwrap()
        })
    }

    pub fn SetPathname(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_path(&value.0);
    }

    pub fn Port(url: &Url) -> USVString {
        USVString(match url.port() {
            None => "".to_owned(),
            Some(port) => port.to_string(),
        })
    }

    pub fn SetPort(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_port(&value.0);
    }

    pub fn Protocol(url: &Url) -> USVString {
        USVString(format!("{}:", url.scheme.clone()))
    }

    pub fn SetProtocol(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_scheme(&value.0);
    }

    // g
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

    pub fn Search(url: &Url) -> USVString {
        USVString(match url.query {
            None => "".to_owned(),
            Some(ref query) if query.is_empty() => "".to_owned(),
            Some(ref query) => format!("?{}", query)
        })
    }

    pub fn SetSearch(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_query(&value.0);
    }

    pub fn Username(url: &Url) -> USVString {
        USVString(url.username().unwrap_or("").to_owned())
    }

    pub fn SetUsername(url: &mut Url, value: USVString) {
        let mut wrapper = UrlUtilsWrapper { url: url, parser: &UrlParser::new() };
        let _ = wrapper.set_username(&value.0);
    }
}
