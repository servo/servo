/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRLayerBinding;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRLayer {
    reflector_: Reflector,
}

impl XRLayer {
    pub fn new_inherited() -> XRLayer {
        XRLayer {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRLayer> {
        reflect_dom_object(
            Box::new(XRLayer::new_inherited()),
            global,
            XRLayerBinding::Wrap,
        )
    }
}
