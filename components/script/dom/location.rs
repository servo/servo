/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::LocationBinding;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::str::USVString;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;
use dom::window::Window;

use url::{Url, UrlParser};
use util::str::DOMString;

#[dom_struct]
pub struct Location {
    reflector_: Reflector,
    window: JS<Window>,
}

impl Location {
    fn new_inherited(window: &Window) -> Location {
        Location {
            reflector_: Reflector::new(),
            window: JS::from_ref(window)
        }
    }

    pub fn new(window: &Window) -> Root<Location> {
        reflect_dom_object(box Location::new_inherited(window),
                           GlobalRef::Window(window),
                           LocationBinding::Wrap)
    }

    fn get_url(&self) -> Url {
        self.window.root().get_url()
    }

    fn set_url_component(&self, value: USVString,
                         setter: fn(&mut Url, USVString)) {
        let window = self.window.root();
        let mut url = window.get_url();
        setter(&mut url, value);
        window.load_url(url);
    }
}

impl LocationMethods for Location {
    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, url: DOMString) {
        let window = self.window.root();
        // TODO: per spec, we should use the _API base URL_ specified by the
        //       _entry settings object_.
        let base_url = window.get_url();
        if let Ok(url) = UrlParser::new().base_url(&base_url).parse(&url) {
            window.load_url(url);
        }
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hash
    fn SetHash(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetHash);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-host
    fn SetHost(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetHost);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hostname
    fn SetHostname(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetHostname);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-href
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-href
    fn SetHref(&self, value: USVString) -> ErrorResult {
        let window = self.window.root();
        if let Ok(url) = UrlParser::new().base_url(&window.get_url()).parse(&value.0) {
            window.load_url(url);
        };
        Ok(())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-password
    fn Password(&self) -> USVString {
        UrlHelper::Password(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-password
    fn SetPassword(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetPassword);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    fn SetPathname(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetPathname);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-port
    fn SetPort(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetPort);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-protocol
    fn SetProtocol(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetProtocol);
    }

    // https://url.spec.whatwg.org/#URLUtils-stringification-behavior
    fn Stringifier(&self) -> DOMString {
        self.Href().0
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    fn SetSearch(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetSearch);
    }

    // https://url.spec.whatwg.org/#dom-urlutils-username
    fn Username(&self) -> USVString {
        UrlHelper::Username(&self.get_url())
    }

    // https://url.spec.whatwg.org/#dom-urlutils-username
    fn SetUsername(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetUsername);
    }
}
