/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use webxr_api::{
    Handedness, InputId, MockDeviceMsg, MockInputMsg, SelectEvent, SelectKind, TargetRayMode,
};

use crate::dom::bindings::codegen::Bindings::FakeXRDeviceBinding::FakeXRRigidTransformInit;
use crate::dom::bindings::codegen::Bindings::FakeXRInputControllerBinding::FakeXRInputControllerMethods;
use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding::{
    XRHandedness, XRTargetRayMode,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::fakexrdevice::get_origin;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct FakeXRInputController {
    reflector: Reflector,
    #[ignore_malloc_size_of = "defined in ipc-channel"]
    #[no_trace]
    sender: IpcSender<MockDeviceMsg>,
    #[ignore_malloc_size_of = "defined in webxr-api"]
    #[no_trace]
    id: InputId,
}

impl FakeXRInputController {
    pub fn new_inherited(sender: IpcSender<MockDeviceMsg>, id: InputId) -> FakeXRInputController {
        FakeXRInputController {
            reflector: Reflector::new(),
            sender,
            id,
        }
    }

    pub fn new(
        global: &GlobalScope,
        sender: IpcSender<MockDeviceMsg>,
        id: InputId,
    ) -> DomRoot<FakeXRInputController> {
        reflect_dom_object(
            Box::new(FakeXRInputController::new_inherited(sender, id)),
            global,
        )
    }

    fn send_message(&self, msg: MockInputMsg) {
        let _ = self
            .sender
            .send(MockDeviceMsg::MessageInputSource(self.id, msg));
    }
}

impl FakeXRInputControllerMethods for FakeXRInputController {
    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-setpointerorigin>
    fn SetPointerOrigin(&self, origin: &FakeXRRigidTransformInit, _emulated: bool) -> Fallible<()> {
        self.send_message(MockInputMsg::SetPointerOrigin(Some(get_origin(origin)?)));
        Ok(())
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-setgriporigin>
    fn SetGripOrigin(&self, origin: &FakeXRRigidTransformInit, _emulated: bool) -> Fallible<()> {
        self.send_message(MockInputMsg::SetGripOrigin(Some(get_origin(origin)?)));
        Ok(())
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-cleargriporigin>
    fn ClearGripOrigin(&self) {
        self.send_message(MockInputMsg::SetGripOrigin(None))
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-disconnect>
    fn Disconnect(&self) {
        self.send_message(MockInputMsg::Disconnect)
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-reconnect>
    fn Reconnect(&self) {
        self.send_message(MockInputMsg::Reconnect)
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-startselection>
    fn StartSelection(&self) {
        self.send_message(MockInputMsg::TriggerSelect(
            SelectKind::Select,
            SelectEvent::Start,
        ))
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-endselection>
    fn EndSelection(&self) {
        self.send_message(MockInputMsg::TriggerSelect(
            SelectKind::Select,
            SelectEvent::End,
        ))
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-simulateselect>
    fn SimulateSelect(&self) {
        self.send_message(MockInputMsg::TriggerSelect(
            SelectKind::Select,
            SelectEvent::Select,
        ))
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-sethandedness>
    fn SetHandedness(&self, handedness: XRHandedness) {
        let h = match handedness {
            XRHandedness::None => Handedness::None,
            XRHandedness::Left => Handedness::Left,
            XRHandedness::Right => Handedness::Right,
        };
        self.send_message(MockInputMsg::SetHandedness(h));
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-settargetraymode>
    fn SetTargetRayMode(&self, target_ray_mode: XRTargetRayMode) {
        let t = match target_ray_mode {
            XRTargetRayMode::Gaze => TargetRayMode::Gaze,
            XRTargetRayMode::Tracked_pointer => TargetRayMode::TrackedPointer,
            XRTargetRayMode::Screen => TargetRayMode::Screen,
        };
        self.send_message(MockInputMsg::SetTargetRayMode(t));
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-setprofiles>
    fn SetProfiles(&self, profiles: Vec<DOMString>) {
        let t = profiles.into_iter().map(String::from).collect();
        self.send_message(MockInputMsg::SetProfiles(t));
    }
}
