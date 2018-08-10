/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::TestBindingIterableBinding::{self, TestBindingIterableMethods};
use dom::bindings::error::Fallible;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct TestBindingIterable<TH: TypeHolderTrait> {
    reflector: Reflector<TH>,
    vals: DomRefCell<Vec<DOMString>>,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> TestBindingIterable<TH> {
    fn new(global: &GlobalScope<TH>) -> DomRoot<TestBindingIterable<TH>> {
        reflect_dom_object(Box::new(TestBindingIterable {
            reflector: Reflector::new(),
            vals: DomRefCell::new(vec![]),
            _p: Default::default(),
        }), global, TestBindingIterableBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope<TH>) -> Fallible<DomRoot<TestBindingIterable<TH>>> {
        Ok(TestBindingIterable::new(global))
    }
}

impl<TH: TypeHolderTrait> TestBindingIterableMethods for TestBindingIterable<TH> {
    fn Add(&self, v: DOMString) { self.vals.borrow_mut().push(v); }
    fn Length(&self) -> u32 { self.vals.borrow().len() as u32 }
    fn GetItem(&self, n: u32) -> DOMString { self.IndexedGetter(n).unwrap_or_default() }
    fn IndexedGetter(&self, n: u32) -> Option<DOMString> {
        self.vals.borrow().get(n as usize).cloned()
    }
}
