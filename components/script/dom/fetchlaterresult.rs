/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use dom_struct::dom_struct;
use parking_lot::Mutex;

use crate::dom::bindings::codegen::Bindings::FetchLaterResultBinding::FetchLaterResultMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::fetch::DeferredFetchRecord;
use crate::script_runtime::CanGc;

/// <https://fetch.spec.whatwg.org/#fetchlaterresult>
#[dom_struct]
pub(crate) struct FetchLaterResult {
    reflector_: Reflector,

    /// <https://fetch.spec.whatwg.org/#fetchlaterresult-activated-getter-steps>
    #[conditional_malloc_size_of]
    #[no_trace]
    activated_getter_steps: Arc<Mutex<DeferredFetchRecord>>,
}

impl FetchLaterResult {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(activated_getter_steps: Arc<Mutex<DeferredFetchRecord>>) -> FetchLaterResult {
        FetchLaterResult {
            reflector_: Reflector::new(),
            activated_getter_steps,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        activated_getter_steps: Arc<Mutex<DeferredFetchRecord>>,
        can_gc: CanGc,
    ) -> DomRoot<FetchLaterResult> {
        reflect_dom_object(
            Box::new(FetchLaterResult::new_inherited(activated_getter_steps)),
            window,
            can_gc,
        )
    }
}

impl FetchLaterResultMethods<crate::DomTypeHolder> for FetchLaterResult {
    /// <https://fetch.spec.whatwg.org/#dom-fetchlaterresult-activated>
    fn Activated(&self) -> bool {
        // The activated getter steps are to return the result of running this’s activated getter steps.
        self.activated_getter_steps.lock().activated_getter_steps()
    }
}
