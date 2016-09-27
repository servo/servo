/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::WorkerId;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::Reflectable;
use dom::crypto::Crypto;
use dom::eventtarget::EventTarget;
use js::jsapi::{JS_GetContext, JS_GetObjectRuntime, JSContext};
use std::cell::Cell;

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableHeap<JS<Crypto>>,
    next_worker_id: Cell<WorkerId>,

    /// A flag to indicate whether the developer tools has requested
    /// live updates from the worker.
    devtools_wants_updates: Cell<bool>,
}

impl GlobalScope {
    pub fn new_inherited() -> GlobalScope {
        GlobalScope {
            eventtarget: EventTarget::new_inherited(),
            crypto: Default::default(),
            next_worker_id: Cell::new(WorkerId(0)),
            devtools_wants_updates: Default::default(),
        }
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> *mut JSContext {
        unsafe {
            let runtime = JS_GetObjectRuntime(
                self.reflector().get_jsobject().get());
            assert!(!runtime.is_null());
            let context = JS_GetContext(runtime);
            assert!(!context.is_null());
            context
        }
    }

    pub fn crypto(&self) -> Root<Crypto> {
        self.crypto.or_init(|| Crypto::new(self))
    }

    /// Get next worker id.
    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }

    pub fn live_devtools_updates(&self) -> bool {
        self.devtools_wants_updates.get()
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }
}
