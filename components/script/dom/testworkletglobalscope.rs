/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::PipelineId;
use crossbeam_channel::Sender;
use dom_struct::dom_struct;
use js::rust::Runtime;
use servo_url::ServoUrl;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TestWorkletGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::TestWorkletGlobalScopeBinding::TestWorkletGlobalScopeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::worklet::WorkletExecutor;
use crate::dom::workletglobalscope::{WorkletGlobalScope, WorkletGlobalScopeInit};
use crate::script_runtime::JSContext;

// check-tidy: no specs after this line

#[dom_struct]
pub struct TestWorkletGlobalScope {
    // The worklet global for this object
    worklet_global: WorkletGlobalScope,
    // The key/value pairs
    lookup_table: DomRefCell<HashMap<String, String>>,
}

impl TestWorkletGlobalScope {
    #[allow(unsafe_code)]
    pub fn new(
        runtime: &Runtime,
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
    ) -> DomRoot<TestWorkletGlobalScope> {
        debug!(
            "Creating test worklet global scope for pipeline {}.",
            pipeline_id
        );
        let global = Box::new(TestWorkletGlobalScope {
            worklet_global: WorkletGlobalScope::new_inherited(
                pipeline_id,
                base_url,
                executor,
                init,
            ),
            lookup_table: Default::default(),
        });
        unsafe { TestWorkletGlobalScopeBinding::Wrap(JSContext::from_ptr(runtime.cx()), global) }
    }

    pub fn perform_a_worklet_task(&self, task: TestWorkletTask) {
        match task {
            TestWorkletTask::Lookup(key, sender) => {
                debug!("Looking up key {}.", key);
                let result = self.lookup_table.borrow().get(&key).cloned();
                let _ = sender.send(result);
            },
        }
    }
}

impl TestWorkletGlobalScopeMethods for TestWorkletGlobalScope {
    fn RegisterKeyValue(&self, key: DOMString, value: DOMString) {
        debug!("Registering test worklet key/value {}/{}.", key, value);
        self.lookup_table
            .borrow_mut()
            .insert(String::from(key), String::from(value));
    }
}

/// Tasks which can be performed by test worklets.
pub enum TestWorkletTask {
    Lookup(String, Sender<Option<String>>),
}
