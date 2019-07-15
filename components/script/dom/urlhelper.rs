/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::USVString;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use url::Position;

#[derive(MallocSizeOf)]
pub struct UrlHelper;

impl UrlHelper {
    /// Getter for https://url.spec.whatwg.org/#dom-url-origin
    pub fn Origin(url: &ServoUrl) -> USVString {
        USVString(url.as_url().origin().ascii_serialization().to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-href
    pub fn Href(url: &ServoUrl) -> USVString {
        USVString(url.as_url().as_str().to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-hash
    pub fn Hash(url: &ServoUrl) -> USVString {
        USVString(trim(&url.as_url()[Position::AfterQuery..]).to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-host
    pub fn Host(url: &ServoUrl) -> USVString {
        USVString(url.as_url()[Position::BeforeHost..Position::AfterPort].to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-port
    pub fn Port(url: &ServoUrl) -> USVString {
        USVString(url.as_url()[Position::BeforePort..Position::AfterPort].to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-search
    pub fn Search(url: &ServoUrl) -> USVString {
        USVString(trim(&url.as_url()[Position::AfterPath..Position::AfterQuery]).to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-hostname
    pub fn Hostname(url: &ServoUrl) -> USVString {
        USVString(url.as_url().host_str().unwrap_or("").to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-password
    pub fn Password(url: &ServoUrl) -> USVString {
        USVString(url.as_url().password().unwrap_or("").to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-pathname
    pub fn Pathname(url: &ServoUrl) -> USVString {
        USVString(url.as_url().path().to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-protocol
    pub fn Protocol(url: &ServoUrl) -> USVString {
        let url = url.as_url();
        USVString(url.as_str()[..url.scheme().len() + ":".len()].to_owned())
    }

    /// Getter for https://url.spec.whatwg.org/#dom-url-username
    pub fn Username(url: &ServoUrl) -> USVString {
        USVString(url.as_url().username().to_owned())
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-hash
    pub fn SetHash(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();
        if url.scheme() != "javascript" {
            url.set_fragment(if value.is_empty() {
                None
            } else if value.starts_with('#') {
                Some(&value[1..])
            } else {
                Some(&value)
            })
        }
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-host
    pub fn SetHost(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();
        if url.cannot_be_a_base() {
            return;
        }
        let host;
        let opt_port;
        {
            let scheme = url.scheme();
            let result = Parser::parse_host(Input::new(new_host), SchemeType::from(scheme));
            match result {
                Ok((h, remaining)) => {
                    host = h;
                    opt_port = if let Some(remaining) = remaining.split_prefix(':') {
                        Parser::parse_port(remaining, || default_port(scheme), Context::Setter)
                            .ok()
                            .map(|(port, _remaining)| port)
                    } else {
                        None
                    };
                },
                Err(_) => return,
            }
        }
        url.set_host_internal(host, opt_port);
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-port
    pub fn SetPort(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();

        // has_host implies !cannot_be_a_base
        let scheme = url.scheme();
        if !url.has_host() || scheme == "file" {
            return;
        }
        let result =
            Parser::parse_port(Input::new(&value), || default_port(scheme), Context::Setter);
        if let Ok((new_port, _remaining)) = result {
            url.set_port_internal(new_port)
        }
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-search
    pub fn SetSearch(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();
        url.set_query(if value.is_empty() {
            None
        } else if value.starts_with('?') {
            Some(&value[1..])
        } else {
            Some(&value)
        })
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-pathname
    pub fn SetPathname(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();
        if !url.cannot_be_a_base() {
            url.set_path(&value)
        }
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-hostname
    pub fn SetHostname(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();
        if url.cannot_be_a_base() {
            return;
        }
        let result = Parser::parse_host(Input::new(&value), SchemeType::from(url.scheme()));
        if let Ok((host, _remaining)) = result {
            url.set_host_internal(host, None)
        }
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-password
    pub fn SetPassword(url: &mut ServoUrl, value: USVString) {
        let _ = url.as_mut_url().set_password(if value.is_empty() { None } else { Some(&value) });
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-protocol
    pub fn SetProtocol(url: &mut ServoUrl, value: USVString) {
        let url = url.as_mut_url();
        // The scheme state in the spec ignores everything after the first `:`,
        // but `set_scheme` errors if there is more.
        let _ = url.set_scheme(if let Some(position) = value.find(':') {
            &value[..position]
        } else {
            &value
        });
    }

    /// Setter for https://url.spec.whatwg.org/#dom-url-username
    pub fn SetUsername(url: &mut ServoUrl, value: USVString) {
        let _ = url.as_mut_url().set_username(&value);
    }

    /// https://w3c.github.io/webappsec-secure-contexts/#is-origin-trustworthy
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

fn trim(s: &str) -> &str {
    if s.len() == 1 {
        ""
    } else {
        s
    }
}
