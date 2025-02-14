/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This crate defines the Rust API for WebXR. It is implemented by the `webxr` crate.

mod device;
mod error;
mod events;
mod frame;
mod hand;
mod hittest;
mod input;
mod layer;
mod mock;
mod registry;
mod session;
mod space;
pub mod util;
mod view;

#[cfg(not(feature = "ipc"))]
pub use std::sync::mpsc::{RecvTimeoutError, WebXrReceiver, WebXrSender};
#[cfg(feature = "ipc")]
use std::thread;
use std::time::Duration;

pub use device::{DeviceAPI, DiscoveryAPI};
pub use error::Error;
pub use events::{Event, EventBuffer, Visibility};
pub use frame::{Frame, FrameUpdateEvent, ViewerPose};
pub use hand::{Finger, FingerJoint, Hand, HandSpace, Joint, JointFrame};
pub use hittest::{
    EntityType, EntityTypes, HitTestId, HitTestResult, HitTestSource, HitTestSpace, Ray, Triangle,
};
pub use input::{
    Handedness, InputFrame, InputId, InputSource, SelectEvent, SelectKind, TargetRayMode,
};
#[cfg(feature = "ipc")]
pub use ipc_channel::ipc::channel as webxr_channel;
#[cfg(feature = "ipc")]
pub use ipc_channel::ipc::IpcReceiver as WebXrReceiver;
#[cfg(feature = "ipc")]
pub use ipc_channel::ipc::IpcSender as WebXrSender;
pub use layer::{
    ContextId, GLContexts, GLTypes, LayerGrandManager, LayerGrandManagerAPI, LayerId, LayerInit,
    LayerLayout, LayerManager, LayerManagerAPI, LayerManagerFactory, SubImage, SubImages,
};
pub use mock::{
    MockButton, MockButtonType, MockDeviceInit, MockDeviceMsg, MockDiscoveryAPI, MockInputInit,
    MockInputMsg, MockRegion, MockViewInit, MockViewsInit, MockWorld,
};
pub use registry::{MainThreadRegistry, MainThreadWaker, Registry};
pub use session::{
    EnvironmentBlendMode, MainThreadSession, Quitter, Session, SessionBuilder, SessionId,
    SessionInit, SessionMode, SessionThread,
};
pub use space::{ApiSpace, BaseSpace, Space};
pub use view::{
    Capture, CubeBack, CubeBottom, CubeLeft, CubeRight, CubeTop, Display, Floor, Input, LeftEye,
    Native, RightEye, SomeEye, View, Viewer, Viewport, Viewports, Views, CUBE_BACK, CUBE_BOTTOM,
    CUBE_LEFT, CUBE_RIGHT, CUBE_TOP, LEFT_EYE, RIGHT_EYE, VIEWER,
};

#[cfg(not(feature = "ipc"))]
pub fn webxr_channel<T>() -> Result<(WebXrWebXrSender<T>, WebXrWebXrReceiver<T>), ()> {
    Ok(std::sync::mpsc::channel())
}

#[cfg(not(feature = "ipc"))]
pub fn recv_timeout<T>(
    receiver: &WebXrReceiver<T>,
    timeout: Duration,
) -> Result<T, RecvTimeoutError> {
    receiver.recv_timeout(timeout)
}

#[cfg(feature = "ipc")]
pub fn recv_timeout<T>(
    receiver: &WebXrReceiver<T>,
    timeout: Duration,
) -> Result<T, ipc_channel::ipc::TryRecvError>
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a>,
{
    // Sigh, polling, sigh.
    let mut delay = timeout / 1000;
    while delay < timeout {
        if let Ok(msg) = receiver.try_recv() {
            return Ok(msg);
        }
        thread::sleep(delay);
        delay *= 2;
    }
    receiver.try_recv()
}
