/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate ipc_channel;
extern crate msg;
#[macro_use] extern crate serde;
pub extern crate rust_webvr as webvr;

mod webvr_traits;

pub use webvr::VRDisplayData as WebVRDisplayData;
pub use webvr::VRDisplayCapabilities as WebVRDisplayCapabilities;
pub use webvr::VRDisplayEvent as WebVRDisplayEvent;
pub use webvr::VRDisplayEventReason as WebVRDisplayEventReason;
pub use webvr::VREvent as WebVREvent;
pub use webvr::VREye as WebVREye;
pub use webvr::VREyeParameters as WebVREyeParameters;
pub use webvr::VRFieldOfView as WebVRFieldOfView;
pub use webvr::VRGamepadButton as WebVRGamepadButton;
pub use webvr::VRGamepadData as WebVRGamepadData;
pub use webvr::VRGamepadEvent as WebVRGamepadEvent;
pub use webvr::VRGamepadHand as WebVRGamepadHand;
pub use webvr::VRGamepadState as WebVRGamepadState;
pub use webvr::VRFrameData as WebVRFrameData;
pub use webvr::VRLayer as WebVRLayer;
pub use webvr::VRPose as WebVRPose;
pub use webvr::VRStageParameters as WebVRStageParameters;
pub use webvr_traits::{WebVRMsg, WebVRResult};
