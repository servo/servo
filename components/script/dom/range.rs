/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RangeBinding;
use dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, Window};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::document::Document;

#[jstraceable]
#[must_root]
pub struct Range {
    reflector_: Reflector
}

impl Range {
    fn new_inherited() -> Range {
        Range {
            reflector_: Reflector::new()
        }
    }

    pub fn new(document: JSRef<Document>) -> Temporary<Range> {
        let window = document.window.root();
        reflect_dom_object(box Range::new_inherited(),
                           &Window(*window),
                           RangeBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<Range>> {
        let document = global.as_window().Document().root();
        Ok(Range::new(*document))
    }
}

impl<'a> RangeMethods for JSRef<'a, Range> {
    /// http://dom.spec.whatwg.org/#dom-range-detach
    fn Detach(self) {
        // This method intentionally left blank.
    }
}

impl Reflectable for Range {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
