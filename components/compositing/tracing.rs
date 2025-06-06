/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Log an event from constellation at trace level.
/// - To disable tracing: RUST_LOG='compositor<constellation@=off'
/// - To enable tracing: RUST_LOG='compositor<constellation@'
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
            concat!("compositor<constellation@", $($name),+)
        };
    }

    impl LogTarget for compositing_traits::CompositorMsg {
        fn log_target(&self) -> &'static str {
            match self {
                Self::ChangeRunningAnimationsState(..) => target!("ChangeRunningAnimationsState"),
                Self::CreateOrUpdateWebView(..) => target!("CreateOrUpdateWebView"),
                Self::RemoveWebView(..) => target!("RemoveWebView"),
                Self::TouchEventProcessed(..) => target!("TouchEventProcessed"),
                Self::CreatePng(..) => target!("CreatePng"),
                Self::IsReadyToSaveImageReply(..) => target!("IsReadyToSaveImageReply"),
                Self::SetThrottled(..) => target!("SetThrottled"),
                Self::NewWebRenderFrameReady(..) => target!("NewWebRenderFrameReady"),
                Self::PipelineExited(..) => target!("PipelineExited"),
                Self::LoadComplete(..) => target!("LoadComplete"),
                Self::WebDriverMouseButtonEvent(..) => target!("WebDriverMouseButtonEvent"),
                Self::WebDriverMouseMoveEvent(..) => target!("WebDriverMouseMoveEvent"),
                Self::WebDriverWheelScrollEvent(..) => target!("WebDriverWheelScrollEvent"),
                Self::SendInitialTransaction(..) => target!("SendInitialTransaction"),
                Self::SendScrollNode(..) => target!("SendScrollNode"),
                Self::SendDisplayList { .. } => target!("SendDisplayList"),
                Self::HitTest(..) => target!("HitTest"),
                Self::GenerateImageKey(..) => target!("GenerateImageKey"),
                Self::AddImage(..) => target!("AddImage"),
                Self::UpdateImages(..) => target!("UpdateImages"),
                Self::GenerateFontKeys(..) => target!("GenerateFontKeys"),
                Self::AddFont(..) => target!("AddFont"),
                Self::AddSystemFont(..) => target!("AddSystemFont"),
                Self::AddFontInstance(..) => target!("AddFontInstance"),
                Self::RemoveFonts(..) => target!("RemoveFonts"),
                Self::GetClientWindowRect(..) => target!("GetClientWindowRect"),
                Self::GetScreenSize(..) => target!("GetScreenSize"),
                Self::GetAvailableScreenSize(..) => target!("GetAvailableScreenSize"),
                Self::CollectMemoryReport(..) => target!("CollectMemoryReport"),
                Self::Viewport(..) => target!("Viewport"),
            }
        }
    }
}
