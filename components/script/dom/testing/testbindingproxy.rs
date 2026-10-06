/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use js::context::NoGC;

use crate::dom::bindings::codegen::Bindings::TestBindingProxyBinding::TestBindingProxyMethods;
use crate::dom::bindings::str::RootedDOMString;
use crate::dom::testbinding::TestBinding;

#[dom_struct]
pub(crate) struct TestBindingProxy {
    testbinding_: TestBinding,
}

impl TestBindingProxyMethods<crate::DomTypeHolder> for TestBindingProxy {
    fn Length(&self) -> u32 {
        0
    }
    fn SupportedPropertyNames(&self, _: &NoGC) -> Vec<RootedDOMString> {
        vec![]
    }
    fn GetNamedItem(&self, _: RootedDOMString) -> RootedDOMString {
        RootedDOMString::new()
    }
    fn SetNamedItem(&self, _: RootedDOMString, _: RootedDOMString) {}
    fn GetItem(&self, _: u32) -> RootedDOMString {
        RootedDOMString::new()
    }
    fn SetItem(&self, _: u32, _: RootedDOMString) {}
    fn RemoveItem(&self, _: RootedDOMString) {}
    fn Stringifier(&self) -> RootedDOMString {
        RootedDOMString::new()
    }
    fn IndexedGetter(&self, _: u32) -> Option<RootedDOMString> {
        None
    }
    fn NamedDeleter(&self, _: RootedDOMString) {}
    fn IndexedSetter(&self, _: u32, _: RootedDOMString) {}
    fn NamedSetter(&self, _: RootedDOMString, _: RootedDOMString) {}
    fn NamedGetter(&self, _: RootedDOMString) -> Option<RootedDOMString> {
        None
    }
}
