/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FakeXRInputControllerBinding::{self};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use webxr_api::{InputId, MockDeviceMsg};

#[dom_struct]
pub struct FakeXRInputController {
    reflector: Reflector,
    #[ignore_malloc_size_of = "defined in ipc-channel"]
    sender: IpcSender<MockDeviceMsg>,
    #[ignore_malloc_size_of = "defined in webxr-api"]
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
            FakeXRInputControllerBinding::Wrap,
        )
    }
}
