/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VRPoseBinding;
use dom::bindings::codegen::Bindings::VRPoseBinding::VRPoseMethods;
use dom::bindings::conversions::{slice_to_array_buffer_view, update_array_buffer_view};
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use js::jsapi::{Heap, JSContext, JSObject};
use std::ptr;
use webvr_traits::webvr;

#[dom_struct]
pub struct VRPose {
    reflector_: Reflector,
    position: DOMRefCell<Heap<*mut JSObject>>,
    orientation: DOMRefCell<Heap<*mut JSObject>>,
    linear_vel: DOMRefCell<Heap<*mut JSObject>>,
    angular_vel: DOMRefCell<Heap<*mut JSObject>>,
    linear_acc: DOMRefCell<Heap<*mut JSObject>>,
    angular_acc: DOMRefCell<Heap<*mut JSObject>>
}

#[allow(unsafe_code)]
unsafe fn update_or_create_typed_array(cx: *mut JSContext,
                      src: Option<&[f32]>,
                      dst: &DOMRefCell<Heap<*mut JSObject>>) {
    let mut dst = dst.borrow_mut();
    match src {
        Some(ref data) => {
            if dst.get().is_null() {
                dst.set(slice_to_array_buffer_view(cx, &data));
            } else {
                update_array_buffer_view(dst.get(), &data);
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
fn heap_to_option(heap: &DOMRefCell<Heap<*mut JSObject>>) -> Option<NonZero<*mut JSObject>> {
    let js_object = heap.borrow_mut().get();
    if js_object.is_null() {
        None
    } else {
        unsafe {
            Some(NonZero::new(js_object))
        }
    }
}

impl VRPose {
    fn new_inherited() -> VRPose {
        VRPose {
            reflector_: Reflector::new(),
            position: DOMRefCell::new(Heap::default()),
            orientation: DOMRefCell::new(Heap::default()),
            linear_vel: DOMRefCell::new(Heap::default()),
            angular_vel: DOMRefCell::new(Heap::default()),
            linear_acc: DOMRefCell::new(Heap::default()),
            angular_acc: DOMRefCell::new(Heap::default())
        }
    }

    pub fn new(global: &GlobalScope, pose: &webvr::VRPose) -> Root<VRPose> {
        let root = reflect_dom_object(box VRPose::new_inherited(),
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
