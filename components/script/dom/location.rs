/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::LocationBinding;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
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
        self.window.get_url()
    }

    fn set_url_component(&self, value: String,
                         setter: fn(&mut Url, String)) {
        let mut url = self.window.get_url();
        setter(&mut url, value);
        self.window.load_url(url);
    }
}

impl LocationMethods for Location {
    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, url: String) {
        // TODO: per spec, we should use the _API base URL_ specified by the
        //       _entry settings object_.
        let base_url = self.window.get_url();
        if let Ok(url) = UrlParser::new().base_url(&base_url).parse(&url) {
            self.window.load_url(url);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    fn Reload(&self) {
        self.window.load_url(self.get_url());
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn Hash(&self) -> String {
        UrlHelper::Hash(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn SetHash(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetHash);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn Host(&self) -> String {
        UrlHelper::Host(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn SetHost(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetHost);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hostname
    fn Hostname(&self) -> String {
        UrlHelper::Hostname(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hostname
    fn SetHostname(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetHostname);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Href(&self) -> String {
        UrlHelper::Href(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn SetHref(&self, value: String) {
        if let Ok(url) = UrlParser::new().base_url(&self.window.get_url()).parse(&value) {
            self.window.load_url(url);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn Pathname(&self) -> String {
        UrlHelper::Pathname(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn SetPathname(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetPathname);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn Port(&self) -> String {
        UrlHelper::Port(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn SetPort(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetPort);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn Protocol(&self) -> String {
        UrlHelper::Protocol(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn SetProtocol(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetProtocol);
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Stringifier(&self) -> DOMString {
        DOMString(self.Href())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn Search(&self) -> String {
        UrlHelper::Search(&self.get_url())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn SetSearch(&self, value: String) {
        self.set_url_component(value, UrlHelper::SetSearch);
    }
}
