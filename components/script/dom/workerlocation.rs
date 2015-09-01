/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerLocationBinding;
use dom::bindings::codegen::Bindings::WorkerLocationBinding::WorkerLocationMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::str::USVString;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;
use dom::workerglobalscope::WorkerGlobalScope;

use url::Url;
use util::str::DOMString;

// https://html.spec.whatwg.org/multipage/#worker-locations
#[dom_struct]
pub struct WorkerLocation {
    reflector_: Reflector,
    url: Url,
}

impl WorkerLocation {
    fn new_inherited(url: Url) -> WorkerLocation {
        WorkerLocation {
            reflector_: Reflector::new(),
            url: url,
        }
    }

    pub fn new(global: &WorkerGlobalScope, url: Url) -> Root<WorkerLocation> {
        reflect_dom_object(box WorkerLocation::new_inherited(url),
                           GlobalRef::Worker(global),
                           WorkerLocationBinding::Wrap)
    }
}

impl WorkerLocationMethods for WorkerLocation {
    // https://url.spec.whatwg.org/#dom-urlutils-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-href
    fn Href(&self) -> USVString {
        UrlHelper::Href(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.url)
    }

    // https://url.spec.whatwg.org/#dom-urlutils-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.url)
    }

    // https://url.spec.whatwg.org/#URLUtils-stringification-behavior
    fn Stringifier(&self) -> DOMString {
        self.Href().0
    }
}
