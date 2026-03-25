/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use base::generic_channel;
use dom_struct::dom_struct;
use embedder_traits::{AllowOrDeny, EmbedderMsg};

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

        // Step 4. Ask the embedder whether the wake lock can be acquired.
        // <https://w3c.github.io/screen-wake-lock/#dfn-acquire-wake-lock>
        let window = global.as_window();
        let (sender, receiver) =
            generic_channel::channel::<AllowOrDeny>().expect("Failed to create wake lock channel");
        window.send_to_embedder(EmbedderMsg::AcquireWakeLock(window.webview_id(), sender));

        // Step 5. If the embedder denied the request, reject with NotAllowedError.
        match receiver.recv() {
            Ok(AllowOrDeny::Allow) => {},
            _ => {
                promise.reject_error(Error::NotAllowed(None), can_gc);
                return promise;
            },
        }

        // Step 6. Create a WakeLockSentinel and resolve the promise with it.
        let sentinel = WakeLockSentinel::new(&global, type_, can_gc);
        promise.resolve_native(&sentinel, can_gc);

        promise
    }
}
