/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::Fallible;
use dom::bindings::codegen::BlobBinding;
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct Blob {
    reflector_: Reflector,
    window: JS<Window>
}

impl Blob {
    pub fn new_inherited(window: JS<Window>) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            window: window
        }
    }

    pub fn new(window: &JS<Window>) -> JS<Blob> {
        reflect_dom_object(~Blob::new_inherited(window.clone()),
                           window.get(),
                           BlobBinding::Wrap)
    }
}

impl Blob {
    pub fn Constructor(window: &JS<Window>) -> Fallible<JS<Blob>> {
        Ok(Blob::new(window))
    }

    pub fn Size(&self) -> u64 {
        0
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn Slice(&self, _start: i64, _end: i64, _contentType: Option<DOMString>) -> JS<Blob> {
        Blob::new(&self.window)
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
