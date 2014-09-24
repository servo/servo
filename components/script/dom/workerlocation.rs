/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerLocationBinding;
use dom::bindings::codegen::Bindings::WorkerLocationBinding::WorkerLocationMethods;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::global::Worker;
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::urlhelper::UrlHelper;
use dom::workerglobalscope::WorkerGlobalScope;

use servo_util::str::DOMString;

use url::Url;

#[jstraceable]
#[must_root]
pub struct WorkerLocation {
    reflector_: Reflector,
    url: Untraceable<Url>,
}

impl WorkerLocation {
    pub fn new_inherited(url: Url) -> WorkerLocation {
        WorkerLocation {
            reflector_: Reflector::new(),
            url: Untraceable::new(url),
        }
    }

    pub fn new(global: JSRef<WorkerGlobalScope>, url: Url) -> Temporary<WorkerLocation> {
        reflect_dom_object(box WorkerLocation::new_inherited(url),
                           &Worker(global),
                           WorkerLocationBinding::Wrap)
    }
}

impl<'a> WorkerLocationMethods for JSRef<'a, WorkerLocation> {
    fn Href(self) -> DOMString {
        UrlHelper::Href(self.url.deref())
    }

    fn Search(self) -> DOMString {
        UrlHelper::Search(self.url.deref())
    }

    fn Hash(self) -> DOMString {
        UrlHelper::Hash(self.url.deref())
    }
}

impl Reflectable for WorkerLocation {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
