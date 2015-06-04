/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 use dom::bindings::codegen::Bindings::TestBindingProxyBinding::TestBindingProxyMethods;
 use dom::testbinding::TestBinding;
 use dom::bindings::global::{GlobalField};
 use dom::bindings::js::{JSRef};
 use dom::bindings::utils::{Reflector};
 use util::str::DOMString;


 #[dom_struct]
 pub struct TestBindingProxy {
     testBinding : TestBinding,
     reflector: Reflector,
     global: GlobalField
 }

 impl<'a> TestBindingProxyMethods for JSRef<'a, TestBindingProxy> {

     fn GetNamedItem(self, name: DOMString) -> DOMString {""}
     fn SetNamedItem(self, name: DOMString, value: DOMString) -> () {}
     fn GetItem(self, index: u32) -> DOMString {""}
     fn SetItem(self, index: u32, value: DOMString) -> () {}
     fn RemoveItem(self, name: DOMString) -> () {}
     fn Stringifier(self) -> DOMString {""}
     fn NamedCreator(self, name: DOMString, value: DOMString) -> () {}
     fn IndexedGetter(self, index: u32, found: &mut bool) -> DOMString {""}
     fn NamedDeleter(self, name: DOMString) -> () {}
     fn IndexedSetter(self, index: u32, value: DOMString) -> () {}
     fn NamedSetter(self, name: DOMString, value: DOMString) -> () {}
     fn IndexedCreator(self, index: u32, value: DOMString) -> () {}
     fn NamedGetter(self, name: DOMString, found: &mut bool) -> DOMString {""}

 }
