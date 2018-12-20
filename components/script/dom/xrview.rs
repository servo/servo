/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
}

impl XRView {
    fn new_inherited() -> XRView {
        XRView {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRView> {
        reflect_dom_object(
            Box::new(XRView::new_inherited()),
            global,
            XRViewBinding::Wrap,
        )
    }
}
