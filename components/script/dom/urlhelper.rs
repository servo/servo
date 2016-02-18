/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::str::USVString;
use std::borrow::ToOwned;
use url::{Url, ParseError, Host, Position};
use url::idna::domain_to_unicode;

#[derive(HeapSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    pub fn SameOrigin(urlA: &Url, urlB: &Url) -> bool { urlA.origin() == urlB.origin() }
    pub fn Origin(url: &Url) -> USVString { USVString(Api::origin(url)) }
    pub fn Href(url: &Url) -> USVString { USVString(Api::href(url).to_owned()) }
    pub fn Hash(url: &Url) -> USVString { USVString(Api::hash(url).to_owned()) }
    pub fn Host(url: &Url) -> USVString { USVString(Api::host(url).to_owned()) }
    pub fn Port(url: &Url) -> USVString { USVString(Api::port(url).to_owned()) }
    pub fn Search(url: &Url) -> USVString { USVString(Api::search(url).to_owned()) }
    pub fn Hostname(url: &Url) -> USVString { USVString(Api::hostname(url).to_owned()) }
    pub fn Password(url: &Url) -> USVString { USVString(Api::password(url).to_owned()) }
    pub fn Pathname(url: &Url) -> USVString { USVString(Api::pathname(url).to_owned()) }
    pub fn Protocol(url: &Url) -> USVString { USVString(Api::protocol(url).to_owned()) }
    pub fn Username(url: &Url) -> USVString { USVString(Api::username(url).to_owned()) }
    pub fn SetHash(url: &mut Url, value: USVString) { Api::set_hash(url, &value.0) }
    pub fn SetHost(url: &mut Url, value: USVString) { Api::set_host(url, &value.0) }
    pub fn SetPort(url: &mut Url, value: USVString) { Api::set_port(url, &value.0) }
    pub fn SetSearch(url: &mut Url, value: USVString) { Api::set_search(url, &value.0) }
    pub fn SetHostname(url: &mut Url, value: USVString) { Api::set_hostname(url, &value.0) }
    pub fn SetPassword(url: &mut Url, value: USVString) { Api::set_password(url, &value.0) }
    pub fn SetPathname(url: &mut Url, value: USVString) { Api::set_pathname(url, &value.0) }
    pub fn SetProtocol(url: &mut Url, value: USVString) { Api::set_protocol(url, &value.0) }
    pub fn SetUsername(url: &mut Url, value: USVString) { Api::set_username(url, &value.0) }
}

/// https://url.spec.whatwg.org/#api
pub struct Api;

impl Api {
    /// https://url.spec.whatwg.org/#dom-url-domaintoascii
    pub fn domain_to_ascii(domain: &str) -> String {
        match Host::parse(domain) {
            Ok(Host::Domain(domain)) => domain,
            _ => String::new(),
        }
    }

    /// https://url.spec.whatwg.org/#dom-url-domaintounicode
    pub fn domain_to_unicode(domain: &str) -> String {
        match Host::parse(domain) {
            Ok(Host::Domain(ref domain)) => {
                let (unicode, _errors) = domain_to_unicode(domain);
                unicode
            }
            _ => String::new(),
        }
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-href
    pub fn href(url: &Url) -> &str {
        url.as_str()
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-href
    pub fn set_href(url: &mut Url, value: &str) -> Result<(), ParseError> {
        *url = try!(Url::parse(value));
        Ok(())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-origin
    pub fn origin(url: &Url) -> String {
        url.origin().unicode_serialization()
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-protocol
    #[inline]
    pub fn protocol(url: &Url) -> &str {
        &url.as_str()[..url.scheme().len() + ":".len()]
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-protocol
    pub fn set_protocol(url: &mut Url, mut new_protocol: &str) {
        // The scheme state in the spec ignores everything after the first `:`,
        // but `set_scheme` errors if there is more.
        if let Some(position) = new_protocol.find(':') {
            new_protocol = &new_protocol[..position];
        }
        let _ = url.set_scheme(new_protocol);
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-username
    #[inline]
    pub fn username(url: &Url) -> &str {
        url.username()
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-username
    pub fn set_username(url: &mut Url, new_username: &str) {
        let _ = url.set_username(new_username);
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-password
    #[inline]
    pub fn password(url: &Url) -> &str {
        url.password().unwrap_or("")
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-password
    pub fn set_password(url: &mut Url, new_password: &str) {
        let _ = url.set_password(if new_password.is_empty() { None } else { Some(new_password) });
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-host
    #[inline]
    pub fn host(url: &Url) -> &str {
        &url[Position::BeforeHost..Position::AfterPort]
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-host
    pub fn set_host(url: &mut Url, new_host: &str) {
        url.quirky_set_host_and_port(new_host)
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-hostname
    #[inline]
    pub fn hostname(url: &Url) -> &str {
        url.host_str().unwrap_or("")
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-hostname
    pub fn set_hostname(url: &mut Url, new_hostname: &str) {
        url.quirky_set_host(new_hostname)
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-port
    #[inline]
    pub fn port(url: &Url) -> &str {
        &url[Position::BeforePort..Position::AfterPort]
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-port
    pub fn set_port(url: &mut Url, new_port: &str) {
        url.quirky_set_port(new_port)
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-pathname
    #[inline]
    pub fn pathname(url: &Url) -> &str {
         url.path()
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-pathname
    pub fn set_pathname(url: &mut Url, new_pathname: &str) {
        if !url.cannot_be_a_base() {
            url.set_path(new_pathname)
        }
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-search
    pub fn search(url: &Url) -> &str {
        trim(&url[Position::AfterPath..Position::AfterQuery])
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-search
    pub fn set_search(url: &mut Url, new_search: &str) {
        url.set_query(match new_search {
            "" => None,
            _ if new_search.starts_with('?') => Some(&new_search[1..]),
            _ => Some(new_search),
        })
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-searchparams
    pub fn search_params(url: &Url) -> Vec<(String, String)> {
        url.query_pairs().unwrap_or_else(Vec::new)
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-hash
    pub fn hash(url: &Url) -> &str {
        trim(&url[Position::AfterQuery..])
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-hash
    pub fn set_hash(url: &mut Url, new_hash: &str) {
        if url.scheme() != "javascript" {
            url.set_fragment(match new_hash {
                "" => None,
                _ if new_hash.starts_with('#') => Some(&new_hash[1..]),
                _ => Some(new_hash),
            })
        }
    }
}

fn trim(s: &str) -> &str {
    if s.len() == 1 {
        ""
    } else {
        s
    }
}
