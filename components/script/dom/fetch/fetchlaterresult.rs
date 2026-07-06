/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::FetchLaterResultBinding::FetchLaterResultMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::fetch::DeferredFetchRecordId;

/// <https://fetch.spec.whatwg.org/#fetchlaterresult>
#[dom_struct]
pub(crate) struct FetchLaterResult {
    reflector_: Reflector,

    /// <https://fetch.spec.whatwg.org/#fetchlaterresult-activated-getter-steps>
    #[no_trace]
    deferred_record_id: DeferredFetchRecordId,
}

impl FetchLaterResult {
    fn new_inherited(deferred_record_id: DeferredFetchRecordId) -> FetchLaterResult {
        FetchLaterResult {
            reflector_: Reflector::new(),
            deferred_record_id,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        deferred_record_id: DeferredFetchRecordId,
    ) -> DomRoot<FetchLaterResult> {
        reflect_dom_object_with_cx(
            Box::new(FetchLaterResult::new_inherited(deferred_record_id)),
            window,
            cx,
        )
    }
}

impl FetchLaterResultMethods<crate::DomTypeHolder> for FetchLaterResult {
    /// <https://fetch.spec.whatwg.org/#dom-fetchlaterresult-activated>
    fn Activated(&self) -> bool {
        // The activated getter steps are to return the result of running this’s activated getter steps.
        self.global()
            .deferred_fetch_record_for_id(&self.deferred_record_id)
            .activated_getter_steps()
    }
}
