/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Log an event from embedder at trace level.
/// - To disable tracing: RUST_LOG='constellation<embedder@=off'
/// - To enable tracing: RUST_LOG='constellation<embedder@'
/// - Recommended filters when tracing is enabled:
///   - constellation<embedder@ForwardEvent(MouseMoveEvent)=off
///   - constellation<embedder@LogEntry=off
///   - constellation<embedder@ReadyToPresent=off
macro_rules! trace_msg_from_embedder {
    // This macro only exists to put the docs in the same file as the target prefix,
    // so the macro definition is always the same.
    ($event:expr, $($rest:tt)+) => {
        ::log::trace!(target: $crate::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

/// Log an event from script at trace level.
/// - To disable tracing: RUST_LOG='constellation<script@=off'
/// - To enable tracing: RUST_LOG='constellation<script@'
/// - Recommended filters when tracing is enabled:
///   - constellation<script@LogEntry=off
macro_rules! trace_script_msg {
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

mod from_embedder {
    use embedder_traits::{InputEvent, InputEventAndId};

    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("constellation<embedder@", $($name),+)
        };
    }

    impl LogTarget for constellation_traits::EmbedderToConstellationMessage {
        fn log_target(&self) -> &'static str {
            match self {
                Self::Exit => target!("Exit"),
                Self::AllowNavigationResponse(..) => target!("AllowNavigationResponse"),
                Self::LoadUrl(..) => target!("LoadUrl"),
                Self::TraverseHistory(..) => target!("TraverseHistory"),
                Self::ChangeViewportDetails(..) => target!("ChangeViewportDetails"),
                Self::ThemeChange(..) => target!("ThemeChange"),
                Self::TickAnimation(..) => target!("TickAnimation"),
                Self::WebDriverCommand(..) => target!("WebDriverCommand"),
                Self::Reload(..) => target!("Reload"),
                Self::LogEntry(..) => target!("LogEntry"),
                Self::NewWebView(..) => target!("NewWebView"),
                Self::CloseWebView(..) => target!("CloseWebView"),
                Self::SendError(..) => target!("SendError"),
                Self::FocusWebView(..) => target!("FocusWebView"),
                Self::BlurWebView => target!("BlurWebView"),
                Self::ForwardInputEvent(_webview_id, event, ..) => event.log_target(),
                Self::RefreshCursor(..) => target!("RefreshCursor"),
                Self::ToggleProfiler(..) => target!("EnableProfiler"),
                Self::ExitFullScreen(_) => target!("ExitFullScreen"),
                Self::MediaSessionAction(_) => target!("MediaSessionAction"),
                Self::SetWebViewThrottled(_, _) => target!("SetWebViewThrottled"),
                Self::SetScrollStates(..) => target!("SetScrollStates"),
                Self::PaintMetric(..) => target!("PaintMetric"),
                Self::EvaluateJavaScript(..) => target!("EvaluateJavaScript"),
                Self::CreateMemoryReport(..) => target!("CreateMemoryReport"),
                Self::SendImageKeysForPipeline(..) => target!("SendImageKeysForPipeline"),
                Self::PreferencesUpdated(..) => target!("PreferencesUpdated"),
                Self::NoLongerWaitingOnAsynchronousImageUpdates(..) => {
                    target!("NoLongerWaitingOnCanvas")
                },
                Self::RequestScreenshotReadiness(..) => target!("RequestScreenshotReadiness"),
                Self::EmbedderControlResponse(..) => target!("EmbedderControlResponse"),
            }
        }
    }

    impl LogTarget for InputEventAndId {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("ForwardInputEvent(" $name ")")
                };
            }
            match self.event {
                InputEvent::EditingAction(..) => target_variant!("EditingAction"),
                InputEvent::Gamepad(..) => target_variant!("Gamepad"),
                InputEvent::Ime(..) => target_variant!("Ime"),
                InputEvent::Keyboard(..) => target_variant!("Keyboard"),
                InputEvent::MouseButton(..) => target_variant!("MouseButton"),
                InputEvent::MouseMove(..) => target_variant!("MouseMove"),
                InputEvent::MouseLeftViewport(..) => target_variant!("MouseLeftViewport"),
                InputEvent::Touch(..) => target_variant!("Touch"),
                InputEvent::Wheel(..) => target_variant!("Wheel"),
                InputEvent::Scroll(..) => target_variant!("Scroll"),
            }
        }
    }
}

