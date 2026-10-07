/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom_struct::dom_struct;
use js::context::NoGC;

use crate::dom::bindings::codegen::Bindings::TestBindingProxyBinding::TestBindingProxyMethods;
use crate::dom::bindings::str::RootedDomString;
use crate::dom::testbinding::TestBinding;

#[dom_struct]
pub(crate) struct TestBindingProxy {
    testbinding_: TestBinding,
}

impl TestBindingProxyMethods<crate::DomTypeHolder> for TestBindingProxy {
    fn Length(&self) -> u32 {
        0
    }
    fn SupportedPropertyNames(&self, _: &NoGC) -> Vec<RootedDomString> {
        vec![]
    }
    fn GetNamedItem(&self, _: RootedDomString) -> RootedDomString {
        RootedDomString::new()
    }
    fn SetNamedItem(&self, _: RootedDomString, _: RootedDomString) {}
    fn GetItem(&self, _: u32) -> RootedDomString {
        RootedDomString::new()
    }
    fn SetItem(&self, _: u32, _: RootedDomString) {}
    fn RemoveItem(&self, _: RootedDomString) {}
    fn Stringifier(&self) -> RootedDomString {
        RootedDomString::new()
    }
    fn IndexedGetter(&self, _: u32) -> Option<RootedDomString> {
        None
    }
    fn NamedDeleter(&self, _: RootedDomString) {}
    fn IndexedSetter(&self, _: u32, _: RootedDomString) {}
    fn NamedSetter(&self, _: RootedDomString, _: RootedDomString) {}
    fn NamedGetter(&self, _: RootedDomString) -> Option<RootedDomString> {
        None
    }
}
