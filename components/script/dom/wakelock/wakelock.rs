/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use embedder_traits::{AllowOrDeny, EmbedderMsg};
use js::context::JSContext;
use js::realm::CurrentRealm;
use servo_constellation_traits::ScriptToConstellationMessage;

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
use crate::routed_promise::{RoutedPromiseListener, callback_promise};
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

    pub(crate) fn new(cx: &mut js::context::JSContext, global: &GlobalScope) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited()), global, cx)
    }
}

impl WakeLockMethods<crate::DomTypeHolder> for WakeLock {
    /// <https://w3c.github.io/screen-wake-lock/#the-request-method>
    fn Request(&self, cx: &mut CurrentRealm, _type_: WakeLockType) -> Rc<Promise> {
        let global = GlobalScope::from_current_realm(cx);
        let promise = Promise::new_in_realm(cx);

        // Step 1. Let document be this's relevant global object's associated Document.
        let document = global.as_window().Document();

        // Step 2. If document is not fully active, reject with NotAllowedError.
        if !document.is_fully_active() {
            promise.reject_error(Error::NotAllowed(None), CanGc::from_cx(cx));
            return promise;
        }

        // Step 3. If document's visibility state is "hidden", reject with NotAllowedError.
        if document.VisibilityState() == DocumentVisibilityState::Hidden {
            promise.reject_error(Error::NotAllowed(None), CanGc::from_cx(cx));
            return promise;
        }

        // Step 4. Obtain permission for "screen-wake-lock".
        // <https://w3c.github.io/screen-wake-lock/#dfn-obtain-permission>
        let Some(webview_id) = global.webview_id() else {
            promise.reject_error(Error::NotAllowed(None), CanGc::from_cx(cx));
            return promise;
        };

        let task_source = global.task_manager().dom_manipulation_task_source();
        let callback = callback_promise(&promise, self, task_source);
        global.send_to_embedder(EmbedderMsg::RequestWakeLockPermission(webview_id, callback));

        promise
    }
}

impl RoutedPromiseListener<AllowOrDeny> for WakeLock {
    /// <https://w3c.github.io/screen-wake-lock/#the-request-method>
    fn handle_response(&self, cx: &mut JSContext, response: AllowOrDeny, promise: &Rc<Promise>) {
        let can_gc = CanGc::from_cx(cx);
        match response {
            // Step 7a. If permission is denied, reject with NotAllowedError.
            AllowOrDeny::Deny => {
                promise.reject_error(Error::NotAllowed(None), can_gc);
            },
            // Step 7b-7c. Acquire the lock and resolve with a WakeLockSentinel.
            AllowOrDeny::Allow => {
                let global = self.global();
                global.as_window().send_to_constellation(
                    ScriptToConstellationMessage::AcquireWakeLock(
                        servo_wakelock::WakeLockType::Screen,
                    ),
                );

                let sentinel = WakeLockSentinel::new(cx, &global, WakeLockType::Screen);
                promise.resolve_native(&sentinel, can_gc);
            },
        }
    }
}
