/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::FakeXRDeviceBinding::{
    self, FakeXRDeviceMethods, FakeXRRigidTransformInit, FakeXRViewInit,
};
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use euclid::{Point2D, Rect, Size2D};
use euclid::{RigidTransform3D, Rotation3D, Transform3D, Vector3D};
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use std::rc::Rc;
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

    pub fn disconnect(&self, sender: IpcSender<()>) {
        let _ = self.sender.send(MockDeviceMsg::Disconnect(sender));
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
    let proj_l = Transform3D::from_array(proj_l);
    let v: Vec<_> = right.projectionMatrix.iter().map(|x| **x).collect();
    proj_r.copy_from_slice(&v);
    let proj_r = Transform3D::from_array(proj_r);

    // spec defines offsets as origins, but mock API expects the inverse transform
    let offset_l = get_origin(&left.viewOffset)?.inverse();
    let offset_r = get_origin(&right.viewOffset)?.inverse();

    let size_l = Size2D::new(views[0].resolution.width, views[0].resolution.height);
    let size_r = Size2D::new(views[1].resolution.width, views[1].resolution.height);

    let origin_l = Point2D::new(0, 0);
    let origin_r = Point2D::new(size_l.width, 0);

    let viewport_l = Rect::new(origin_l, size_l);
    let viewport_r = Rect::new(origin_r, size_r);

    let left = View {
        projection: proj_l,
        transform: offset_l,
        viewport: viewport_l,
    };
    let right = View {
        projection: proj_r,
        transform: offset_r,
        viewport: viewport_r,
    };
    Ok(Views::Stereo(left, right))
}

pub fn get_origin<T, U>(
    origin: &FakeXRRigidTransformInit,
) -> Fallible<RigidTransform3D<f32, T, U>> {
    if origin.position.len() != 3 || origin.orientation.len() != 4 {
        return Err(Error::Type("Incorrectly sized array".into()));
    }
    let p = Vector3D::new(
        *origin.position[0],
        *origin.position[1],
        *origin.position[2],
    );
    let o = Rotation3D::unit_quaternion(
        *origin.orientation[0],
        *origin.orientation[1],
        *origin.orientation[2],
        *origin.orientation[3],
    );

    Ok(RigidTransform3D::new(o, p))
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

    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn Disconnect(&self) -> Rc<Promise> {
        let global = self.global();
        let p = Promise::new(&global);
        let mut trusted = Some(TrustedPromise::new(p.clone()));
        let (task_source, canceller) = global
            .as_window()
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        ROUTER.add_route(
            receiver.to_opaque(),
            Box::new(move |_| {
                let trusted = trusted
                    .take()
                    .expect("disconnect callback called multiple times");
                let _ = task_source.queue_with_canceller(trusted.resolve_task(()), &canceller);
            }),
        );
        self.disconnect(sender);
        p
    }
}
