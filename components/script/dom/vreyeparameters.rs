/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VREyeParametersBinding;
use dom::bindings::codegen::Bindings::VREyeParametersBinding::VREyeParametersMethods;
use dom::bindings::conversions::slice_to_array_buffer_view;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom::vrfieldofview::VRFieldOfView;
use js::jsapi::{Heap, JSContext, JSObject};
use std::default::Default;
use webvr_traits::WebVREyeParameters;

#[dom_struct]
pub struct VREyeParameters {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    parameters: DOMRefCell<WebVREyeParameters>,
    offset: Heap<*mut JSObject>,
    fov: JS<VRFieldOfView>,
}

unsafe_no_jsmanaged_fields!(WebVREyeParameters);

impl VREyeParameters {
    #[allow(unsafe_code)]
    #[allow(unrooted_must_root)]
    fn new_inherited(parameters: WebVREyeParameters, global: &GlobalScope) -> VREyeParameters {
        let fov = VRFieldOfView::new(&global, parameters.field_of_view.clone());
        let mut result = VREyeParameters {
            reflector_: Reflector::new(),
            parameters: DOMRefCell::new(parameters),
            offset: Heap::default(),
            fov: JS::from_ref(&*fov)
        };

        unsafe {
            result.offset.set(slice_to_array_buffer_view(global.get_cx(), &result.parameters.borrow().offset));
        }
        result
    }

    pub fn new(parameters: WebVREyeParameters, global: &GlobalScope) -> Root<VREyeParameters> {
        reflect_dom_object(box VREyeParameters::new_inherited(parameters, global),
                           global,
                           VREyeParametersBinding::Wrap)
    }
}

impl VREyeParametersMethods for VREyeParameters {
    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vreyeparameters-offset
    unsafe fn Offset(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new(self.offset.get())
    }

    // https://w3c.github.io/webvr/#dom-vreyeparameters-fieldofview
    fn FieldOfView(&self) -> Root<VRFieldOfView> {
        Root::from_ref(&*self.fov)
    }

    // https://w3c.github.io/webvr/#dom-vreyeparameters-renderwidth
    fn RenderWidth(&self) -> u32 {
        self.parameters.borrow().render_width
    }

    // https://w3c.github.io/webvr/#dom-vreyeparameters-renderheight
    fn RenderHeight(&self) -> u32 {
        self.parameters.borrow().render_height
    }
}
