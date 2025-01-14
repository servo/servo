/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::WorkerLocationBinding::WorkerLocationMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::USVString;
use crate::dom::urlhelper::UrlHelper;
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::script_runtime::CanGc;

// https://html.spec.whatwg.org/multipage/#worker-locations
#[dom_struct]
pub(crate) struct WorkerLocation {
    reflector_: Reflector,
    #[no_trace]
    url: ServoUrl,
}

impl WorkerLocation {
    fn new_inherited(url: ServoUrl) -> WorkerLocation {
        WorkerLocation {
            reflector_: Reflector::new(),
            url,
        }
    }

    pub(crate) fn new(global: &WorkerGlobalScope, url: ServoUrl) -> DomRoot<WorkerLocation> {
        reflect_dom_object(
            Box::new(WorkerLocation::new_inherited(url)),
            global,
            CanGc::note(),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-origin
    #[allow(dead_code)]
    pub(crate) fn origin(&self) -> ImmutableOrigin {
        self.url.origin()
    }
}

impl WorkerLocationMethods<crate::DomTypeHolder> for WorkerLocation {
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

    // https://html.spec.whatwg.org/multipage/#dom-workerlocation-origin
    fn Origin(&self) -> USVString {
        UrlHelper::Origin(&self.url)
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
}
