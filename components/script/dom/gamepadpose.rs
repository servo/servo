/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::typedarray::{Float32, Float32Array};

use super::bindings::buffer_source_types::HeapBufferSourceTypes;
use crate::dom::bindings::codegen::Bindings::GamepadPoseBinding::GamepadPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct GamepadPose {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "mozjs"]
    position: HeapBufferSourceTypes<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    orientation: HeapBufferSourceTypes<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_vel: HeapBufferSourceTypes<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_vel: HeapBufferSourceTypes<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_acc: HeapBufferSourceTypes<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_acc: HeapBufferSourceTypes<Float32>,
}

// TODO: support gamepad discovery
#[allow(dead_code)]
impl GamepadPose {
    fn new_inherited() -> GamepadPose {
        GamepadPose {
            reflector_: Reflector::new(),
            position: HeapBufferSourceTypes::default(),
            orientation: HeapBufferSourceTypes::default(),
            linear_vel: HeapBufferSourceTypes::default(),
            angular_vel: HeapBufferSourceTypes::default(),
            linear_acc: HeapBufferSourceTypes::default(),
            angular_acc: HeapBufferSourceTypes::default(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<GamepadPose> {
        reflect_dom_object(Box::new(GamepadPose::new_inherited()), global)
    }
}

impl GamepadPoseMethods for GamepadPose {
    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-position
    fn GetPosition(&self, _cx: JSContext) -> Option<Float32Array> {
        self.position.buffer_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-hasposition
    fn HasPosition(&self) -> bool {
        self.position.is_initialized()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-linearvelocity
    fn GetLinearVelocity(&self, _cx: JSContext) -> Option<Float32Array> {
        self.linear_vel.buffer_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-linearacceleration
    fn GetLinearAcceleration(&self, _cx: JSContext) -> Option<Float32Array> {
        self.linear_acc.buffer_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-orientation
    fn GetOrientation(&self, _cx: JSContext) -> Option<Float32Array> {
        self.orientation.buffer_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-orientation
    fn HasOrientation(&self) -> bool {
        self.orientation.is_initialized()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-angularvelocity
    fn GetAngularVelocity(&self, _cx: JSContext) -> Option<Float32Array> {
        self.angular_vel.buffer_to_option()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepadpose-angularacceleration
    fn GetAngularAcceleration(&self, _cx: JSContext) -> Option<Float32Array> {
        self.angular_acc.buffer_to_option()
    }
}
