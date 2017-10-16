/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MediaErrorBinding::{self, MediaErrorMethods};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MediaError {
    reflector_: Reflector,
    code: u16,
}

impl MediaError {
    fn new_inherited(code: u16) -> MediaError {
        MediaError {
            reflector_: Reflector::new(),
            code: code,
        }
    }

    pub fn new(window: &Window, code: u16) -> DomRoot<MediaError> {
        reflect_dom_object(Box::new(MediaError::new_inherited(code)),
                           window,
                           MediaErrorBinding::Wrap)
    }
}

impl MediaErrorMethods for MediaError {
    // https://html.spec.whatwg.org/multipage/#dom-mediaerror-code
    fn Code(&self) -> u16 {
        self.code
    }

    // https://html.spec.whatwg.org/multipage/#dom-mediaerror-message
    fn Message(&self) -> DOMString {
        DOMString::new()
    }
}
