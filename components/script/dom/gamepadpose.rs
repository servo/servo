/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::typedarray::Float32Array;

use super::bindings::typedarrays::HeapFloat32Array;
use crate::dom::bindings::codegen::Bindings::GamepadPoseBinding::GamepadPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct GamepadPose {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "mozjs"]
    position: HeapFloat32Array,
    #[ignore_malloc_size_of = "mozjs"]
    orientation: HeapFloat32Array,
    #[ignore_malloc_size_of = "mozjs"]
    linear_vel: HeapFloat32Array,
    #[ignore_malloc_size_of = "mozjs"]
    angular_vel: HeapFloat32Array,
    #[ignore_malloc_size_of = "mozjs"]
    linear_acc: HeapFloat32Array,
    #[ignore_malloc_size_of = "mozjs"]
    angular_acc: HeapFloat32Array,
}

// TODO: support gamepad discovery
#[allow(dead_code)]
impl GamepadPose {
    fn new_inherited() -> GamepadPose {
        GamepadPose {
            reflector_: Reflector::new(),
            position: HeapFloat32Array::default(),
            orientation: HeapFloat32Array::default(),
            linear_vel: HeapFloat32Array::default(),
            angular_vel: HeapFloat32Array::default(),
            linear_acc: HeapFloat32Array::default(),
            angular_acc: HeapFloat32Array::default(),
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
