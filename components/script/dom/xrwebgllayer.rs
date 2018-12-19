/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrlayer::XRLayer;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRWebGLLayer {
    xrlayer: XRLayer,
}

impl XRWebGLLayer {
    pub fn new_inherited() -> XRWebGLLayer {
        XRWebGLLayer {
            xrlayer: XRLayer::new_inherited(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object(
            Box::new(XRWebGLLayer::new_inherited()),
            global,
            XRWebGLLayerBinding::Wrap,
        )
    }
}
