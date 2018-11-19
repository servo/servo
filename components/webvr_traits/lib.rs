/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate serde;

mod webvr_traits;

pub use crate::webvr_traits::{WebVRMsg, WebVRResult};
pub use rust_webvr_api as webvr;
pub use rust_webvr_api::VRDisplayCapabilities as WebVRDisplayCapabilities;
pub use rust_webvr_api::VRDisplayData as WebVRDisplayData;
pub use rust_webvr_api::VRDisplayEvent as WebVRDisplayEvent;
pub use rust_webvr_api::VRDisplayEventReason as WebVRDisplayEventReason;
pub use rust_webvr_api::VREvent as WebVREvent;
pub use rust_webvr_api::VREye as WebVREye;
pub use rust_webvr_api::VREyeParameters as WebVREyeParameters;
pub use rust_webvr_api::VRFieldOfView as WebVRFieldOfView;
pub use rust_webvr_api::VRFrameData as WebVRFrameData;
pub use rust_webvr_api::VRGamepadButton as WebVRGamepadButton;
pub use rust_webvr_api::VRGamepadData as WebVRGamepadData;
pub use rust_webvr_api::VRGamepadEvent as WebVRGamepadEvent;
pub use rust_webvr_api::VRGamepadHand as WebVRGamepadHand;
pub use rust_webvr_api::VRGamepadState as WebVRGamepadState;
pub use rust_webvr_api::VRLayer as WebVRLayer;
pub use rust_webvr_api::VRPose as WebVRPose;
pub use rust_webvr_api::VRStageParameters as WebVRStageParameters;
