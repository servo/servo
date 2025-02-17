/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::Rect;
use webxr_api::Viewport;

use crate::dom::bindings::codegen::Bindings::XRViewportBinding::XRViewportMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRViewport {
    reflector_: Reflector,
    #[no_trace]
    viewport: Rect<i32, Viewport>,
}

impl XRViewport {
    fn new_inherited(viewport: Rect<i32, Viewport>) -> XRViewport {
        XRViewport {
            reflector_: Reflector::new(),
            viewport,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        viewport: Rect<i32, Viewport>,
        can_gc: CanGc,
    ) -> DomRoot<XRViewport> {
        reflect_dom_object(
            Box::new(XRViewport::new_inherited(viewport)),
            global,
            can_gc,
        )
    }
}

impl XRViewportMethods<crate::DomTypeHolder> for XRViewport {
    /// <https://immersive-web.github.io/webxr/#dom-xrviewport-x>
    fn X(&self) -> i32 {
        self.viewport.origin.x
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrviewport-y>
    fn Y(&self) -> i32 {
        self.viewport.origin.y
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrviewport-width>
    fn Width(&self) -> i32 {
        self.viewport.size.width
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrviewport-height>
    fn Height(&self) -> i32 {
        self.viewport.size.height
    }
}
