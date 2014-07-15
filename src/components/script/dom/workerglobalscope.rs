/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, WorkerGlobalScopeTypeId};

use js::jsapi::JSContext;
use js::rust::Cx;

use std::rc::Rc;

#[deriving(PartialEq,Encodable)]
pub enum WorkerGlobalScopeId {
    DedicatedGlobalScope,
}

#[deriving(Encodable)]
pub struct WorkerGlobalScope {
    pub eventtarget: EventTarget,
    js_context: Untraceable<Rc<Cx>>,
}

impl WorkerGlobalScope {
    pub fn new_inherited(type_id: WorkerGlobalScopeId,
                         cx: Rc<Cx>) -> WorkerGlobalScope {
        WorkerGlobalScope {
            eventtarget: EventTarget::new_inherited(WorkerGlobalScopeTypeId(type_id)),
            js_context: Untraceable::new(cx),
        }
    }

    pub fn get_rust_cx<'a>(&'a self) -> &'a Rc<Cx> {
        &*self.js_context
    }
    pub fn get_cx(&self) -> *mut JSContext {
        self.js_context.ptr
    }
}

pub trait WorkerGlobalScopeMethods {
}

impl Reflectable for WorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}
