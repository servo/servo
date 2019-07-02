/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FakeXRDeviceBinding::{
    self, FakeXRDeviceMethods, FakeXRRigidTransformInit, FakeXRViewInit,
};
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use webvr_traits::{MockVRControlMsg, MockVRView, WebVRMsg};

#[dom_struct]
pub struct FakeXRDevice {
    reflector: Reflector,
}

impl FakeXRDevice {
    pub fn new_inherited() -> FakeXRDevice {
        FakeXRDevice {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<FakeXRDevice> {
        reflect_dom_object(
            Box::new(FakeXRDevice::new_inherited()),
            global,
            FakeXRDeviceBinding::Wrap,
        )
    }

    fn send_msg(&self, msg: MockVRControlMsg) {
        self.global()
            .as_window()
            .webvr_thread()
            .unwrap()
            .send(WebVRMsg::MessageMockDisplay(msg))
            .unwrap();
    }
}

pub fn get_views(views: &[FakeXRViewInit]) -> Fallible<(MockVRView, MockVRView)> {
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
        left.viewOffset.position.len() != 3 ||
        right.viewOffset.position.len() != 3
    {
        return Err(Error::Type("Incorrectly sized array".into()));
    }

    let mut proj_l = [0.; 16];
    let mut proj_r = [0.; 16];
    let v: Vec<_> = left.projectionMatrix.iter().map(|x| **x).collect();
    proj_l.copy_from_slice(&v);
    let v: Vec<_> = right.projectionMatrix.iter().map(|x| **x).collect();
    proj_r.copy_from_slice(&v);

    let mut offset_l = [0.; 3];
    let mut offset_r = [0.; 3];
    let v: Vec<_> = left.viewOffset.position.iter().map(|x| **x).collect();
    offset_l.copy_from_slice(&v);
    let v: Vec<_> = right.viewOffset.position.iter().map(|x| **x).collect();
    offset_r.copy_from_slice(&v);
    let left = MockVRView {
        projection: proj_l,
        offset: offset_l,
    };
    let right = MockVRView {
        projection: proj_r,
        offset: offset_r,
    };
    Ok((left, right))
}

pub fn get_origin(origin: &FakeXRRigidTransformInit) -> Fallible<([f32; 3], [f32; 4])> {
    if origin.position.len() != 3 || origin.orientation.len() != 4 {
        return Err(Error::Type("Incorrectly sized array".into()));
    }
    let mut p = [0.; 3];
    let mut o = [0.; 4];
    let v: Vec<_> = origin.position.iter().map(|x| **x).collect();
    p.copy_from_slice(&v[0..3]);
    let v: Vec<_> = origin.orientation.iter().map(|x| **x).collect();
    o.copy_from_slice(&v);

    Ok((p, o))
}

impl FakeXRDeviceMethods for FakeXRDevice {
    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn SetViews(&self, views: Vec<FakeXRViewInit>) -> Fallible<()> {
        let (left, right) = get_views(&views)?;
        self.send_msg(MockVRControlMsg::SetViews(left, right));
        Ok(())
    }

    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn SetViewerOrigin(
        &self,
        origin: &FakeXRRigidTransformInit,
        _emulated_position: bool,
    ) -> Fallible<()> {
        let (position, orientation) = get_origin(origin)?;
        self.send_msg(MockVRControlMsg::SetViewerPose(position, orientation));
        Ok(())
    }
}
