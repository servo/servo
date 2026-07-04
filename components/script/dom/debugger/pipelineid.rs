/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::DebuggerAddDebuggeeEventBinding::PipelineIdMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct PipelineId {
    reflector_: Reflector,
    #[no_trace]
    inner: servo_base::id::PipelineId,
}

impl PipelineId {
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        pipeline_id: servo_base::id::PipelineId,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(
            Box::new(Self {
                reflector_: Reflector::new(),
                inner: pipeline_id,
            }),
            global,
            cx,
        )
    }
}

impl PipelineIdMethods<crate::DomTypeHolder> for PipelineId {
    // check-tidy: no specs after this line
    fn NamespaceId(&self) -> u32 {
        self.inner.namespace_id.0
    }

    fn Index(&self) -> u32 {
        self.inner.index.0.get()
    }
}
