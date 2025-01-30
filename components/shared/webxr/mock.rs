/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::DiscoveryAPI;
use crate::Display;
use crate::EntityType;
use crate::Error;
use crate::Floor;
use crate::Handedness;
use crate::Input;
use crate::InputId;
use crate::InputSource;
use crate::LeftEye;
use crate::Native;
use crate::Receiver;
use crate::RightEye;
use crate::SelectEvent;
use crate::SelectKind;
use crate::Sender;
use crate::TargetRayMode;
use crate::Triangle;
use crate::Viewer;
use crate::Viewport;
use crate::Visibility;

use euclid::{Point2D, Rect, RigidTransform3D, Transform3D};

#[cfg(feature = "ipc")]
use serde::{Deserialize, Serialize};

/// A trait for discovering mock XR devices
pub trait MockDiscoveryAPI<GL>: 'static {
    fn simulate_device_connection(
        &mut self,
        init: MockDeviceInit,
        receiver: Receiver<MockDeviceMsg>,
    ) -> Result<Box<dyn DiscoveryAPI<GL>>, Error>;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct MockDeviceInit {
    pub floor_origin: Option<RigidTransform3D<f32, Floor, Native>>,
    pub supports_inline: bool,
    pub supports_vr: bool,
    pub supports_ar: bool,
    pub viewer_origin: Option<RigidTransform3D<f32, Viewer, Native>>,
    pub views: MockViewsInit,
    pub supported_features: Vec<String>,
    pub world: Option<MockWorld>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct MockViewInit<Eye> {
    pub transform: RigidTransform3D<f32, Viewer, Eye>,
    pub projection: Transform3D<f32, Eye, Display>,
    pub viewport: Rect<i32, Viewport>,
    /// field of view values, in radians
    pub fov: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum MockViewsInit {
    Mono(MockViewInit<Viewer>),
    Stereo(MockViewInit<LeftEye>, MockViewInit<RightEye>),
}

#[derive(Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum MockDeviceMsg {
    SetViewerOrigin(Option<RigidTransform3D<f32, Viewer, Native>>),
    SetFloorOrigin(Option<RigidTransform3D<f32, Floor, Native>>),
    SetViews(MockViewsInit),
    AddInputSource(MockInputInit),
    MessageInputSource(InputId, MockInputMsg),
    VisibilityChange(Visibility),
    SetWorld(MockWorld),
    ClearWorld,
    Disconnect(Sender<()>),
    SetBoundsGeometry(Vec<Point2D<f32, Floor>>),
    SimulateResetPose,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct MockInputInit {
    pub source: InputSource,
    pub pointer_origin: Option<RigidTransform3D<f32, Input, Native>>,
    pub grip_origin: Option<RigidTransform3D<f32, Input, Native>>,
    pub supported_buttons: Vec<MockButton>,
}

#[derive(Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum MockInputMsg {
    SetHandedness(Handedness),
    SetTargetRayMode(TargetRayMode),
    SetProfiles(Vec<String>),
    SetPointerOrigin(Option<RigidTransform3D<f32, Input, Native>>),
    SetGripOrigin(Option<RigidTransform3D<f32, Input, Native>>),
    /// Note: SelectEvent::Select here refers to a complete Select event,
    /// not just the end event, i.e. it refers to
    /// https://immersive-web.github.io/webxr-test-api/#dom-fakexrinputcontroller-simulateselect
    TriggerSelect(SelectKind, SelectEvent),
    Disconnect,
    Reconnect,
    SetSupportedButtons(Vec<MockButton>),
    UpdateButtonState(MockButton),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct MockRegion {
    pub faces: Vec<Triangle>,
    pub ty: EntityType,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct MockWorld {
    pub regions: Vec<MockRegion>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum MockButtonType {
    Grip,
    Touchpad,
    Thumbstick,
    OptionalButton,
    OptionalThumbstick,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct MockButton {
    pub button_type: MockButtonType,
    pub pressed: bool,
    pub touched: bool,
    pub pressed_value: f32,
    pub x_value: f32,
    pub y_value: f32,
}
