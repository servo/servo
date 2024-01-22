/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::XRSubImageBinding::XRSubImage_Binding::XRSubImageMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::xrviewport::XRViewport;

#[dom_struct]
pub struct XRSubImage {
    reflector: Reflector,
    viewport: Dom<XRViewport>,
}

impl XRSubImageMethods for XRSubImage {
    /// <https://immersive-web.github.io/layers/#dom-xrsubimage-viewport>
    fn Viewport(&self) -> DomRoot<XRViewport> {
        DomRoot::from_ref(&self.viewport)
    }
}
