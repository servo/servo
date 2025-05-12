/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::{Heap, Value};
use js::jsval::JSVal;
use js::rust::{HandleObject, MutableHandleValue};

use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

/// <https://dom.spec.whatwg.org/#abortsignal>
#[dom_struct]
pub(crate) struct AbortSignal {
    eventtarget: EventTarget,

    /// <https://dom.spec.whatwg.org/#abortsignal-abort-reason>
    #[ignore_malloc_size_of = "mozjs"]
    abort_reason: Heap<JSVal>,
}

impl AbortSignal {
    #[allow(dead_code)]
    fn new_inherited() -> AbortSignal {
        AbortSignal {
            eventtarget: EventTarget::new_inherited(),
            abort_reason: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<AbortSignal> {
        reflect_dom_object_with_proto(
            Box::new(AbortSignal::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }
}

impl AbortSignalMethods<crate::DomTypeHolder> for AbortSignal {
    /// <https://dom.spec.whatwg.org/#dom-abortsignal-aborted>
    fn Aborted(&self) -> bool {
        // TODO
        false
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-reason>
    fn Reason(&self, _: JSContext,  _rval: MutableHandleValue) {
        // TODO
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-throwifaborted>
    #[allow(unsafe_code)]
    fn ThrowIfAborted(&self) {
        // TODO
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);
}
