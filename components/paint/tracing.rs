/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Log an event from constellation at trace level.
/// - To disable tracing: RUST_LOG='paint<constellation@=off'
/// - To enable tracing: RUST_LOG='paint<constellation@'
macro_rules! trace_msg_from_constellation {
    // This macro only exists to put the docs in the same file as the target prefix,
    // so the macro definition is always the same.
    ($event:expr, $($rest:tt)+) => {
        ::log::trace!(target: $crate::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

/// Get the log target for an event, as a static string.
pub(crate) trait LogTarget {
    fn log_target(&self) -> &'static str;
}

mod from_constellation {
    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("paint<constellation@", $($name),+)
        };
    }

    impl LogTarget for paint_api::PaintMessage {
        fn log_target(&self) -> &'static str {
            match self {
                Self::ChangeRunningAnimationsState(..) => target!("ChangeRunningAnimationsState"),
                Self::SetFrameTreeForWebView(..) => target!("SetFrameTreeForWebView"),
                Self::SetThrottled(..) => target!("SetThrottled"),
                Self::NewWebRenderFrameReady(..) => target!("NewWebRenderFrameReady"),
                Self::PipelineExited(..) => target!("PipelineExited"),
                Self::SendInitialTransaction(..) => target!("SendInitialTransaction"),
                Self::ScrollNodeByDelta(..) => target!("ScrollNodeByDelta"),
                Self::ScrollViewportByDelta(..) => target!("ScrollViewportByDelta"),
                Self::UpdateEpoch { .. } => target!("UpdateEpoch"),
                Self::SendDisplayList { .. } => target!("SendDisplayList"),
                Self::GenerateFrame { .. } => target!("GenerateFrame"),
                Self::GenerateImageKey(..) => target!("GenerateImageKey"),
                Self::UpdateImages(..) => target!("UpdateImages"),
                Self::GenerateFontKeys(..) => target!("GenerateFontKeys"),
                Self::AddFont(..) => target!("AddFont"),
                Self::AddSystemFont(..) => target!("AddSystemFont"),
                Self::AddFontInstance(..) => target!("AddFontInstance"),
                Self::RemoveFonts(..) => target!("RemoveFonts"),
                Self::CollectMemoryReport(..) => target!("CollectMemoryReport"),
                Self::Viewport(..) => target!("Viewport"),
                Self::GenerateImageKeysForPipeline(..) => target!("GenerateImageKeysForPipeline"),
                Self::DelayNewFrameForCanvas(..) => target!("DelayFramesForCanvas"),
                Self::ScreenshotReadinessReponse(..) => target!("ScreenshotReadinessResponse"),
                Self::SendLCPCandidate(..) => target!("SendLCPCandidate"),
                Self::EnableLCPCalculation(..) => target!("EnableLCPCalculation"),
            }
        }
    }
}
