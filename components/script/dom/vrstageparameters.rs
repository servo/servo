/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VRStageParametersBinding;
use dom::bindings::codegen::Bindings::VRStageParametersBinding::VRStageParametersMethods;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use js::typedarray::{Float32Array, CreateWith};
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
        let stage = VRStageParameters {
            reflector_: Reflector::new(),
            parameters: DOMRefCell::new(parameters),
            transform: Heap::default()
        };
        // XXX unsound!
        unsafe {
            let _ = Float32Array::create(global.get_cx(),
                                         CreateWith::Slice(&stage.parameters.borrow().sitting_to_standing_transform),
                                         stage.transform.handle_mut());
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
            let cx = self.global().get_cx();
            typedarray!(in(cx) let array: Float32Array = self.transform.get());
            if let Ok(mut array) = array {
                array.update(&parameters.sitting_to_standing_transform);
            }
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
