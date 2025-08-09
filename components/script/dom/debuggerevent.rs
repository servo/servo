/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::{JSObject, Value};
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::reflector::DomObject;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::DebuggerEventBinding::DebuggerEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::Event;
use crate::dom::types::{GlobalScope, PipelineId};
use crate::script_runtime::CanGc;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::DebuggerGlobalScope`].
pub(crate) struct DebuggerEvent {
    event: Event,
    global: Dom<GlobalScope>,
    pipeline_id: Dom<PipelineId>,
    worker_id: Option<DOMString>,
}

impl DebuggerEvent {
    pub(crate) fn new(
        debugger_global: &GlobalScope,
        global: &GlobalScope,
        pipeline_id: &PipelineId,
        worker_id: Option<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let result = Box::new(Self {
            event: Event::new_inherited(),
            global: Dom::from_ref(global),
            pipeline_id: Dom::from_ref(pipeline_id),
            worker_id,
        });
        let result = reflect_dom_object(result, debugger_global, can_gc);
        result.event.init_event("addDebuggee".into(), false, false);

        result
    }
}

impl DebuggerEventMethods<crate::DomTypeHolder> for DebuggerEvent {
    // check-tidy: no specs after this line
    fn Global(&self, cx: script_bindings::script_runtime::JSContext) -> NonNull<JSObject> {
        // Convert the debuggee global’s reflector to a Value, wrapping it from its originating realm (debuggee realm)
        // into the active realm (debugger realm) so that it can be passed across compartments.
        rooted!(in(*cx) let mut result: Value);
        self.global
            .reflector()
            .safe_to_jsval(cx, result.handle_mut());
        NonNull::new(result.to_object()).unwrap()
    }

    fn PipelineId(
        &self,
    ) -> DomRoot<<crate::DomTypeHolder as script_bindings::DomTypes>::PipelineId> {
        DomRoot::from_ref(&self.pipeline_id)
    }

    fn GetWorkerId(&self) -> Option<DOMString> {
        self.worker_id.clone()
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
