/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::urlhelper::UrlHelper;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use net_traits::request::Referrer;
use script_traits::{HistoryEntryReplacement, LoadData, LoadOrigin};
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
        reflect_dom_object(Box::new(Location::new_inherited(window)), window)
    }

    /// https://html.spec.whatwg.org/multipage/#location-object-navigate
    fn navigate(
        &self,
        url: ServoUrl,
        referrer: Referrer,
        replacement_flag: HistoryEntryReplacement,
        reload_triggered: bool,
    ) {
        let document = self.window.Document();
        let referrer_policy = document.get_referrer_policy();
        let pipeline_id = self.window.upcast::<GlobalScope>().pipeline_id();
        let load_data = LoadData::new(
            LoadOrigin::Script(document.origin().immutable().clone()),
            url,
            Some(pipeline_id),
            referrer,
            referrer_policy,
            None, // Top navigation doesn't inherit secure context
        );
        // TODO: rethrow exceptions, set exceptions enabled flag.
        self.window
            .load_url(replacement_flag, reload_triggered, load_data);
    }

    fn get_url(&self) -> ServoUrl {
        self.window.get_url()
    }

    fn check_same_origin_domain(&self) -> ErrorResult {
        let this_document = self.window.Document();
        if self
            .entry_settings_object()
            .origin()
            .same_origin_domain(this_document.origin())
        {
            Ok(())
        } else {
            Err(Error::Security)
        }
    }

    fn entry_settings_object(&self) -> DomRoot<GlobalScope> {
        GlobalScope::entry()
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    pub fn reload_without_origin_check(&self) {
        let url = self.get_url();
        let referrer = Referrer::ReferrerUrl(url.clone());
        self.navigate(url, referrer, HistoryEntryReplacement::Enabled, true);
    }

    #[allow(dead_code)]
    pub fn origin(&self) -> &MutableOrigin {
        self.window.origin()
    }
}

