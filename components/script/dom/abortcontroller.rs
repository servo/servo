/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::Value;
use js::rust::{Handle, HandleObject};

use crate::dom::bindings::codegen::Bindings::AbortControllerBinding::AbortControllerMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct AbortController {
    reflector_: Reflector,
}

impl AbortController {
    fn new_inherited() -> AbortController {
        AbortController {
            reflector_: Reflector::new(),
        }
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortController> {
        reflect_dom_object_with_proto(
            Box::new(AbortController::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }
}

impl AbortControllerMethods<crate::DomTypeHolder> for AbortController {
    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abortcontroller>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortController> {
        AbortController::new_with_proto(global, proto, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-abortcontroller-abort>
    fn Abort(&self, _cx: JSContext, _reason: Handle<'_, Value>) {}
}
