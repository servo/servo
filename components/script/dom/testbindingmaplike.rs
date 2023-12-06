/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use indexmap::IndexMap;
use js::rust::HandleObject;

use super::bindings::error::Error;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TestBindingMaplikeBinding::TestBindingMaplikeMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::like::Maplike;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::maplike;

/// maplike<DOMString, long>
#[dom_struct]
pub struct TestBindingMaplike {
    reflector: Reflector,
    #[custom_trace]
    internal: DomRefCell<IndexMap<DOMString, i32>>,
}

impl TestBindingMaplike {
    fn new(global: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<TestBindingMaplike> {
        reflect_dom_object_with_proto(
            Box::new(TestBindingMaplike {
                reflector: Reflector::new(),
                internal: DomRefCell::new(IndexMap::new()),
            }),
            global,
            proto,
        )
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TestBindingMaplike>> {
        Ok(TestBindingMaplike::new(global, proto))
    }
}

impl TestBindingMaplikeMethods for TestBindingMaplike {
    fn SetInternal(&self, key: DOMString, value: i32) {
        self.internal.set(key, value)
    }

    fn ClearInternal(&self) {
        self.internal.clear()
    }

    fn DeleteInternal(&self, key: DOMString) -> bool {
        self.internal.delete(key)
    }

    fn HasInternal(&self, key: DOMString) -> bool {
        self.internal.has(key)
    }

    fn GetInternal(&self, key: DOMString) -> Fallible<i32> {
        // TODO: error type?
        self.internal
            .borrow()
            .get(&key)
            .ok_or_else(|| Error::Type(format!("No entry for key {key}")))
            .copied()
    }

    fn Size(&self) -> u32 {
        self.internal.size()
    }
}

// this error is wrong because if we inline Self::Key and Self::Value all errors are gone
// TODO: FIX THIS
#[allow(crown::unrooted_must_root)]
impl Maplike for TestBindingMaplike {
    type Key = DOMString;
    type Value = i32;

    maplike!(self, internal);
}
