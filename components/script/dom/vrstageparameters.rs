/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VRStageParametersBinding;
use dom::bindings::codegen::Bindings::VRStageParametersBinding::VRStageParametersMethods;
use dom::bindings::conversions::{slice_to_array_buffer_view, update_array_buffer_view};
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use js::jsapi::{Heap, JSContext, JSObject};
use webvr_traits::WebVRStageParameters;

#[dom_struct]
pub struct VRStageParameters {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    parameters: DOMRefCell<WebVRStageParameters>,
    transform: Heap<*mut JSObject>,
}

unsafe_no_jsmanaged_fields!(WebVRStageParameters);

impl VRStageParameters {
    #[allow(unsafe_code)]
    #[allow(unrooted_must_root)]
    fn new_inherited(parameters: WebVRStageParameters, global: &GlobalScope) -> VRStageParameters {
        let mut stage = VRStageParameters {
            reflector_: Reflector::new(),
            parameters: DOMRefCell::new(parameters),
            transform: Heap::default()
        };
        unsafe {
            stage.transform.set(slice_to_array_buffer_view(global.get_cx(),
                                                           &stage.parameters.borrow().sitting_to_standing_transform));
        }

        stage
    }

    pub fn new(parameters: WebVRStageParameters, global: &GlobalScope) -> Root<VRStageParameters> {
        reflect_dom_object(box VRStageParameters::new_inherited(parameters, global),
                           global,
                           VRStageParametersBinding::Wrap)
    }

    #[allow(unsafe_code)]
    pub fn update(&self, parameters: &WebVRStageParameters) {
        unsafe {
            update_array_buffer_view(self.transform.get(), &parameters.sitting_to_standing_transform);
        }
        *self.parameters.borrow_mut() = parameters.clone();
    }
}

impl VRStageParametersMethods for VRStageParameters {
    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrstageparameters-sittingtostandingtransform
    unsafe fn SittingToStandingTransform(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new(self.transform.get())
    }

    // https://w3c.github.io/webvr/#dom-vrstageparameters-sizex
    fn SizeX(&self) -> Finite<f32> {
        Finite::wrap(self.parameters.borrow().size_x)
    }

    // https://w3c.github.io/webvr/#dom-vrstageparameters-sizez
    fn SizeZ(&self) -> Finite<f32> {
        Finite::wrap(self.parameters.borrow().size_z)
    }
}
