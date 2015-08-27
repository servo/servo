/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::URLBinding::{self, URLMethods};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::str::USVString;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;

use url::{Host, Url, UrlParser};
use util::str::DOMString;

use std::borrow::ToOwned;

// https://url.spec.whatwg.org/#url
#[dom_struct]
pub struct URL {
    reflector_: Reflector,

    // https://url.spec.whatwg.org/#concept-urlutils-url
    url: Url,
}

impl URL {
    fn new_inherited(url: Url) -> URL {
        URL {
            reflector_: Reflector::new(),
            url: url,
        }
    }

    pub fn new(global: GlobalRef, url: Url) -> Root<URL> {
        reflect_dom_object(box URL::new_inherited(url),
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
        let parsed_url = match parser_with_base(parsed_base.as_ref()).parse(&url.0) {
            Ok(url) => url,
            Err(error) => {
                // Step 4.
                return Err(Error::Type(format!("could not parse URL: {}", error)));
            }
        };
        // Steps 5-8.
        Ok(URL::new(global, parsed_url))
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

impl<'a> URLMethods for &'a URL {
    // https://url.spec.whatwg.org/#dom-urlutils-hash
    fn Hash(self) -> USVString {
        UrlHelper::Hash(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-host
    fn Host(self) -> USVString {
        UrlHelper::Host(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hostname
    fn Hostname(self) -> USVString {
        UrlHelper::Hostname(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-href
    fn Href(self) -> USVString {
        UrlHelper::Href(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-password
    fn Password(self) -> USVString {
        UrlHelper::Password(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    fn Pathname(self) -> USVString {
        UrlHelper::Pathname(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-port
    fn Port(self) -> USVString {
        UrlHelper::Port(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-protocol
    fn Protocol(self) -> USVString {
        UrlHelper::Protocol(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    fn Search(self) -> USVString {
        UrlHelper::Search(&self.url)
    }

    // https://url.spec.whatwg.org/#URLUtils-stringification-behavior
    fn Stringifier(self) -> DOMString {
        self.Href().0
    }

    // https://url.spec.whatwg.org/#dom-urlutils-username
    fn Username(self) -> USVString {
        UrlHelper::Username(&self.url)
    }
}

fn parser_with_base(base: Option<&Url>) -> UrlParser {
    let mut parser = UrlParser::new();
    if let Some(base) = base {
        parser.base_url(base);
    }
    parser
}
