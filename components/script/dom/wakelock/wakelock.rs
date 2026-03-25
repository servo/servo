use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, DocumentVisibilityState,
};
use crate::dom::bindings::codegen::Bindings::WakeLockBinding::{WakeLockMethods, WakeLockType};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_cx};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::wakelock::wakelocksentinel::WakeLockSentinel;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/screen-wake-lock/#the-wakelock-interface>
#[dom_struct]
pub(crate) struct WakeLock {
    reflector_: Reflector,
}

impl WakeLock {
    pub(crate) fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        _can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited()), global, cx)
    }
}

impl WakeLockMethods<crate::DomTypeHolder> for WakeLock {
    /// <https://w3c.github.io/screen-wake-lock/#the-request-method>
    fn Request(&self, _cx: &mut js::context::JSContext, type_: WakeLockType) -> Rc<Promise> {
        let global = self.global();
        let can_gc = CanGc::note();
        let promise = Promise::new(&global, can_gc);

        // Step 1. Let document be this's relevant global object's associated Document.
        let document = global.as_window().Document();

        // Step 2. If document is not fully active, reject with NotAllowedError.
        if !document.is_fully_active() {
            promise.reject_error(Error::NotAllowed(None), can_gc);
            return promise;
        }

        // Step 3. If document's visibility state is "hidden", reject with NotAllowedError.
        if document.VisibilityState() == DocumentVisibilityState::Hidden {
            promise.reject_error(Error::NotAllowed(None), can_gc);
            return promise;
        }

        // Step 4. Create a WakeLockSentinel and resolve the promise with it.
        // TODO: Notify the embedder to actually acquire the platform wake lock.
        let sentinel = WakeLockSentinel::new(&global, type_, can_gc);
        promise.resolve_native(&sentinel, can_gc);

        promise
    }
}
