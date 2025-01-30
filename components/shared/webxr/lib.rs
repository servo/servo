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

pub use device::DeviceAPI;
pub use device::DiscoveryAPI;

pub use error::Error;

pub use events::Event;
pub use events::EventBuffer;
pub use events::Visibility;

pub use frame::Frame;
pub use frame::FrameUpdateEvent;
pub use frame::ViewerPose;

pub use hand::Finger;
pub use hand::FingerJoint;
pub use hand::Hand;
pub use hand::HandSpace;
pub use hand::Joint;
pub use hand::JointFrame;

pub use hittest::EntityType;
pub use hittest::EntityTypes;
pub use hittest::HitTestId;
pub use hittest::HitTestResult;
pub use hittest::HitTestSource;
pub use hittest::HitTestSpace;
pub use hittest::Ray;
pub use hittest::Triangle;

pub use input::Handedness;
pub use input::InputFrame;
pub use input::InputId;
pub use input::InputSource;
pub use input::SelectEvent;
pub use input::SelectKind;
pub use input::TargetRayMode;

pub use layer::ContextId;
pub use layer::GLContexts;
pub use layer::GLTypes;
pub use layer::LayerGrandManager;
pub use layer::LayerGrandManagerAPI;
pub use layer::LayerId;
pub use layer::LayerInit;
pub use layer::LayerLayout;
pub use layer::LayerManager;
pub use layer::LayerManagerAPI;
pub use layer::LayerManagerFactory;
pub use layer::SubImage;
pub use layer::SubImages;

pub use mock::MockButton;
pub use mock::MockButtonType;
pub use mock::MockDeviceInit;
pub use mock::MockDeviceMsg;
pub use mock::MockDiscoveryAPI;
pub use mock::MockInputInit;
pub use mock::MockInputMsg;
pub use mock::MockRegion;
pub use mock::MockViewInit;
pub use mock::MockViewsInit;
pub use mock::MockWorld;

pub use registry::MainThreadRegistry;
pub use registry::MainThreadWaker;
pub use registry::Registry;

pub use session::EnvironmentBlendMode;
pub use session::MainThreadSession;
pub use session::Quitter;
pub use session::Session;
pub use session::SessionBuilder;
pub use session::SessionId;
pub use session::SessionInit;
pub use session::SessionMode;
pub use session::SessionThread;

pub use space::ApiSpace;
pub use space::BaseSpace;
pub use space::Space;

pub use view::Capture;
pub use view::CubeBack;
pub use view::CubeBottom;
pub use view::CubeLeft;
pub use view::CubeRight;
pub use view::CubeTop;
pub use view::Display;
pub use view::Floor;
pub use view::Input;
pub use view::LeftEye;
pub use view::Native;
pub use view::RightEye;
pub use view::SomeEye;
pub use view::View;
pub use view::Viewer;
pub use view::Viewport;
pub use view::Viewports;
pub use view::Views;
pub use view::CUBE_BACK;
pub use view::CUBE_BOTTOM;
pub use view::CUBE_LEFT;
pub use view::CUBE_RIGHT;
pub use view::CUBE_TOP;
pub use view::LEFT_EYE;
pub use view::RIGHT_EYE;
pub use view::VIEWER;

#[cfg(feature = "ipc")]
use std::thread;

use std::time::Duration;

#[cfg(feature = "ipc")]
pub use ipc_channel::ipc::IpcSender as Sender;

#[cfg(feature = "ipc")]
pub use ipc_channel::ipc::IpcReceiver as Receiver;

#[cfg(feature = "ipc")]
pub use ipc_channel::ipc::channel;

#[cfg(not(feature = "ipc"))]
pub use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};

#[cfg(not(feature = "ipc"))]
pub fn channel<T>() -> Result<(Sender<T>, Receiver<T>), ()> {
    Ok(std::sync::mpsc::channel())
}

#[cfg(not(feature = "ipc"))]
pub fn recv_timeout<T>(receiver: &Receiver<T>, timeout: Duration) -> Result<T, RecvTimeoutError> {
    receiver.recv_timeout(timeout)
}

#[cfg(feature = "ipc")]
pub fn recv_timeout<T>(
    receiver: &Receiver<T>,
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
        delay = delay * 2;
    }
    receiver.try_recv()
}
