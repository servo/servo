/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

extern crate ipc_channel;
extern crate msg;
#[macro_use]
extern crate serde;
pub extern crate rust_webvr_api as webvr;

mod webvr_traits;

pub use crate::webvr::VRDisplayData as WebVRDisplayData;
pub use crate::webvr::VRDisplayCapabilities as WebVRDisplayCapabilities;
pub use crate::webvr::VRDisplayEvent as WebVRDisplayEvent;
pub use crate::webvr::VRDisplayEventReason as WebVRDisplayEventReason;
pub use crate::webvr::VREvent as WebVREvent;
pub use crate::webvr::VREye as WebVREye;
pub use crate::webvr::VREyeParameters as WebVREyeParameters;
pub use crate::webvr::VRFieldOfView as WebVRFieldOfView;
pub use crate::webvr::VRGamepadButton as WebVRGamepadButton;
pub use crate::webvr::VRGamepadData as WebVRGamepadData;
pub use crate::webvr::VRGamepadEvent as WebVRGamepadEvent;
pub use crate::webvr::VRGamepadHand as WebVRGamepadHand;
pub use crate::webvr::VRGamepadState as WebVRGamepadState;
pub use crate::webvr::VRFrameData as WebVRFrameData;
pub use crate::webvr::VRLayer as WebVRLayer;
pub use crate::webvr::VRPose as WebVRPose;
pub use crate::webvr::VRStageParameters as WebVRStageParameters;
pub use crate::webvr_traits::{WebVRMsg, WebVRResult};
