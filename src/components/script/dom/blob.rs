/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::Fallible;
use dom::bindings::codegen::BlobBinding;
use dom::window::Window;

pub struct Blob {
    reflector_: Reflector,
    window: @mut Window,
}

impl Blob {
    pub fn new_inherited(window: @mut Window) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window) -> @mut Blob {
        reflect_dom_object(@mut Blob::new_inherited(window), window, BlobBinding::Wrap)
    }
}

impl Blob {
    pub fn Constructor(window: @mut Window) -> Fallible<@mut Blob> {
        Ok(Blob::new(window))
    }

    pub fn Size(&self) -> u64 {
        0
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn Slice(&self, _start: i64, _end: i64, _contentType: Option<DOMString>) -> @mut Blob {
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
