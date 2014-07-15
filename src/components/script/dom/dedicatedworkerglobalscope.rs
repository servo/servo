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

use js::jsapi::JSContext;

#[deriving(Encodable)]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
}

impl DedicatedWorkerGlobalScope {
    pub fn new_inherited() -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(DedicatedGlobalScope),
        }
    }

    pub fn new(cx: *mut JSContext) -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited();
        DedicatedWorkerGlobalScopeBinding::Wrap(cx, scope)
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
