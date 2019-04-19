/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::LocationBinding;
use crate::dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::globalscope::GlobalScope;
use crate::dom::urlhelper::UrlHelper;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use net_traits::request::Referrer;
use servo_url::{MutableOrigin, ServoUrl};

#[dom_struct]
pub struct Location {
    reflector_: Reflector,
    window: Dom<Window>,
}

impl Location {
    fn new_inherited(window: &Window) -> Location {
        Location {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub fn new(window: &Window) -> DomRoot<Location> {
        reflect_dom_object(
            Box::new(Location::new_inherited(window)),
            window,
            LocationBinding::Wrap,
        )
    }

    fn get_url(&self) -> ServoUrl {
        self.window.get_url()
    }

    fn set_url_component(&self, value: USVString, setter: fn(&mut ServoUrl, USVString)) {
        let mut url = self.window.get_url();
        let referrer = Referrer::ReferrerUrl(url.clone());
        setter(&mut url, value);
        self.window.load_url(url, false, false, referrer, None);
    }

    fn check_same_origin_domain(&self) -> ErrorResult {
        let entry_document = GlobalScope::entry().as_window().Document();
        let this_document = self.window.Document();
        if entry_document
            .origin()
            .same_origin_domain(this_document.origin())
        {
            Ok(())
        } else {
            Err(Error::Security)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    pub fn reload_without_origin_check(&self) {
        let url = self.get_url();
        let referrer = Referrer::ReferrerUrl(url.clone());
        self.window.load_url(url, true, true, referrer, None);
    }

    #[allow(dead_code)]
    pub fn origin(&self) -> &MutableOrigin {
        self.window.origin()
    }
}

impl LocationMethods for Location {
    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, url: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        // TODO: per spec, we should use the _API base URL_ specified by the
        //       _entry settings object_.
        let base_url = self.window.get_url();
        if let Ok(url) = base_url.join(&url.0) {
            let referrer = Referrer::ReferrerUrl(url.clone());
            self.window.load_url(url, false, false, referrer, None);
            Ok(())
        } else {
            Err(Error::Syntax)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    fn Reload(&self) -> ErrorResult {
        self.check_same_origin_domain()?;
        let url = self.get_url();
        let referrer = Referrer::ReferrerUrl(url.clone());
        self.window.load_url(url, true, true, referrer, None);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-replace
    fn Replace(&self, url: USVString) -> ErrorResult {
        // Note: no call to self.check_same_origin_domain()
        // TODO: per spec, we should use the _API base URL_ specified by the
        //       _entry settings object_.
        let base_url = self.window.get_url();
        if let Ok(url) = base_url.join(&url.0) {
            let referrer = Referrer::ReferrerUrl(url.clone());
            self.window.load_url(url, true, false, referrer, None);
            Ok(())
        } else {
            Err(Error::Syntax)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn GetHash(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Hash(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn SetHash(&self, mut value: USVString) -> ErrorResult {
        if value.0.is_empty() {
            value = USVString("#".to_owned());
        }
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetHash);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn GetHost(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Host(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn SetHost(&self, value: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetHost);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-origin
    fn GetOrigin(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Origin(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hostname
    fn GetHostname(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Hostname(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hostname
    fn SetHostname(&self, value: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetHostname);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn GetHref(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Href(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn SetHref(&self, value: USVString) -> ErrorResult {
        // Note: no call to self.check_same_origin_domain()
        let url = match self.window.get_url().join(&value.0) {
            Ok(url) => url,
            Err(e) => return Err(Error::Type(format!("Couldn't parse URL: {}", e))),
        };
        let referrer = Referrer::ReferrerUrl(url.clone());
        self.window.load_url(url, false, false, referrer, None);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn GetPathname(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Pathname(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn SetPathname(&self, value: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetPathname);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn GetPort(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Port(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn SetPort(&self, value: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetPort);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn GetProtocol(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Protocol(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn SetProtocol(&self, value: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetProtocol);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn Stringifier(&self) -> Fallible<DOMString> {
        Ok(DOMString::from(self.GetHref()?.0))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn GetSearch(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Search(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn SetSearch(&self, value: USVString) -> ErrorResult {
        self.check_same_origin_domain()?;
        self.set_url_component(value, UrlHelper::SetSearch);
        Ok(())
    }
}
