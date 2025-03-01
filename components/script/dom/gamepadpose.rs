/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::typedarray::{Float32, Float32Array};

use super::bindings::buffer_source::HeapBufferSource;
use crate::dom::bindings::codegen::Bindings::GamepadPoseBinding::GamepadPoseMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct GamepadPose {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "mozjs"]
    position: HeapBufferSource<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    orientation: HeapBufferSource<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_vel: HeapBufferSource<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_vel: HeapBufferSource<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    linear_acc: HeapBufferSource<Float32>,
    #[ignore_malloc_size_of = "mozjs"]
    angular_acc: HeapBufferSource<Float32>,
}

// TODO: support gamepad discovery
#[allow(dead_code)]
impl GamepadPose {
    fn new_inherited() -> GamepadPose {
        GamepadPose {
            reflector_: Reflector::new(),
            position: HeapBufferSource::default(),
            orientation: HeapBufferSource::default(),
            linear_vel: HeapBufferSource::default(),
            angular_vel: HeapBufferSource::default(),
            linear_acc: HeapBufferSource::default(),
            angular_acc: HeapBufferSource::default(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<GamepadPose> {
        reflect_dom_object(Box::new(GamepadPose::new_inherited()), global, can_gc)
    }
}

impl GamepadPoseMethods<crate::DomTypeHolder> for GamepadPose {
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
