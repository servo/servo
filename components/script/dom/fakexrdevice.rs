/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use euclid::{Point2D, Point3D, Rect, RigidTransform3D, Rotation3D, Size2D, Transform3D, Vector3D};
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use webxr_api::{
    EntityType, Handedness, InputId, InputSource, MockDeviceMsg, MockInputInit, MockRegion,
    MockViewInit, MockViewsInit, MockWorld, TargetRayMode, Triangle, Visibility,
};

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::FakeXRDeviceBinding::{
    FakeXRDeviceMethods, FakeXRRegionType, FakeXRRigidTransformInit, FakeXRViewInit,
    FakeXRWorldInit,
};
use crate::dom::bindings::codegen::Bindings::FakeXRInputControllerBinding::FakeXRInputSourceInit;
use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding::{
    XRHandedness, XRTargetRayMode,
};
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRVisibilityState;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::XREye;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::fakexrinputcontroller::FakeXRInputController;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct FakeXRDevice {
    reflector: Reflector,
    #[ignore_malloc_size_of = "defined in ipc-channel"]
    #[no_trace]
    sender: IpcSender<MockDeviceMsg>,
    #[ignore_malloc_size_of = "defined in webxr-api"]
    #[no_trace]
    next_input_id: Cell<InputId>,
}

impl FakeXRDevice {
    pub fn new_inherited(sender: IpcSender<MockDeviceMsg>) -> FakeXRDevice {
        FakeXRDevice {
            reflector: Reflector::new(),
            sender,
            next_input_id: Cell::new(InputId(0)),
        }
    }

    pub fn new(global: &GlobalScope, sender: IpcSender<MockDeviceMsg>) -> DomRoot<FakeXRDevice> {
        reflect_dom_object(Box::new(FakeXRDevice::new_inherited(sender)), global)
    }

    pub fn disconnect(&self, sender: IpcSender<()>) {
        let _ = self.sender.send(MockDeviceMsg::Disconnect(sender));
    }
}

pub fn view<Eye>(view: &FakeXRViewInit) -> Fallible<MockViewInit<Eye>> {
    if view.projectionMatrix.len() != 16 || view.viewOffset.position.len() != 3 {
        return Err(Error::Type("Incorrectly sized array".into()));
    }

    let mut proj = [0.; 16];
    let v: Vec<_> = view.projectionMatrix.iter().map(|x| **x).collect();
    proj.copy_from_slice(&v);
    let projection = Transform3D::from_array(proj);

    // spec defines offsets as origins, but mock API expects the inverse transform
    let transform = get_origin(&view.viewOffset)?.inverse();

    let size = Size2D::new(view.resolution.width, view.resolution.height);
    let origin = match view.eye {
        XREye::Right => Point2D::new(size.width, 0),
        _ => Point2D::new(0, 0),
    };
    let viewport = Rect::new(origin, size);

    let fov = view.fieldOfView.as_ref().map(|fov| {
        (
            fov.leftDegrees.to_radians(),
            fov.rightDegrees.to_radians(),
            fov.upDegrees.to_radians(),
            fov.downDegrees.to_radians(),
        )
    });

    Ok(MockViewInit {
        projection,
        transform,
        viewport,
        fov,
    })
}

