/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rustc_hash::FxHashMap;
use servo_base::Epoch;
use servo_base::id::PipelineId;

/// When a [`ScreenshotReadinessRequest`] is received from the renderer, the
/// [`crate::Constellation`] goes through a variety of states to process them. This data structure
/// represents those states.
#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum ScreenshotRequestState {
    /// The [`crate::Constellation`] has received the [`ScreenshotReadinessRequest`], but has not
    /// yet forwarded it to the `Pipeline`'s of the request's WebView. This is likely because there
    /// are still pending navigation changes in the [`crate::Constellation`]. Once those changes are
    /// resolved the request will be forwarded to the `Pipeline`s.
    #[default]
    Pending,
    /// The [`crate::Constellation`] has forwarded the [`ScreenshotReadinessRequest`] to the
    /// `Pipeline`s of the corresponding `WebView`. The `Pipeline`s are waiting for a variety of
    /// things to happen in order to report what the appropriate display list epoch is for the
    /// screenshot. Once they all report back, the [`crate::Constellation`] considers that the
    /// request is handled, and the renderer is responsible for waiting to take the screenshot.
    WaitingOnScript,
}

pub(crate) struct ScreenshotReadinessRequest {
    pub state: ScreenshotRequestState,
    pub pipeline_states: FxHashMap<PipelineId, Option<Epoch>>,
}
