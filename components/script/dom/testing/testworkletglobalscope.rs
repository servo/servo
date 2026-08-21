/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::inheritance::Castable;
use script_bindings::interfaces::HasOrigin;
use servo_base::id::PipelineId;
use servo_url::{MutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::TestWorkletGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::TestWorkletGlobalScopeBinding::TestWorkletGlobalScopeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::worklet::WorkletExecutor;
use crate::dom::workletglobalscope::{WorkletGlobalScope, WorkletGlobalScopeInit};
use crate::runtime::microtask::MicrotaskQueue;

// check-tidy: no specs after this line

#[dom_struct]
pub(crate) struct TestWorkletGlobalScope {
    // The worklet global for this object
    worklet_global: WorkletGlobalScope,
    // The key/value pairs
    lookup_table: DomRefCell<HashMap<String, String>>,
}

impl TestWorkletGlobalScope {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        inherited_secure_context: Option<bool>,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
        cx: &mut JSContext,
        closing: Arc<AtomicBool>,
        microtask_queue: Rc<MicrotaskQueue>,
    ) -> DomRoot<TestWorkletGlobalScope> {
        debug!(
            "Creating test worklet global scope for pipeline {}.",
            pipeline_id
        );

        let global = Box::new(TestWorkletGlobalScope {
            worklet_global: WorkletGlobalScope::new_inherited(
                pipeline_id,
                base_url,
                inherited_secure_context,
                executor,
                init,
                closing,
                microtask_queue,
            ),
            lookup_table: Default::default(),
        });
        TestWorkletGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(cx, &global.origin(), global)
    }

    /// Get the value associated with the given key.
    pub fn lookup_value(&self, key: &str) -> Option<String> {
        self.lookup_table.borrow().get(key).cloned()
    }
}

impl TestWorkletGlobalScopeMethods<crate::DomTypeHolder> for TestWorkletGlobalScope {
    fn RegisterKeyValue(&self, key: DOMString, value: DOMString) {
        debug!("Registering test worklet key/value {}/{}.", key, value);
        self.lookup_table
            .borrow_mut()
            .insert(String::from(key), String::from(value));
    }
}

impl HasOrigin for TestWorkletGlobalScope {
    fn origin(&self) -> MutableOrigin {
        self.upcast::<WorkletGlobalScope>().origin()
    }
}
