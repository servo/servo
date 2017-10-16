/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::VRPoseBinding;
use dom::bindings::codegen::Bindings::VRPoseBinding::VRPoseMethods;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use js::typedarray::{Float32Array, CreateWith};
use std::ptr;
use webvr_traits::webvr;

#[dom_struct]
pub struct VRPose {
    reflector_: Reflector,
    position: Heap<*mut JSObject>,
    orientation: Heap<*mut JSObject>,
    linear_vel: Heap<*mut JSObject>,
    angular_vel: Heap<*mut JSObject>,
    linear_acc: Heap<*mut JSObject>,
    angular_acc: Heap<*mut JSObject>,
}

#[allow(unsafe_code)]
unsafe fn update_or_create_typed_array(cx: *mut JSContext,
                      src: Option<&[f32]>,
                      dst: &Heap<*mut JSObject>) {
    match src {
        Some(data) => {
            if dst.get().is_null() {
                rooted!(in (cx) let mut array = ptr::null_mut());
                let _ = Float32Array::create(cx, CreateWith::Slice(data), array.handle_mut());
                (*dst).set(array.get());
            } else {
                typedarray!(in(cx) let array: Float32Array = dst.get());
                if let Ok(mut array) = array {
                    array.update(data);
                }
            }
        },
        None => {
            if !dst.get().is_null() {
                dst.set(ptr::null_mut());
            }
        }
    }
}

#[inline]
#[allow(unsafe_code)]
fn heap_to_option(heap: &Heap<*mut JSObject>) -> Option<NonZero<*mut JSObject>> {
    let js_object = heap.get();
    if js_object.is_null() {
        None
    } else {
        unsafe {
            Some(NonZero::new_unchecked(js_object))
        }
    }
}

impl VRPose {
    fn new_inherited() -> VRPose {
        VRPose {
            reflector_: Reflector::new(),
            position: Heap::default(),
            orientation: Heap::default(),
            linear_vel: Heap::default(),
            angular_vel: Heap::default(),
            linear_acc: Heap::default(),
            angular_acc: Heap::default(),
        }
    }

    pub fn new(global: &GlobalScope, pose: &webvr::VRPose) -> DomRoot<VRPose> {
        let root = reflect_dom_object(Box::new(VRPose::new_inherited()),
                                      global,
                                      VRPoseBinding::Wrap);
        root.update(&pose);
        root
    }

    #[allow(unsafe_code)]
    pub fn update(&self, pose: &webvr::VRPose) {
        let cx = self.global().get_cx();
        unsafe {
            update_or_create_typed_array(cx, pose.position.as_ref().map(|v| &v[..]), &self.position);
            update_or_create_typed_array(cx, pose.orientation.as_ref().map(|v| &v[..]), &self.orientation);
            update_or_create_typed_array(cx, pose.linear_velocity.as_ref().map(|v| &v[..]), &self.linear_vel);
            update_or_create_typed_array(cx, pose.angular_velocity.as_ref().map(|v| &v[..]), &self.angular_vel);
            update_or_create_typed_array(cx, pose.linear_acceleration.as_ref().map(|v| &v[..]), &self.linear_acc);
            update_or_create_typed_array(cx, pose.angular_acceleration.as_ref().map(|v| &v[..]), &self.angular_acc);
        }
    }
}

impl VRPoseMethods for VRPose {
    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrpose-position
    unsafe fn GetPosition(&self, _cx: *mut JSContext) -> Option<NonZero<*mut JSObject>> {
        heap_to_option(&self.position)
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrpose-linearvelocity
    unsafe fn GetLinearVelocity(&self, _cx: *mut JSContext) -> Option<NonZero<*mut JSObject>> {
        heap_to_option(&self.linear_vel)
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrpose-linearacceleration
    unsafe fn GetLinearAcceleration(&self, _cx: *mut JSContext) -> Option<NonZero<*mut JSObject>> {
        heap_to_option(&self.linear_acc)
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrpose-orientation
    unsafe fn GetOrientation(&self, _cx: *mut JSContext) -> Option<NonZero<*mut JSObject>> {
        heap_to_option(&self.orientation)
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrpose-angularvelocity
    unsafe fn GetAngularVelocity(&self, _cx: *mut JSContext) -> Option<NonZero<*mut JSObject>> {
        heap_to_option(&self.angular_vel)
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/webvr/#dom-vrpose-angularacceleration
    unsafe fn GetAngularAcceleration(&self, _cx: *mut JSContext) -> Option<NonZero<*mut JSObject>> {
        heap_to_option(&self.angular_acc)
    }
}