impl LocationMethods for Location {
    // https://html.spec.whatwg.org/multipage/#dom-location-assign
    fn Assign(&self, url: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not same
            // origin-domain with the entry settings object's origin, then throw a
            // "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Parse url relative to the entry settings object. If that failed,
            // throw a "SyntaxError" DOMException.
            let base_url = self.entry_settings_object().api_base_url();
            let url = match base_url.join(&url.0) {
                Ok(url) => url,
                Err(_) => return Err(Error::Syntax),
            };
            // Step 4: Location-object navigate to the resulting URL record.
            let referrer = Referrer::ReferrerUrl(self.get_url());
            self.navigate(url, referrer, HistoryEntryReplacement::Disabled, false);
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-reload
    fn Reload(&self) -> ErrorResult {
        self.check_same_origin_domain()?;
        let url = self.get_url();
        let referrer = Referrer::ReferrerUrl(url.clone());
        self.navigate(url, referrer, HistoryEntryReplacement::Enabled, true);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-replace
    fn Replace(&self, url: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: Parse url relative to the entry settings object. If that failed,
            // throw a "SyntaxError" DOMException.
            let base_url = self.entry_settings_object().api_base_url();
            let url = match base_url.join(&url.0) {
                Ok(url) => url,
                Err(_) => return Err(Error::Syntax),
            };
            // Step 3: Location-object navigate to the resulting URL record with
            // the replacement flag set.
            let referrer = Referrer::ReferrerUrl(self.get_url());
            self.navigate(url, referrer, HistoryEntryReplacement::Enabled, false);
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn GetHash(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Hash(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-hash
    fn SetHash(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: Let input be the given value with a single leading "#" removed, if any.
            // Step 5: Set copyURL's fragment to the empty string.
            // Step 6: Basic URL parse input, with copyURL as url and fragment state as
            // state override.
            copy_url.as_mut_url().set_fragment(match value.0.as_str() {
                "" => Some("#"),
                _ if value.0.starts_with('#') => Some(&value.0[1..]),
                _ => Some(&value.0),
            });
            // Step 7: Location-object-setter navigate to copyURL.
            let referrer = Referrer::ReferrerUrl(self.get_url());
            self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn GetHost(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Host(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-host
    fn SetHost(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: If copyURL's cannot-be-a-base-URL flag is set, terminate these steps.
            if !copy_url.cannot_be_a_base() {
                // Step 5: Basic URL parse the given value, with copyURL as url and host state
                // as state override.
                let _ = copy_url.as_mut_url().set_host(Some(&value.0));
                // Step 6: Location-object-setter navigate to copyURL.
                let referrer = Referrer::ReferrerUrl(self.get_url());
                self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
            }
        }
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
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: If copyURL's cannot-be-a-base-URL flag is set, terminate these steps.
            if !copy_url.cannot_be_a_base() {
                // Step 5: Basic URL parse the given value, with copyURL as url and hostname
                // state as state override.
                let _ = copy_url.as_mut_url().set_host(Some(&value.0));
                // Step 6: Location-object-setter navigate to copyURL.
                let referrer = Referrer::ReferrerUrl(self.get_url());
                self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn GetHref(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Href(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-href
    fn SetHref(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Note: no call to self.check_same_origin_domain()
            // Step 2: Parse the given value relative to the entry settings object.
            // If that failed, throw a TypeError exception.
            let base_url = self.entry_settings_object().api_base_url();
            let url = match base_url.join(&value.0) {
                Ok(url) => url,
                Err(e) => return Err(Error::Type(format!("Couldn't parse URL: {}", e))),
            };
            // Step 3: Location-object-setter navigate to the resulting URL record.
            let referrer = Referrer::ReferrerUrl(self.get_url());
            self.navigate(url, referrer, HistoryEntryReplacement::Disabled, false);
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn GetPathname(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Pathname(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-pathname
    fn SetPathname(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: If copyURL's cannot-be-a-base-URL flag is set, terminate these steps.
            if !copy_url.cannot_be_a_base() {
                // Step 5: Set copyURL's path to the empty list.
                // Step 6: Basic URL parse the given value, with copyURL as url and path
                // start state as state override.
                copy_url.as_mut_url().set_path(&value.0);
                // Step 7: Location-object-setter navigate to copyURL.
                let referrer = Referrer::ReferrerUrl(self.get_url());
                self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn GetPort(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Port(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-port
    fn SetPort(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: If copyURL cannot have a username/password/port, then return.
            // https://url.spec.whatwg.org/#cannot-have-a-username-password-port
            if copy_url.host().is_some() &&
                !copy_url.cannot_be_a_base() &&
                copy_url.scheme() != "file"
            {
                // Step 5: If the given value is the empty string, then set copyURL's
                // port to null.
                // Step 6: Otherwise, basic URL parse the given value, with copyURL as url
                // and port state as state override.
                let _ = url::quirks::set_port(copy_url.as_mut_url(), &value.0);
                // Step 7: Location-object-setter navigate to copyURL.
                let referrer = Referrer::ReferrerUrl(self.get_url());
                self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn GetProtocol(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Protocol(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-protocol
    fn SetProtocol(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: Let possibleFailure be the result of basic URL parsing the given
            // value, followed by ":", with copyURL as url and scheme start state as
            // state override.
            let scheme = match value.0.find(':') {
                Some(position) => &value.0[..position],
                None => &value.0,
            };
            if let Err(_) = copy_url.as_mut_url().set_scheme(scheme) {
                // Step 5: If possibleFailure is failure, then throw a "SyntaxError" DOMException.
                return Err(Error::Syntax);
            }
            // Step 6: If copyURL's scheme is not an HTTP(S) scheme, then terminate these steps.
            if copy_url.scheme().eq_ignore_ascii_case("http") ||
                copy_url.scheme().eq_ignore_ascii_case("https")
            {
                // Step 7: Location-object-setter navigate to copyURL.
                let referrer = Referrer::ReferrerUrl(self.get_url());
                self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
            }
        }
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn GetSearch(&self) -> Fallible<USVString> {
        self.check_same_origin_domain()?;
        Ok(UrlHelper::Search(&self.get_url()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-location-search
    fn SetSearch(&self, value: USVString) -> ErrorResult {
        // Step 1: If this Location object's relevant Document is null, then return.
        if self.window.has_document() {
            // Step 2: If this Location object's relevant Document's origin is not
            // same origin-domain with the entry settings object's origin, then
            // throw a "SecurityError" DOMException.
            self.check_same_origin_domain()?;
            // Step 3: Let copyURL be a copy of this Location object's url.
            let mut copy_url = self.get_url();
            // Step 4: If the given value is the empty string, set copyURL's query to null.
            // Step 5: Otherwise, run these substeps:
            //   1. Let input be the given value with a single leading "?" removed, if any.
            //   2. Set copyURL's query to the empty string.
            //   3. Basic URL parse input, with copyURL as url and query state as state
            //      override, and the relevant Document's document's character encoding as
            //      encoding override.
            copy_url.as_mut_url().set_query(match value.0.as_str() {
                "" => None,
                _ if value.0.starts_with('?') => Some(&value.0[1..]),
                _ => Some(&value.0),
            });
            // Step 6: Location-object-setter navigate to copyURL.
            let referrer = Referrer::ReferrerUrl(self.get_url());
            self.navigate(copy_url, referrer, HistoryEntryReplacement::Disabled, false);
        }
        Ok(())
    }
}
