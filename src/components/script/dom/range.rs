/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RangeBinding;
use dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};

#[deriving(Encodable)]
pub struct Range {
    pub reflector_: Reflector
}

impl Range {
    pub fn new_inherited() -> Range {
        Range {
            reflector_: Reflector::new()
        }
    }

    pub fn new(global: &GlobalRef) -> Temporary<Range> {
        reflect_dom_object(box Range::new_inherited(), global, RangeBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<Range>> {
        Ok(Range::new(global))
    }
}

impl<'a> RangeMethods for JSRef<'a, Range> {
}

impl Reflectable for Range {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
