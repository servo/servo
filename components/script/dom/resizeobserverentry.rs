/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsval::JSVal;

use crate::dom::bindings::codegen::Bindings::ResizeObserverEntryBinding::ResizeObserverEntryMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::domrectreadonly::DOMRectReadOnly;
use crate::dom::element::Element;
use crate::script_runtime::JSContext as SafeJSContext;

#[dom_struct]
pub struct ResizeObserverEntry {
    reflector_: Reflector,
}

impl ResizeObserverEntryMethods for ResizeObserverEntry {
    fn Target(&self) -> DomRoot<Element> {
        unimplemented!()
    }
    fn ContentRect(&self) -> DomRoot<DOMRectReadOnly> {
        unimplemented!()
    }
    fn BorderBoxSize(&self, cx: SafeJSContext) -> JSVal {
        unimplemented!()
    }
    fn ContentBoxSize(&self, cx: SafeJSContext) -> JSVal {
        unimplemented!()
    }
    fn DevicePixelContentBoxSize(&self, cx: SafeJSContext) -> JSVal {
        unimplemented!()
    }
}
