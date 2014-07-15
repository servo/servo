/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeDerived;
use dom::bindings::js::Temporary;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::EventTarget;
use dom::eventtarget::WorkerGlobalScopeTypeId;
use dom::workerglobalscope::DedicatedGlobalScope;
use dom::workerglobalscope::WorkerGlobalScope;
use script_task::ScriptTask;

use js::rust::Cx;

use std::rc::Rc;

#[deriving(Encodable)]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
}

impl DedicatedWorkerGlobalScope {
    pub fn new_inherited(cx: Rc<Cx>) -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(DedicatedGlobalScope, cx),
        }
    }

    pub fn new(cx: Rc<Cx>) -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(cx.clone());
        DedicatedWorkerGlobalScopeBinding::Wrap(cx.ptr, scope)
    }

    pub fn init() -> Temporary<DedicatedWorkerGlobalScope> {
        let (_js_runtime, js_context) = ScriptTask::new_rt_and_cx();
        DedicatedWorkerGlobalScope::new(js_context.clone())
    }

    pub fn get_rust_cx<'a>(&'a self) -> &'a Rc<Cx> {
        self.workerglobalscope.get_rust_cx()
    }
}

pub trait DedicatedWorkerGlobalScopeMethods {
}

impl Reflectable for DedicatedWorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.workerglobalscope.reflector()
    }
}

impl DedicatedWorkerGlobalScopeDerived for EventTarget {
    fn is_dedicatedworkerglobalscope(&self) -> bool {
        match self.type_id {
            WorkerGlobalScopeTypeId(DedicatedGlobalScope) => true,
            _ => false
        }
    }
}
