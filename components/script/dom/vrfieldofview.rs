/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VRFieldOfViewBinding;
use dom::bindings::codegen::Bindings::VRFieldOfViewBinding::VRFieldOfViewMethods;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use webvr_traits::WebVRFieldOfView;

#[dom_struct]
pub struct VRFieldOfView {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    fov: DOMRefCell<WebVRFieldOfView>
}

unsafe_no_jsmanaged_fields!(WebVRFieldOfView);

impl VRFieldOfView {
    fn new_inherited(fov: WebVRFieldOfView) -> VRFieldOfView {
        VRFieldOfView {
            reflector_: Reflector::new(),
            fov: DOMRefCell::new(fov)
        }
    }

    pub fn new(global: &GlobalScope, fov: WebVRFieldOfView) -> Root<VRFieldOfView> {
        reflect_dom_object(box VRFieldOfView::new_inherited(fov),
                           global,
                           VRFieldOfViewBinding::Wrap)
    }
}

impl VRFieldOfViewMethods for VRFieldOfView {
    // https://w3c.github.io/webvr/#interface-interface-vrfieldofview
    fn UpDegrees(&self) -> Finite<f64> {
        Finite::wrap(self.fov.borrow().up_degrees)
    }

    // https://w3c.github.io/webvr/#interface-interface-vrfieldofview
    fn RightDegrees(&self) -> Finite<f64> {
        Finite::wrap(self.fov.borrow().right_degrees)
    }

    // https://w3c.github.io/webvr/#interface-interface-vrfieldofview
    fn DownDegrees(&self) -> Finite<f64> {
        Finite::wrap(self.fov.borrow().down_degrees)
    }

    // https://w3c.github.io/webvr/#interface-interface-vrfieldofview
    fn LeftDegrees(&self) -> Finite<f64> {
        Finite::wrap(self.fov.borrow().left_degrees)
    }
}
