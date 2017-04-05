/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::codegen::Bindings::TestBindingProxyBinding::TestBindingProxyMethods;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom_struct::dom_struct;

#[dom_struct]
pub struct TestBindingProxy {
    reflector_: Reflector
}

impl TestBindingProxyMethods for TestBindingProxy {
    fn Length(&self) -> u32 { 0 }
    fn SupportedPropertyNames(&self) -> Vec<DOMString> { vec![] }
    fn GetNamedItem(&self, _: DOMString) -> DOMString { DOMString::new() }
    fn SetNamedItem(&self, _: DOMString, _: DOMString) {}
    fn GetItem(&self, _: u32) -> DOMString { DOMString::new() }
    fn SetItem(&self, _: u32, _: DOMString) {}
    fn RemoveItem(&self, _: DOMString) {}
    fn Stringifier(&self) -> DOMString { DOMString::new() }
    fn IndexedGetter(&self, _: u32) -> Option<DOMString> { None }
    fn NamedDeleter(&self, _: DOMString) {}
    fn IndexedSetter(&self, _: u32, _: DOMString) {}
    fn NamedSetter(&self, _: DOMString, _: DOMString) {}
    fn NamedGetter(&self, _: DOMString) -> Option<DOMString> { None }

}
