/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FakeXRDeviceBinding::{
    self, FakeXRDeviceMethods, FakeXRRigidTransformInit, FakeXRViewInit,
};
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use euclid::{TypedRigidTransform3D, TypedRotation3D, TypedTransform3D, TypedVector3D};
use ipc_channel::ipc::IpcSender;
use webxr_api::{MockDeviceMsg, View, Views};

#[dom_struct]
pub struct FakeXRDevice {
    reflector: Reflector,
    #[ignore_malloc_size_of = "defined in ipc-channel"]
    sender: IpcSender<MockDeviceMsg>,
}

impl FakeXRDevice {
    pub fn new_inherited(sender: IpcSender<MockDeviceMsg>) -> FakeXRDevice {
        FakeXRDevice {
            reflector: Reflector::new(),
            sender,
        }
    }

    pub fn new(global: &GlobalScope, sender: IpcSender<MockDeviceMsg>) -> DomRoot<FakeXRDevice> {
        reflect_dom_object(
            Box::new(FakeXRDevice::new_inherited(sender)),
            global,
            FakeXRDeviceBinding::Wrap,
        )
    }
}

pub fn get_views(views: &[FakeXRViewInit]) -> Fallible<Views> {
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
    let proj_l = TypedTransform3D::from_array(proj_l);
    let v: Vec<_> = right.projectionMatrix.iter().map(|x| **x).collect();
    proj_r.copy_from_slice(&v);
    let proj_r = TypedTransform3D::from_array(proj_r);

    // spec defines offsets as origins, but mock API expects the inverse transform
    let offset_l = get_origin(&left.viewOffset)?.inverse();
    let offset_r = get_origin(&right.viewOffset)?.inverse();

    let left = View {
        projection: proj_l,
        transform: offset_l,
    };
    let right = View {
        projection: proj_r,
        transform: offset_r,
    };
    Ok(Views::Stereo(left, right))
}

pub fn get_origin<T, U>(
    origin: &FakeXRRigidTransformInit,
) -> Fallible<TypedRigidTransform3D<f32, T, U>> {
    if origin.position.len() != 3 || origin.orientation.len() != 4 {
        return Err(Error::Type("Incorrectly sized array".into()));
    }
    let p = TypedVector3D::new(
        *origin.position[0],
        *origin.position[1],
        *origin.position[2],
    );
    let o = TypedRotation3D::unit_quaternion(
        *origin.orientation[0],
        *origin.orientation[1],
        *origin.orientation[2],
        *origin.orientation[3],
    );

    Ok(TypedRigidTransform3D::new(o, p))
}

impl FakeXRDeviceMethods for FakeXRDevice {
    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn SetViews(&self, views: Vec<FakeXRViewInit>) -> Fallible<()> {
        let _ = self
            .sender
            .send(MockDeviceMsg::SetViews(get_views(&views)?));
        Ok(())
    }

    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn SetViewerOrigin(
        &self,
        origin: &FakeXRRigidTransformInit,
        _emulated_position: bool,
    ) -> Fallible<()> {
        let _ = self
            .sender
            .send(MockDeviceMsg::SetViewerOrigin(get_origin(origin)?));
        Ok(())
    }
}
