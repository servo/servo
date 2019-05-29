/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FakeXRDeviceControllerBinding::{
    self, FakeXRDeviceControllerMethods, FakeXRViewInit,
};
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webvr_traits::{MockVRControlMsg, WebVREyeParameters, WebVRMsg};

#[dom_struct]
pub struct FakeXRDeviceController {
    reflector: Reflector,
}

impl FakeXRDeviceController {
    pub fn new_inherited() -> FakeXRDeviceController {
        FakeXRDeviceController {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<FakeXRDeviceController> {
        reflect_dom_object(
            Box::new(FakeXRDeviceController::new_inherited()),
            global,
            FakeXRDeviceControllerBinding::Wrap,
        )
    }

    pub fn send_msg(&self, msg: MockVRControlMsg) {
        self.global()
            .as_window()
            .webvr_thread()
            .unwrap()
            .send(WebVRMsg::MessageMockDisplay(msg));
    }
}

impl FakeXRDeviceControllerMethods for FakeXRDeviceController {
    // check-tidy: no specs after this line
    fn SetViews(&self, views: Vec<FakeXRViewInit>) -> Fallible<()> {
        if views.len() != 2 {
            return Err(Error::NotSupported);
        }

        let (left, right) = match (views[0].eye, views[1].eye) {
            (XREye::Left, XREye::Right) => (&views[0], &views[1]),
            (XREye::Right, XREye::Left) => (&views[1], &views[0]),
            _ => return Err(Error::NotSupported),
        };

        if left.projectionMatrix.len() != 16 ||
            right.projectionMatrix.len() != 16 ||
            left.viewOffset.position.len() != 4 ||
            right.viewOffset.position.len() != 4
        {
            return Err(Error::Type("Incorrectly sized matrix".into()));
        }

        let mut proj_l = [0.; 16];
        let mut proj_r = [0.; 16];
        let v: Vec<_> = left.projectionMatrix.iter().map(|x| **x).collect();
        proj_l.copy_from_slice(&v);
        let v: Vec<_> = right.projectionMatrix.iter().map(|x| **x).collect();
        proj_r.copy_from_slice(&v);

        let mut params_l = WebVREyeParameters::default();
        let mut params_r = WebVREyeParameters::default();
        let v: Vec<_> = left.viewOffset.position.iter().map(|x| **x).collect();
        params_l.offset.copy_from_slice(&v);
        let v: Vec<_> = right.viewOffset.position.iter().map(|x| **x).collect();
        params_r.offset.copy_from_slice(&v);

        self.send_msg(MockVRControlMsg::SetProjectionMatrices(proj_l, proj_r));
        self.send_msg(MockVRControlMsg::SetEyeParameters(params_l, params_r));
        Ok(())
    }
}