mod from_script {
    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("constellation<script@", $($name),+)
        };
    }

    impl LogTarget for constellation_traits::ScriptToConstellationMessage {
        fn log_target(&self) -> &'static str {
            match self {
                Self::CompleteMessagePortTransfer(..) => target!("CompleteMessagePortTransfer"),
                Self::MessagePortTransferResult(..) => target!("MessagePortTransferResult"),
                Self::NewMessagePort(..) => target!("NewMessagePort"),
                Self::NewMessagePortRouter(..) => target!("NewMessagePortRouter"),
                Self::RemoveMessagePortRouter(..) => target!("RemoveMessagePortRouter"),
                Self::RerouteMessagePort(..) => target!("RerouteMessagePort"),
                Self::MessagePortShipped(..) => target!("MessagePortShipped"),
                Self::EntanglePorts(..) => target!("EntanglePorts"),
                Self::DisentanglePorts(..) => target!("DisentanglePorts"),
                Self::NewBroadcastChannelRouter(..) => target!("NewBroadcastChannelRouter"),
                Self::RemoveBroadcastChannelRouter(..) => target!("RemoveBroadcastChannelRouter"),
                Self::NewBroadcastChannelNameInRouter(..) => {
                    target!("NewBroadcastChannelNameInRouter")
                },
                Self::RemoveBroadcastChannelNameInRouter(..) => {
                    target!("RemoveBroadcastChannelNameInRouter")
                },
                Self::ScheduleBroadcast(..) => target!("ScheduleBroadcast"),
                Self::BroadcastStorageEvent(..) => target!("BroadcastStorageEvent"),
                Self::ChangeRunningAnimationsState(..) => target!("ChangeRunningAnimationsState"),
                Self::CreateCanvasPaintThread(..) => target!("CreateCanvasPaintThread"),
                Self::Focus(..) => target!("Focus"),
                Self::FocusRemoteDocument(..) => target!("FocusRemoteDocument"),
                Self::GetTopForBrowsingContext(..) => target!("GetTopForBrowsingContext"),
                Self::GetBrowsingContextInfo(..) => target!("GetBrowsingContextInfo"),
                Self::GetChildBrowsingContextId(..) => target!("GetChildBrowsingContextId"),
                Self::LoadComplete => target!("LoadComplete"),
                Self::LoadUrl(..) => target!("LoadUrl"),
                Self::AbortLoadUrl => target!("AbortLoadUrl"),
                Self::PostMessage { .. } => target!("PostMessage"),
                Self::NavigatedToFragment(..) => target!("NavigatedToFragment"),
                Self::TraverseHistory(..) => target!("TraverseHistory"),
                Self::PushHistoryState(..) => target!("PushHistoryState"),
                Self::ReplaceHistoryState(..) => target!("ReplaceHistoryState"),
                Self::JointSessionHistoryLength(..) => target!("JointSessionHistoryLength"),
                Self::RemoveIFrame(..) => target!("RemoveIFrame"),
                Self::SetThrottledComplete(..) => target!("SetThrottledComplete"),
                Self::ScriptLoadedURLInIFrame(..) => target!("ScriptLoadedURLInIFrame"),
                Self::ScriptNewIFrame(..) => target!("ScriptNewIFrame"),
                Self::CreateAuxiliaryWebView(..) => target!("ScriptNewAuxiliary"),
                Self::ActivateDocument => target!("ActivateDocument"),
                Self::SetDocumentState(..) => target!("SetDocumentState"),
                Self::SetFinalUrl(..) => target!("SetFinalUrl"),
                Self::LogEntry(..) => target!("LogEntry"),
                Self::DiscardDocument => target!("DiscardDocument"),
                Self::DiscardTopLevelBrowsingContext => target!("DiscardTopLevelBrowsingContext"),
                Self::PipelineExited => target!("PipelineExited"),
                Self::ForwardDOMMessage(..) => target!("ForwardDOMMessage"),
                Self::ScheduleJob(..) => target!("ScheduleJob"),
                Self::MediaSessionEvent(..) => target!("MediaSessionEvent"),
                #[cfg(feature = "webgpu")]
                Self::RequestAdapter(..) => target!("RequestAdapter"),
                #[cfg(feature = "webgpu")]
                Self::GetWebGPUChan(..) => target!("GetWebGPUChan"),
                Self::TitleChanged(..) => target!("TitleChanged"),
                Self::IFrameSizes(..) => target!("IFrameSizes"),
                Self::ReportMemory(..) => target!("ReportMemory"),
                Self::FinishJavaScriptEvaluation(..) => target!("FinishJavaScriptEvaluation"),
                Self::ForwardKeyboardScroll(..) => target!("ForwardKeyboardScroll"),
                Self::RespondToScreenshotReadinessRequest(..) => {
                    target!("RespondToScreenshotReadinessRequest")
                },
            }
        }
    }
}
