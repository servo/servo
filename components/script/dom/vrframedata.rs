/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::VRFrameDataBinding;
use dom::bindings::codegen::Bindings::VRFrameDataBinding::VRFrameDataMethods;
use dom::bindings::error::Fallible;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::globalscope::GlobalScope;
use dom::vrpose::VRPose;
use dom::window::Window;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use js::typedarray::{Float32Array, CreateWith};
use std::cell::Cell;
use std::ptr;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct VRFrameData {
    reflector_: Reflector,
    left_proj: Heap<*mut JSObject>,
    left_view: Heap<*mut JSObject>,
    right_proj: Heap<*mut JSObject>,
    right_view: Heap<*mut JSObject>,
    pose: Dom<VRPose>,
    timestamp: Cell<f64>,
    first_timestamp: Cell<f64>
}

impl VRFrameData {
    fn new_inherited(pose: &VRPose) -> VRFrameData {
        VRFrameData {
            reflector_: Reflector::new(),
            left_proj: Heap::default(),
            left_view: Heap::default(),
            right_proj: Heap::default(),
            right_view: Heap::default(),
            pose: Dom::from_ref(&*pose),
            timestamp: Cell::new(0.0),
            first_timestamp: Cell::new(0.0)
        }
    }

    #[allow(unsafe_code)]
    fn new(global: &GlobalScope) -> DomRoot<VRFrameData> {
        let matrix = [1.0, 0.0, 0.0, 0.0,
                      0.0, 1.0, 0.0, 0.0,
                      0.0, 0.0, 1.0, 0.0,
                      0.0, 0.0, 0.0, 1.0f32];
        let pose = VRPose::new(&global, &Default::default());

        let root = reflect_dom_object(Box::new(VRFrameData::new_inherited(&pose)),
                                      global,
                                      VRFrameDataBinding::Wrap);
        let cx = global.get_cx();
        create_typed_array(cx, &matrix, &root.left_proj);
        create_typed_array(cx, &matrix, &root.left_view);
        create_typed_array(cx, &matrix, &root.right_proj);
        create_typed_array(cx, &matrix, &root.right_view);

        root
    }

    pub fn Constructor(window: &Window) -> Fallible<DomRoot<VRFrameData>> {
        Ok(VRFrameData::new(&window.global()))
    }
}


#[allow(unsafe_code)]
fn create_typed_array(cx: *mut JSContext, src: &[f32], dst: &Heap<*mut JSObject>) {
    rooted!(in (cx) let mut array = ptr::null_mut());
    unsafe {
        let _ = Float32Array::create(cx, CreateWith::Slice(src), array.handle_mut());
    }
    (*dst).set(array.get());
}

impl VRFrameData {
    #[allow(unsafe_code)]
    pub fn update(&self, data: &WebVRFrameData) {
        unsafe {
            let cx = self.global().get_cx();
            typedarray!(in(cx) let left_proj_array: Float32Array = self.left_proj.get());
            if let Ok(mut array) = left_proj_array {
                array.update(&data.left_projection_matrix);
            }
            typedarray!(in(cx) let left_view_array: Float32Array = self.left_view.get());
            if let Ok(mut array) = left_view_array {
                array.update(&data.left_view_matrix);
            }
            typedarray!(in(cx) let right_proj_array: Float32Array = self.right_proj.get());
            if let Ok(mut array) = right_proj_array {
                array.update(&data.right_projection_matrix);
            }
            typedarray!(in(cx) let right_view_array: Float32Array = self.right_view.get());
            if let Ok(mut array) = right_view_array {
                array.update(&data.right_view_matrix);
            }
        }
        self.pose.update(&data.pose);
        self.timestamp.set(data.timestamp);
        if self.first_timestamp.get() == 0.0 {
            self.first_timestamp.set(data.timestamp);
        }
    }
}

impl VRFrameDataMethods for VRFrameData {
    // https://w3c.github.io/webvr/#dom-vrframedata-timestamp
    fn Timestamp(&self) -> Finite<f64> {
        Finite::wrap(self.timestamp.get() - self.first_timestamp.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-leftprojectionmatrix
    unsafe fn LeftProjectionMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new_unchecked(self.left_proj.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-leftviewmatrix
    unsafe fn LeftViewMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new_unchecked(self.left_view.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-rightprojectionmatrix
    unsafe fn RightProjectionMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new_unchecked(self.right_proj.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-rightviewmatrix
    unsafe fn RightViewMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new_unchecked(self.right_view.get())
    }

    // https://w3c.github.io/webvr/#dom-vrframedata-pose
    fn Pose(&self) -> DomRoot<VRPose> {
        DomRoot::from_ref(&*self.pose)
    }
}
