/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::VRFieldOfViewBinding;
use dom::bindings::codegen::Bindings::VRFieldOfViewBinding::VRFieldOfViewMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webvr_traits::WebVRFieldOfView;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[dom_struct]
pub struct VRFieldOfView<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    fov: DomRefCell<WebVRFieldOfView>,
    _p: PhantomData<TH>,
}

unsafe_no_jsmanaged_fields!(WebVRFieldOfView);

impl<TH: TypeHolderTrait> VRFieldOfView<TH> {
    fn new_inherited(fov: WebVRFieldOfView) -> VRFieldOfView<TH> {
        VRFieldOfView {
            reflector_: Reflector::new(),
            fov: DomRefCell::new(fov),
            _p: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope<TH>, fov: WebVRFieldOfView) -> DomRoot<VRFieldOfView<TH>> {
        reflect_dom_object(Box::new(VRFieldOfView::new_inherited(fov)),
                           global,
                           VRFieldOfViewBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> VRFieldOfViewMethods for VRFieldOfView<TH> {
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
