/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use webvr::*;

pub type WebVRResult<T> = Result<T, String>;

// Messages from Script thread to WebVR thread.
#[derive(Deserialize, Serialize)]
pub enum WebVRMsg {
    RegisterContext(PipelineId),
    UnregisterContext(PipelineId),
    PollEvents(IpcSender<bool>),
    GetDisplays(IpcSender<WebVRResult<Vec<VRDisplayData>>>),
    GetFrameData(PipelineId, u64, f64, f64, IpcSender<WebVRResult<VRFrameData>>),
    ResetPose(PipelineId, u64, IpcSender<WebVRResult<VRDisplayData>>),
    RequestPresent(PipelineId, u64, IpcSender<WebVRResult<()>>),
    ExitPresent(PipelineId, u64, Option<IpcSender<WebVRResult<()>>>),
    CreateCompositor(u64),
    Exit,
}
