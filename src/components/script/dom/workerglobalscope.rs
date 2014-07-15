/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, WorkerGlobalScopeTypeId};

#[deriving(PartialEq,Encodable)]
pub enum WorkerGlobalScopeId {
    DedicatedGlobalScope,
}

#[deriving(Encodable)]
pub struct WorkerGlobalScope {
    pub eventtarget: EventTarget,
}

impl WorkerGlobalScope {
    pub fn new_inherited(type_id: WorkerGlobalScopeId) -> WorkerGlobalScope {
        WorkerGlobalScope {
            eventtarget: EventTarget::new_inherited(WorkerGlobalScopeTypeId(type_id)),
        }
    }
}

pub trait WorkerGlobalScopeMethods {
}

impl Reflectable for WorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
