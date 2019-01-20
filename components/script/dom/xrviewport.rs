/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewportBinding;
use crate::dom::bindings::codegen::Bindings::XRViewportBinding::XRViewportMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct XRViewport {
    reflector_: Reflector,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl XRViewport {
    fn new_inherited(x: u32, y: u32, width: u32, height: u32) -> XRViewport {
        XRViewport {
            reflector_: Reflector::new(),
            x,
            y,
            width,
            height,
        }
    }

    pub fn new(
        global: &GlobalScope,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> DomRoot<XRViewport> {
        reflect_dom_object(
            Box::new(XRViewport::new_inherited(x, y, width, height)),
            global,
            XRViewportBinding::Wrap,
        )
    }
}

impl XRViewportMethods for XRViewport {
    /// https://immersive-web.github.io/webxr/#dom-xrviewport-x
    fn X(&self) -> i32 {
        self.x as i32
    }

    /// https://immersive-web.github.io/webxr/#dom-xrviewport-y
    fn Y(&self) -> i32 {
        self.y as i32
    }

    /// https://immersive-web.github.io/webxr/#dom-xrviewport-width
    fn Width(&self) -> i32 {
        self.height as i32
    }

    /// https://immersive-web.github.io/webxr/#dom-xrviewport-height
    fn Height(&self) -> i32 {
        self.height as i32
    }
}
