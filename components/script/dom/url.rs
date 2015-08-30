/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::URLBinding::{self, URLMethods};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::str::USVString;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;

use url::{Host, ParseResult, Url, UrlParser};
use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::RefCell;

// https://url.spec.whatwg.org/#url
#[dom_struct]
pub struct URL {
    reflector_: Reflector,

    // https://url.spec.whatwg.org/#concept-urlutils-url
    url: RefCell<Url>,

    // https://url.spec.whatwg.org/#concept-urlutils-get-the-base
    base: Option<Url>,
}

impl URL {
    fn new_inherited(url: Url, base: Option<Url>) -> URL {
        URL {
            reflector_: Reflector::new(),
            url: RefCell::new(url),
            base: base,
        }
    }

    pub fn new(global: GlobalRef, url: Url, base: Option<Url>) -> Root<URL> {
        reflect_dom_object(box URL::new_inherited(url, base),
                           global, URLBinding::Wrap)
    }
}

impl URL {
    // https://url.spec.whatwg.org/#constructors
    pub fn Constructor(global: GlobalRef, url: USVString,
                       base: Option<USVString>)
                       -> Fallible<Root<URL>> {
        let parsed_base = match base {
            None => {
                // Step 1.
                None
            },
            Some(base) =>
                // Step 2.1.
                match Url::parse(&base.0) {
                    Ok(base) => Some(base),
                    Err(error) => {
                        // Step 2.2.
                        return Err(Error::Type(format!("could not parse base: {}", error)));
                    }
                }
        };
        // Step 3.
        let parsed_url = match parse_with_base(url, parsed_base.as_ref()) {
            Ok(url) => url,
            Err(error) => {
                // Step 4.
                return Err(Error::Type(format!("could not parse URL: {}", error)));
            }
        };
        // Steps 5-8.
        Ok(URL::new(global, parsed_url, parsed_base))
    }

    // https://url.spec.whatwg.org/#dom-url-domaintoasciidomain
    pub fn DomainToASCII(_: GlobalRef, origin: USVString) -> USVString {
        // Step 1.
        let ascii_domain = Host::parse(&origin.0);
        if let Ok(Host::Domain(string)) = ascii_domain {
            // Step 3.
            USVString(string.to_owned())
        } else {
            // Step 2.
            USVString("".to_owned())
        }
    }
}

impl URLMethods for URL {
    // https://url.spec.whatwg.org/#dom-urlutils-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hash
    fn SetHash(&self, value: USVString) {
        UrlHelper::SetHash(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-host
    fn SetHost(&self, value: USVString) {
        UrlHelper::SetHost(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hostname
    fn SetHostname(&self, value: USVString) {
        UrlHelper::SetHostname(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-href
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-href
    fn SetHref(&self, value: USVString) -> ErrorResult {
        match parse_with_base(value, self.base.as_ref()) {
            Ok(url) => {
                *self.url.borrow_mut() = url;
                Ok(())
            },
            Err(error) => {
                Err(Error::Type(format!("could not parse URL: {}", error)))
            },
        }
    }

    // https://url.spec.whatwg.org/#dom-urlutils-password
    fn Password(&self) -> USVString {
        UrlHelper::Password(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-password
    fn SetPassword(&self, value: USVString) {
        UrlHelper::SetPassword(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    fn SetPathname(&self, value: USVString) {
        UrlHelper::SetPathname(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-port
    fn SetPort(&self, value: USVString) {
        UrlHelper::SetPort(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-protocol
    fn SetProtocol(&self, value: USVString) {
        UrlHelper::SetProtocol(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    fn SetSearch(&self, value: USVString) {
        UrlHelper::SetSearch(&mut self.url.borrow_mut(), value);
    }

    // https://url.spec.whatwg.org/#URLUtils-stringification-behavior
    fn Stringifier(&self) -> DOMString {
        self.Href().0
    }

    // https://url.spec.whatwg.org/#dom-urlutils-username
    fn Username(&self) -> USVString {
        UrlHelper::Username(&self.url.borrow())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-username
    fn SetUsername(&self, value: USVString) {
        UrlHelper::SetUsername(&mut self.url.borrow_mut(), value);
    }
}

fn parse_with_base(input: USVString, base: Option<&Url>) -> ParseResult<Url> {
    let mut parser = UrlParser::new();
    if let Some(base) = base {
        parser.base_url(base);
    }
    parser.parse(&input.0)
}
