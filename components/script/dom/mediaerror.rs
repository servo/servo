/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MediaErrorBinding::{self, MediaErrorMethods};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::window::Window;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct MediaError<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    code: u16,
    _p: PhantomData<TH>,
}

impl<TH: TypeHolderTrait> MediaError<TH> {
    fn new_inherited(code: u16) -> MediaError<TH> {
        MediaError {
            reflector_: Reflector::new(),
            code: code,
            _p: Default::default(),
        }
    }

    pub fn new(window: &Window<TH>, code: u16) -> DomRoot<MediaError<TH>> {
        reflect_dom_object(Box::new(MediaError::new_inherited(code)),
                           window,
                           MediaErrorBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> MediaErrorMethods for MediaError<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-mediaerror-code
    fn Code(&self) -> u16 {
        self.code
    }

    // https://html.spec.whatwg.org/multipage/#dom-mediaerror-message
    fn Message(&self) -> DOMString {
        DOMString::new()
    }
}
