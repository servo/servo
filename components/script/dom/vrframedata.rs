/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::VRFrameDataBinding;
use crate::dom::bindings::codegen::Bindings::VRFrameDataBinding::VRFrameDataMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::reduce_timing_resolution;
use crate::dom::vrpose::VRPose;
use crate::dom::window::Window;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::typedarray::{CreateWith, Float32Array};
use std::cell::Cell;
use std::ptr;
use std::ptr::NonNull;
use webvr_traits::WebVRFrameData;

#[dom_struct]
pub struct VRFrameData {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "mozjs"]
    left_proj: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    left_view: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    right_proj: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    right_view: Heap<*mut JSObject>,
    pose: Dom<VRPose>,
    timestamp: Cell<f64>,
    first_timestamp: Cell<f64>,
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
            first_timestamp: Cell::new(0.0),
        }
    }

    #[allow(unsafe_code)]
    fn new(global: &GlobalScope) -> DomRoot<VRFrameData> {
        let matrix = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0f32,
        ];
        let pose = VRPose::new(&global, &Default::default());

        let root = reflect_dom_object(
            Box::new(VRFrameData::new_inherited(&pose)),
            global,
            VRFrameDataBinding::Wrap,
        );
        let cx = global.get_cx();
        create_typed_array(cx, &matrix, &root.left_proj);
        create_typed_array(cx, &matrix, &root.left_view);
        create_typed_array(cx, &matrix, &root.right_proj);
        create_typed_array(cx, &matrix, &root.right_view);

        root
    }

    #[allow(non_snake_case)]
    pub fn Constructor(window: &Window) -> Fallible<DomRoot<VRFrameData>> {
        Ok(VRFrameData::new(&window.global()))
    }
}

/// FIXME(#22526) this should be in a better place
#[allow(unsafe_code)]
pub fn create_typed_array(cx: JSContext, src: &[f32], dst: &Heap<*mut JSObject>) {
    rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
    unsafe {
        let _ = Float32Array::create(*cx, CreateWith::Slice(src), array.handle_mut());
    }
    (*dst).set(array.get());
}

impl VRFrameData {
    #[allow(unsafe_code)]
    pub fn update(&self, data: &WebVRFrameData) {
        unsafe {
            let cx = self.global().get_cx();
            typedarray!(in(*cx) let left_proj_array: Float32Array = self.left_proj.get());
            if let Ok(mut array) = left_proj_array {
                array.update(&data.left_projection_matrix);
            }
            typedarray!(in(*cx) let left_view_array: Float32Array = self.left_view.get());
            if let Ok(mut array) = left_view_array {
                array.update(&data.left_view_matrix);
            }
            typedarray!(in(*cx) let right_proj_array: Float32Array = self.right_proj.get());
            if let Ok(mut array) = right_proj_array {
                array.update(&data.right_projection_matrix);
            }
            typedarray!(in(*cx) let right_view_array: Float32Array = self.right_view.get());
            if let Ok(mut array) = right_view_array {
                array.update(&data.right_view_matrix);
            }
            self.pose.update(&data.pose);
            self.timestamp.set(data.timestamp);
            if self.first_timestamp.get() == 0.0 {
                self.first_timestamp.set(data.timestamp);
            }
        }
    }
}

impl VRFrameDataMethods for VRFrameData {
    // https://w3c.github.io/webvr/#dom-vrframedata-timestamp
    fn Timestamp(&self) -> DOMHighResTimeStamp {
        reduce_timing_resolution(self.timestamp.get() - self.first_timestamp.get())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-leftprojectionmatrix
    fn LeftProjectionMatrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        unsafe { NonNull::new_unchecked(self.left_proj.get()) }
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-leftviewmatrix
    fn LeftViewMatrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        unsafe { NonNull::new_unchecked(self.left_view.get()) }
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-rightprojectionmatrix
    fn RightProjectionMatrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        unsafe { NonNull::new_unchecked(self.right_proj.get()) }
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrframedata-rightviewmatrix
    fn RightViewMatrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        unsafe { NonNull::new_unchecked(self.right_view.get()) }
    }

    // https://w3c.github.io/webvr/#dom-vrframedata-pose
    fn Pose(&self) -> DomRoot<VRPose> {
        DomRoot::from_ref(&*self.pose)
    }
}
