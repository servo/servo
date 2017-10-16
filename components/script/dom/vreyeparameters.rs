/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::VREyeParametersBinding;
use dom::bindings::codegen::Bindings::VREyeParametersBinding::VREyeParametersMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::globalscope::GlobalScope;
use dom::vrfieldofview::VRFieldOfView;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use js::typedarray::{Float32Array, CreateWith};
use std::default::Default;
use std::ptr;
use webvr_traits::WebVREyeParameters;

#[dom_struct]
pub struct VREyeParameters {
    reflector_: Reflector,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    parameters: DomRefCell<WebVREyeParameters>,
    offset: Heap<*mut JSObject>,
    fov: Dom<VRFieldOfView>,
}

unsafe_no_jsmanaged_fields!(WebVREyeParameters);

impl VREyeParameters {
    fn new_inherited(parameters: WebVREyeParameters, fov: &VRFieldOfView) -> VREyeParameters {
        VREyeParameters {
            reflector_: Reflector::new(),
            parameters: DomRefCell::new(parameters),
            offset: Heap::default(),
            fov: Dom::from_ref(&*fov)
        }
    }

    #[allow(unsafe_code)]
    pub fn new(parameters: WebVREyeParameters, global: &GlobalScope) -> DomRoot<VREyeParameters> {
        let fov = VRFieldOfView::new(&global, parameters.field_of_view.clone());

        let cx = global.get_cx();
        rooted!(in (cx) let mut array = ptr::null_mut());
        unsafe {
            let _ = Float32Array::create(cx, CreateWith::Slice(&parameters.offset), array.handle_mut());
        }

        let eye_parameters = reflect_dom_object(Box::new(VREyeParameters::new_inherited(parameters, &fov)),
                                                global,
                                                VREyeParametersBinding::Wrap);
        eye_parameters.offset.set(array.get());

        eye_parameters
    }
}

impl VREyeParametersMethods for VREyeParameters {
    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vreyeparameters-offset
    unsafe fn Offset(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new_unchecked(self.offset.get())
    }

    // https://w3c.github.io/webvr/#dom-vreyeparameters-fieldofview
    fn FieldOfView(&self) -> DomRoot<VRFieldOfView> {
        DomRoot::from_ref(&*self.fov)
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
