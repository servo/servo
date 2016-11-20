/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::LocationBinding;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::urlhelper::UrlHelper;
use dom::window::Window;
use servo_url::ServoUrl;

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
                           window,
                           LocationBinding::Wrap)
    }

    fn get_url(&self) -> ServoUrl {
        self.window.get_url()
    }

    fn set_url_component(&self, value: USVString,
                         setter: fn(&mut ServoUrl, USVString)) {
        let mut url = self.window.get_url();
        setter(&mut url, value);
        self.window.load_url(url, false, false, None);
    }
}

impl LocationMethods for Location {
    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, url: USVString) -> ErrorResult {
        // TODO: per spec, we should use the _API base URL_ specified by the
        //       _entry settings object_.
        let base_url = self.window.get_url();
        if let Ok(url) = base_url.join(&url.0) {
            self.window.load_url(url, false, false, None);
            Ok(())
        } else {
            Err(Error::Syntax)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    fn Reload(&self) {
        self.window.load_url(self.get_url(), true, true, None);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-replace
    fn Replace(&self, url: USVString) -> ErrorResult {
        // TODO: per spec, we should use the _API base URL_ specified by the
        //       _entry settings object_.
        let base_url = self.window.get_url();
        if let Ok(url) = base_url.join(&url.0) {
            self.window.load_url(url, true, false, None);
            Ok(())
        } else {
            Err(Error::Syntax)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn SetHash(&self, mut value: USVString) {
        if value.0.is_empty() {
            value = USVString("#".to_owned());
        }
        self.set_url_component(value, UrlHelper::SetHash);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn SetHost(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetHost);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-origin
    fn Origin(&self) -> USVString {
        UrlHelper::Origin(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hostname
    fn SetHostname(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetHostname);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn SetHref(&self, value: USVString) {
        if let Ok(url) = self.window.get_url().join(&value.0) {
            self.window.load_url(url, false, false, None);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn SetPathname(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetPathname);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn SetPort(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetPort);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn SetProtocol(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetProtocol);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Stringifier(&self) -> DOMString {
        DOMString::from(self.Href().0)
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn SetSearch(&self, value: USVString) {
        self.set_url_component(value, UrlHelper::SetSearch);
    }
}
