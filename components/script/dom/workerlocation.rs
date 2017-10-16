/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerLocationBinding;
use dom::bindings::codegen::Bindings::WorkerLocationBinding::WorkerLocationMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::{DOMString, USVString};
use dom::urlhelper::UrlHelper;
use dom::workerglobalscope::WorkerGlobalScope;
use dom_struct::dom_struct;
use servo_url::ServoUrl;

// https://html.spec.whatwg.org/multipage/#worker-locations
#[dom_struct]
pub struct WorkerLocation {
    reflector_: Reflector,
    url: ServoUrl,
}

impl WorkerLocation {
    fn new_inherited(url: ServoUrl) -> WorkerLocation {
        WorkerLocation {
            reflector_: Reflector::new(),
            url: url,
        }
    }

    pub fn new(global: &WorkerGlobalScope, url: ServoUrl) -> DomRoot<WorkerLocation> {
        reflect_dom_object(Box::new(WorkerLocation::new_inherited(url)),
                           global,
                           WorkerLocationBinding::Wrap)
    }
}

impl WorkerLocationMethods for WorkerLocation {
    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-href
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-href
    fn Stringifier(&self) -> DOMString {
        DOMString::from(self.Href().0)
    }
}
