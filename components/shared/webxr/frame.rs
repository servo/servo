/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::Floor;
use crate::HitTestId;
use crate::HitTestResult;
use crate::InputFrame;
use crate::Native;
use crate::SubImages;
use crate::Viewer;
use crate::Viewports;
use crate::Views;

use euclid::RigidTransform3D;

/// The per-frame data that is provided by the device.
/// https://www.w3.org/TR/webxr/#xrframe
// TODO: other fields?
#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
    /// The pose information of the viewer
    pub pose: Option<ViewerPose>,
    /// Frame information for each connected input source
    pub inputs: Vec<InputFrame>,

    /// Events that occur with the frame.
    pub events: Vec<FrameUpdateEvent>,

    /// The subimages to render to
    pub sub_images: Vec<SubImages>,

    /// The hit test results for this frame, if any
    pub hit_test_results: Vec<HitTestResult>,

    /// The average point in time this XRFrame is expected to be displayed on the devices' display
    pub predicted_display_time: f64,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameUpdateEvent {
    UpdateFloorTransform(Option<RigidTransform3D<f32, Native, Floor>>),
    UpdateViewports(Viewports),
    HitTestSourceAdded(HitTestId),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct ViewerPose {
    /// The transform from the viewer to native coordinates
    ///
    /// This is equivalent to the pose of the viewer in native coordinates.
    /// This is the inverse of the view matrix.
    pub transform: RigidTransform3D<f32, Viewer, Native>,

    // The various views
    pub views: Views,
}
