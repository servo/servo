/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

 use dom_struct::dom_struct;
 use js::jsapi::Value;
 use js::jsapi::Heap;
 use js::jsapi::{ExceptionStackBehavior, JS_SetPendingException}; 
 use js::rust::{Handle, HandleObject};
 use js::rust::{MutableHandleValue, HandleValue};
 use js::jsval::JSVal; 
 use crate::dom::bindings::codegen::Bindings::AbortSignalBinding::AbortSignalMethods;
 use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
 use crate::dom::bindings::root::DomRoot;
 use crate::dom::eventtarget::EventTarget;
 use crate::dom::globalscope::GlobalScope;
 use crate::script_runtime::{CanGc, JSContext};
 
 #[dom_struct]
 pub(crate) struct AbortSignal {
     eventtarget: EventTarget,

     /// <https://dom.spec.whatwg.org/#abortsignal-abort-reason>
     #[ignore_malloc_size_of = "mozjs"]
     abort_reason: Heap<JSVal>,

 }
 
 impl AbortSignal {
     fn new_inherited() -> AbortSignal {
         AbortSignal {
            eventtarget: EventTarget::new_inherited(),
            abort_reason: Default::default(),
         }
     }
 
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

    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-reason>
    fn Reason(&self, _: JSContext, mut rval: MutableHandleValue) { 

    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-throwifaborted>
    #[allow(unsafe_code)]
    fn ThrowIfAborted(&self) { 
    }

    /// <https://dom.spec.whatwg.org/#dom-abortsignal-onabort>
    event_handler!(abort, GetOnabort, SetOnabort);
 }