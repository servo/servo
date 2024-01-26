/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::typedarray::{Float32, Float32Array};

use super::bindings::typedarrays::HeapTypedArray;
use crate::dom::bindings::codegen::Bindings::GamepadPoseBinding::GamepadPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct GamepadPose {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "mozjs"]
    position: HeapTypedArray<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    orientation: HeapTypedArray<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_vel: HeapTypedArray<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_vel: HeapTypedArray<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_acc: HeapTypedArray<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_acc: HeapTypedArray<Float32>,
}

// TODO: support gamepad discovery
#[allow(dead_code)]
impl GamepadPose {
    fn new_inherited() -> GamepadPose {
        GamepadPose {
            reflector_: Reflector::new(),
            position: HeapTypedArray::default(),
            orientation: HeapTypedArray::default(),
            linear_vel: HeapTypedArray::default(),
            angular_vel: HeapTypedArray::default(),
            linear_acc: HeapTypedArray::default(),
            angular_acc: HeapTypedArray::default(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<GamepadPose> {
        reflect_dom_object(Box::new(GamepadPose::new_inherited()), global)
    }
}

impl GamepadPoseMethods for GamepadPose {
    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-position
    fn GetPosition(&self, _cx: JSContext) -> Option<Float32Array> {
        self.position.internal_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-hasposition
    fn HasPosition(&self) -> bool {
        self.position.is_initialized()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-linearvelocity
    fn GetLinearVelocity(&self, _cx: JSContext) -> Option<Float32Array> {
        self.linear_vel.internal_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-linearacceleration
    fn GetLinearAcceleration(&self, _cx: JSContext) -> Option<Float32Array> {
        self.linear_acc.internal_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-orientation
    fn GetOrientation(&self, _cx: JSContext) -> Option<Float32Array> {
        self.orientation.internal_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-orientation
    fn HasOrientation(&self) -> bool {
        self.orientation.is_initialized()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-angularvelocity
    fn GetAngularVelocity(&self, _cx: JSContext) -> Option<Float32Array> {
        self.angular_vel.internal_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-angularacceleration
    fn GetAngularAcceleration(&self, _cx: JSContext) -> Option<Float32Array> {
        self.angular_acc.internal_to_option()
    }
}
