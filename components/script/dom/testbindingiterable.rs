/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::TestBindingIterableBinding::{self, TestBindingIterableMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;

#[dom_struct]
pub struct TestBindingIterable {
    reflector: Reflector,
    vals: DOMRefCell<Vec<DOMString>>,
}

impl TestBindingIterable {
    fn new(global: GlobalRef) -> Root<TestBindingIterable> {
        reflect_dom_object(box TestBindingIterable {
            reflector: Reflector::new(),
            vals: DOMRefCell::new(vec![]),
        }, global, TestBindingIterableBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<TestBindingIterable>> {
        Ok(TestBindingIterable::new(global))
    }
}

impl TestBindingIterableMethods for TestBindingIterable {
    fn Add(&self, v: DOMString) { self.vals.borrow_mut().push(v); }
    fn Length(&self) -> u32 { self.vals.borrow().len() as u32 }
    fn GetItem(&self, n: u32) -> DOMString { self.vals.borrow().get(n as usize).unwrap().clone() }
    fn IndexedGetter(&self, n: u32, found: &mut bool) -> DOMString {
        let s = self.GetItem(n);
        *found = true;
        s
    }
}
