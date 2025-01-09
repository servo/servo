/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::MediaErrorBinding::MediaErrorMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaError {
    reflector_: Reflector,
    code: u16,
}

impl MediaError {
    fn new_inherited(code: u16) -> MediaError {
        MediaError {
            reflector_: Reflector::new(),
            code,
        }
    }

    pub(crate) fn new(window: &Window, code: u16) -> DomRoot<MediaError> {
        reflect_dom_object(
            Box::new(MediaError::new_inherited(code)),
            window,
            CanGc::note(),
        )
    }
}

impl MediaErrorMethods<crate::DomTypeHolder> for MediaError {
    // https://html.spec.whatwg.org/multipage/#dom-mediaerror-code
    fn Code(&self) -> u16 {
        self.code
    }

    // https://html.spec.whatwg.org/multipage/#dom-mediaerror-message
    fn Message(&self) -> DOMString {
        DOMString::new()
    }
}
