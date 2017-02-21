/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::TestBindingPairIterableBinding;
use dom::bindings::codegen::Bindings::TestBindingPairIterableBinding::TestBindingPairIterableMethods;
use dom::bindings::error::Fallible;
use dom::bindings::iterable::Iterable;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct TestBindingPairIterable {
    reflector: Reflector,
    map: DOMRefCell<Vec<(DOMString, u32)>>,
}

impl Iterable for TestBindingPairIterable {
    type Key = DOMString;
    type Value = u32;
    fn get_iterable_length(&self) -> u32 {
        self.map.borrow().len() as u32
    }
    fn get_value_at_index(&self, index: u32) -> u32 {
        self.map.borrow().iter().nth(index as usize).map(|a| &a.1).unwrap().clone()
    }
    fn get_key_at_index(&self, index: u32) -> DOMString {
        self.map.borrow().iter().nth(index as usize).map(|a| &a.0).unwrap().clone()
    }
}

impl TestBindingPairIterable {
    fn new(global: &GlobalScope) -> Root<TestBindingPairIterable> {
        reflect_dom_object(box TestBindingPairIterable {
            reflector: Reflector::new(),
            map: DOMRefCell::new(vec![]),
        }, global, TestBindingPairIterableBinding::TestBindingPairIterableWrap)
    }

    pub fn Constructor(global: &GlobalScope) -> Fallible<Root<TestBindingPairIterable>> {
        Ok(TestBindingPairIterable::new(global))
    }
}

impl TestBindingPairIterableMethods for TestBindingPairIterable {
    fn Add(&self, key: DOMString, value: u32) {
        self.map.borrow_mut().push((key, value));
    }
}
