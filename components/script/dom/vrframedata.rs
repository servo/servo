/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::VRFrameDataBinding;
use dom::bindings::codegen::Bindings::VRFrameDataBinding::VRFrameDataMethods;
use dom::bindings::conversions::{slice_to_array_buffer_view, update_array_buffer_view};
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, Root};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom::vrpose::VRPose;
use dom::window::Window;
use js::jsapi::{Heap, JSContext, JSObject};
use std::cell::Cell;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct VRFrameData {
    reflector_: Reflector,
    left_proj: Heap<*mut JSObject>,
    left_view: Heap<*mut JSObject>,
    right_proj: Heap<*mut JSObject>,
    right_view: Heap<*mut JSObject>,
    pose: JS<VRPose>,
    timestamp: Cell<f64>,
    first_timestamp: Cell<f64>
}

impl VRFrameData {
    #[allow(unsafe_code)]
    #[allow(unrooted_must_root)]
    fn new(global: &GlobalScope) -> Root<VRFrameData> {
        let matrix = [1.0, 0.0, 0.0, 0.0,
                      0.0, 1.0, 0.0, 0.0,
                      0.0, 0.0, 1.0, 0.0,
                      0.0, 0.0, 0.0, 1.0f32];
        let pose = VRPose::new(&global, &Default::default());

        let mut framedata = VRFrameData {
            reflector_: Reflector::new(),
            left_proj: Heap::default(),
            left_view: Heap::default(),
            right_proj: Heap::default(),
            right_view: Heap::default(),
            pose: JS::from_ref(&*pose),
            timestamp: Cell::new(0.0),
            first_timestamp: Cell::new(0.0)
        };

        unsafe {
            framedata.left_proj.set(slice_to_array_buffer_view(global.get_cx(), &matrix));
            framedata.left_view.set(slice_to_array_buffer_view(global.get_cx(), &matrix));
            framedata.right_proj.set(slice_to_array_buffer_view(global.get_cx(), &matrix));
            framedata.right_view.set(slice_to_array_buffer_view(global.get_cx(), &matrix));
        }

        reflect_dom_object(box framedata,
                           global,
                           VRFrameDataBinding::Wrap)
    }

    pub fn Constructor(window: &Window) -> Fallible<Root<VRFrameData>> {
        Ok(VRFrameData::new(&window.global()))
    }
}


impl VRFrameData {
    #[allow(unsafe_code)]
    pub fn update(&self, data: &WebVRFrameData) {
        unsafe {
            update_array_buffer_view(self.left_proj.get(), &data.left_projection_matrix);
            update_array_buffer_view(self.left_view.get(), &data.left_view_matrix);
            update_array_buffer_view(self.right_proj.get(), &data.right_projection_matrix);
            update_array_buffer_view(self.right_view.get(), &data.right_view_matrix);
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
        NonZero::new(self.left_proj.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-leftviewmatrix
    unsafe fn LeftViewMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new(self.left_view.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-rightprojectionmatrix
    unsafe fn RightProjectionMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new(self.right_proj.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-rightviewmatrix
    unsafe fn RightViewMatrix(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new(self.right_view.get())
    }

    // https://w3c.github.io/webvr/#dom-vrframedata-pose
    fn Pose(&self) -> Root<VRPose> {
        Root::from_ref(&*self.pose)
    }
}
