/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::GamepadPoseBinding::GamepadPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::typedarray::{CreateWith, Float32Array};
use std::ptr;
use std::ptr::NonNull;

#[dom_struct]
pub struct GamepadPose {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "mozjs"]
    position: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    orientation: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_vel: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_vel: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_acc: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_acc: Heap<*mut JSObject>,
}

// TODO: support gamepad discovery
#[allow(dead_code)]
#[allow(unsafe_code)]
fn update_or_create_typed_array(cx: JSContext, src: Option<&[f32]>, dst: &Heap<*mut JSObject>) {
    match src {
        Some(data) => {
            if dst.get().is_null() {
                rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
                let _ = unsafe {
                    Float32Array::create(*cx, CreateWith::Slice(data), array.handle_mut())
                };
                (*dst).set(array.get());
            } else {
                typedarray!(in(*cx) let array: Float32Array = dst.get());
                if let Ok(mut array) = array {
                    unsafe { array.update(data) };
                }
            }
        },
        None => {
            if !dst.get().is_null() {
                dst.set(ptr::null_mut());
            }
        },
    }
}

#[inline]
#[allow(unsafe_code)]
fn heap_to_option(heap: &Heap<*mut JSObject>) -> Option<NonNull<JSObject>> {
    let js_object = heap.get();
    if js_object.is_null() {
        None
    } else {
        unsafe { Some(NonNull::new_unchecked(js_object)) }
    }
}

// TODO: support gamepad discovery
#[allow(dead_code)]
impl GamepadPose {
    fn new_inherited() -> GamepadPose {
        GamepadPose {
            reflector_: Reflector::new(),
            position: Heap::default(),
            orientation: Heap::default(),
            linear_vel: Heap::default(),
            angular_vel: Heap::default(),
            linear_acc: Heap::default(),
            angular_acc: Heap::default(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<GamepadPose> {
        reflect_dom_object(Box::new(GamepadPose::new_inherited()), global)
    }
}

impl GamepadPoseMethods for GamepadPose {
    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-position
    fn GetPosition(&self, _cx: JSContext) -> Option<NonNull<JSObject>> {
        heap_to_option(&self.position)
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-hasposition
    fn HasPosition(&self) -> bool {
        !self.position.get().is_null()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-linearvelocity
    fn GetLinearVelocity(&self, _cx: JSContext) -> Option<NonNull<JSObject>> {
        heap_to_option(&self.linear_vel)
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-linearacceleration
    fn GetLinearAcceleration(&self, _cx: JSContext) -> Option<NonNull<JSObject>> {
        heap_to_option(&self.linear_acc)
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-orientation
    fn GetOrientation(&self, _cx: JSContext) -> Option<NonNull<JSObject>> {
        heap_to_option(&self.orientation)
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-orientation
    fn HasOrientation(&self) -> bool {
        !self.orientation.get().is_null()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-angularvelocity
    fn GetAngularVelocity(&self, _cx: JSContext) -> Option<NonNull<JSObject>> {
        heap_to_option(&self.angular_vel)
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-angularacceleration
    fn GetAngularAcceleration(&self, _cx: JSContext) -> Option<NonNull<JSObject>> {
        heap_to_option(&self.angular_acc)
    }
}
