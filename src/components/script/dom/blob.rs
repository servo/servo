/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, Reflectable, Reflector, reflect_dom_object2};
use dom::bindings::utils::Fallible;
use dom::bindings::codegen::BlobBinding;
use dom::window::Window;

pub struct Blob {
    reflector_: Reflector,
    window: JSManaged<Window>
}

impl Blob {
    pub fn new_inherited(window: JSManaged<Window>) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            window: window
        }
    }

    pub fn new(window: JSManaged<Window>) -> JSManaged<Blob> {
        reflect_dom_object2(~Blob::new_inherited(window), window.value(), BlobBinding::Wrap)
    }
}

impl Blob {
    pub fn Constructor(window: JSManaged<Window>) -> Fallible<JSManaged<Blob>> {
        Ok(Blob::new(window))
    }

    pub fn Size(&self) -> u64 {
        0
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn Slice(&self, _start: i64, _end: i64, _contentType: Option<DOMString>) -> JSManaged<Blob> {
        Blob::new(self.window)
    }

    pub fn Close(&self) {}
}

impl Reflectable for Blob {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
