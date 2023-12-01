/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::DynamicModuleOwnerBinding::DynamicModuleOwnerMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;

/// An unique id for dynamic module
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub struct DynamicModuleId(#[no_trace] pub Uuid);

#[dom_struct]
pub struct DynamicModuleOwner {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc"]
    promise: Rc<Promise>,

    /// Unique id for each dynamic module
    #[ignore_malloc_size_of = "Defined in uuid"]
    id: DynamicModuleId,
}

impl DynamicModuleOwner {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(promise: Rc<Promise>, id: DynamicModuleId) -> Self {
        DynamicModuleOwner {
            reflector_: Reflector::new(),
            promise,
            id,
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(global: &GlobalScope, promise: Rc<Promise>, id: DynamicModuleId) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(DynamicModuleOwner::new_inherited(promise, id)),
            global,
        )
    }
}

impl DynamicModuleOwnerMethods for DynamicModuleOwner {
    // https://html.spec.whatwg.org/multipage/#integration-with-the-javascript-module-system:import()
    fn Promise(&self) -> Rc<Promise> {
        self.promise.clone()
    }
}
