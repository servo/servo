/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::TestBindingPairIterableBinding::TestBindingPairIterableMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct TestBindingPairIterable {
    reflector: Reflector,
    map: DomRefCell<Vec<(DOMString, u32)>>,
}

impl Iterable for TestBindingPairIterable {
    type Key = DOMString;
    type Value = u32;
    fn get_iterable_length(&self) -> u32 {
        self.map.borrow().len() as u32
    }
    fn get_value_at_index(&self, index: u32) -> u32 {
        *self.map.borrow().get(index as usize).map(|a| &a.1).unwrap()
    }
    fn get_key_at_index(&self, index: u32) -> DOMString {
        self.map
            .borrow()
            .get(index as usize)
            .map(|a| &a.0)
            .unwrap()
            .clone()
    }
}

impl TestBindingPairIterable {
    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<TestBindingPairIterable> {
        reflect_dom_object_with_proto(
            Box::new(TestBindingPairIterable {
                reflector: Reflector::new(),
                map: DomRefCell::new(vec![]),
            }),
            global,
            proto,
            can_gc,
        )
    }
}

impl TestBindingPairIterableMethods<crate::DomTypeHolder> for TestBindingPairIterable {
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TestBindingPairIterable>> {
        Ok(TestBindingPairIterable::new(global, proto, can_gc))
    }

    fn Add(&self, key: DOMString, value: u32) {
        self.map.borrow_mut().push((key, value));
    }
}