pub fn get_views(views: &[FakeXRViewInit]) -> Fallible<MockViewsInit> {
    match views.len() {
        1 => Ok(MockViewsInit::Mono(view(&views[0])?)),
        2 => {
            let (left, right) = match (views[0].eye, views[1].eye) {
                (XREye::Left, XREye::Right) => (&views[0], &views[1]),
                (XREye::Right, XREye::Left) => (&views[1], &views[0]),
                _ => return Err(Error::NotSupported),
            };
            Ok(MockViewsInit::Stereo(view(left)?, view(right)?))
        },
        _ => Err(Error::NotSupported),
    }
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

pub fn get_point<T>(pt: &DOMPointInit) -> Point3D<f32, T> {
    Point3D::new(pt.x / pt.w, pt.y / pt.w, pt.z / pt.w).cast()
}

pub fn get_world(world: &FakeXRWorldInit) -> Fallible<MockWorld> {
    let regions = world
        .hitTestRegions
        .iter()
        .map(|region| {
            let ty = region.type_.into();
            let faces = region
                .faces
                .iter()
                .map(|face| {
                    if face.vertices.len() != 3 {
                        return Err(Error::Type(
                            "Incorrectly sized array for triangle list".into(),
                        ));
                    }

                    Ok(Triangle {
                        first: get_point(&face.vertices[0]),
                        second: get_point(&face.vertices[1]),
                        third: get_point(&face.vertices[2]),
                    })
                })
                .collect::<Fallible<Vec<_>>>()?;
            Ok(MockRegion { faces, ty })
        })
        .collect::<Fallible<Vec<_>>>()?;

    Ok(MockWorld { regions })
}

impl From<FakeXRRegionType> for EntityType {
    fn from(x: FakeXRRegionType) -> Self {
        match x {
            FakeXRRegionType::Point => EntityType::Point,
            FakeXRRegionType::Plane => EntityType::Plane,
            FakeXRRegionType::Mesh => EntityType::Mesh,
        }
    }
}

impl FakeXRDeviceMethods for FakeXRDevice {
    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn SetViews(&self, views: Vec<FakeXRViewInit>) -> Fallible<()> {
        let _ = self
            .sender
            .send(MockDeviceMsg::SetViews(get_views(&views)?));
        Ok(())
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-setviewerorigin>
    fn SetViewerOrigin(
        &self,
        origin: &FakeXRRigidTransformInit,
        _emulated_position: bool,
    ) -> Fallible<()> {
        let _ = self
            .sender
            .send(MockDeviceMsg::SetViewerOrigin(Some(get_origin(origin)?)));
        Ok(())
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-clearviewerorigin>
    fn ClearViewerOrigin(&self) {
        let _ = self.sender.send(MockDeviceMsg::SetViewerOrigin(None));
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-clearfloororigin>
    fn ClearFloorOrigin(&self) {
        let _ = self.sender.send(MockDeviceMsg::SetFloorOrigin(None));
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-setfloororigin>
    fn SetFloorOrigin(&self, origin: &FakeXRRigidTransformInit) -> Fallible<()> {
        let _ = self
            .sender
            .send(MockDeviceMsg::SetFloorOrigin(Some(get_origin(origin)?)));
        Ok(())
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-clearworld>
    fn ClearWorld(&self) {
        let _ = self.sender.send(MockDeviceMsg::ClearWorld);
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-setworld>
    fn SetWorld(&self, world: &FakeXRWorldInit) -> Fallible<()> {
        let _ = self.sender.send(MockDeviceMsg::SetWorld(get_world(world)?));
        Ok(())
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-simulatevisibilitychange>
    fn SimulateVisibilityChange(&self, v: XRVisibilityState) {
        let v = match v {
            XRVisibilityState::Visible => Visibility::Visible,
            XRVisibilityState::Visible_blurred => Visibility::VisibleBlurred,
            XRVisibilityState::Hidden => Visibility::Hidden,
        };
        let _ = self.sender.send(MockDeviceMsg::VisibilityChange(v));
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-simulateinputsourceconnection>
    fn SimulateInputSourceConnection(
        &self,
        init: &FakeXRInputSourceInit,
    ) -> Fallible<DomRoot<FakeXRInputController>> {
        let id = self.next_input_id.get();
        self.next_input_id.set(InputId(id.0 + 1));

        let handedness = init.handedness.into();
        let target_ray_mode = init.targetRayMode.into();

        let pointer_origin = Some(get_origin(&init.pointerOrigin)?);

        let grip_origin = if let Some(ref g) = init.gripOrigin {
            Some(get_origin(g)?)
        } else {
            None
        };

        let profiles = init.profiles.iter().cloned().map(String::from).collect();

        // XXXManishearth deal with supportedButtons and selection*

        let source = InputSource {
            handedness,
            target_ray_mode,
            id,
            supports_grip: true,
            profiles,
            hand_support: None,
        };

        let init = MockInputInit {
            source,
            pointer_origin,
            grip_origin,
        };

        let global = self.global();
        let _ = self.sender.send(MockDeviceMsg::AddInputSource(init));

        let controller = FakeXRInputController::new(&global, self.sender.clone(), id);

        Ok(controller)
    }

    /// <https://immersive-web.github.io/webxr-test-api/#dom-fakexrdevice-disconnect>
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

impl From<XRHandedness> for Handedness {
    fn from(h: XRHandedness) -> Self {
        match h {
            XRHandedness::None => Handedness::None,
            XRHandedness::Left => Handedness::Left,
            XRHandedness::Right => Handedness::Right,
        }
    }
}

impl From<XRTargetRayMode> for TargetRayMode {
    fn from(t: XRTargetRayMode) -> Self {
        match t {
            XRTargetRayMode::Gaze => TargetRayMode::Gaze,
            XRTargetRayMode::Tracked_pointer => TargetRayMode::TrackedPointer,
            XRTargetRayMode::Screen => TargetRayMode::Screen,
        }
    }
}
