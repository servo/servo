/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Log an event from compositor at trace level.
/// - To disable tracing: RUST_LOG='constellation<compositor@=off'
/// - To enable tracing: RUST_LOG='constellation<compositor@'
/// - Recommended filters when tracing is enabled:
///   - constellation<compositor@ForwardEvent(MouseMoveEvent)=off
///   - constellation<compositor@LogEntry=off
///   - constellation<compositor@ReadyToPresent=off
macro_rules! trace_msg_from_compositor {
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

/// Log an event from layout at trace level.
/// - To disable tracing: RUST_LOG='constellation<layout@=off'
/// - To enable tracing: RUST_LOG='constellation<layout@'
macro_rules! trace_layout_msg {
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

mod from_compositor {
    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("constellation<compositor@", $($name),+)
        };
    }

    impl LogTarget for compositing_traits::ConstellationMsg {
        fn log_target(&self) -> &'static str {
            match self {
                Self::Exit => target!("Exit"),
                Self::GetBrowsingContext(..) => target!("GetBrowsingContext"),
                Self::GetPipeline(..) => target!("GetPipeline"),
                Self::GetFocusTopLevelBrowsingContext(..) => {
                    target!("GetFocusTopLevelBrowsingContext")
                },
                Self::IsReadyToSaveImage(..) => target!("IsReadyToSaveImage"),
                Self::Keyboard(..) => target!("Keyboard"),
                Self::AllowNavigationResponse(..) => target!("AllowNavigationResponse"),
                Self::LoadUrl(..) => target!("LoadUrl"),
                Self::ClearCache => target!("ClearCache"),
                Self::TraverseHistory(..) => target!("TraverseHistory"),
                Self::WindowSize(..) => target!("WindowSize"),
                Self::TickAnimation(..) => target!("TickAnimation"),
                Self::WebDriverCommand(..) => target!("WebDriverCommand"),
                Self::Reload(..) => target!("Reload"),
                Self::LogEntry(..) => target!("LogEntry"),
                Self::NewWebView(..) => target!("NewWebView"),
                Self::WebViewOpened(..) => target!("WebViewOpened"),
                Self::CloseWebView(..) => target!("CloseWebView"),
                Self::SendError(..) => target!("SendError"),
                Self::FocusWebView(..) => target!("FocusWebView"),
                Self::BlurWebView => target!("BlurWebView"),
                Self::ForwardEvent(_, event) => event.log_target(),
                Self::SetCursor(..) => target!("SetCursor"),
                Self::EnableProfiler(..) => target!("EnableProfiler"),
                Self::DisableProfiler => target!("DisableProfiler"),
                Self::ExitFullScreen(_) => target!("ExitFullScreen"),
                Self::MediaSessionAction(_) => target!("MediaSessionAction"),
                Self::SetWebViewThrottled(_, _) => target!("SetWebViewThrottled"),
                Self::IMEDismissed => target!("IMEDismissed"),
                Self::ReadyToPresent(..) => target!("ReadyToPresent"),
                Self::Gamepad(..) => target!("Gamepad"),
            }
        }
    }

    impl LogTarget for script_traits::CompositorEvent {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("ForwardEvent(" $name ")")
                };
            }
            match self {
                Self::ResizeEvent(..) => target_variant!("ResizeEvent"),
                Self::MouseButtonEvent(..) => target_variant!("MouseButtonEvent"),
                Self::MouseMoveEvent(..) => target_variant!("MouseMoveEvent"),
                Self::TouchEvent(..) => target_variant!("TouchEvent"),
                Self::WheelEvent(..) => target_variant!("WheelEvent"),
                Self::KeyboardEvent(..) => target_variant!("KeyboardEvent"),
                Self::CompositionEvent(..) => target_variant!("CompositionEvent"),
                Self::IMEDismissedEvent => target_variant!("IMEDismissedEvent"),
                Self::GamepadEvent(..) => target_variant!("GamepadEvent"),
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

    impl LogTarget for script_traits::ScriptMsg {
        fn log_target(&self) -> &'static str {
            match self {
                Self::CompleteMessagePortTransfer(..) => target!("CompleteMessagePortTransfer"),
                Self::MessagePortTransferResult(..) => target!("MessagePortTransferResult"),
                Self::NewMessagePort(..) => target!("NewMessagePort"),
                Self::NewMessagePortRouter(..) => target!("NewMessagePortRouter"),
                Self::RemoveMessagePortRouter(..) => target!("RemoveMessagePortRouter"),
                Self::RerouteMessagePort(..) => target!("RerouteMessagePort"),
                Self::MessagePortShipped(..) => target!("MessagePortShipped"),
                Self::RemoveMessagePort(..) => target!("RemoveMessagePort"),
                Self::EntanglePorts(..) => target!("EntanglePorts"),
                Self::NewBroadcastChannelRouter(..) => target!("NewBroadcastChannelRouter"),
                Self::RemoveBroadcastChannelRouter(..) => target!("RemoveBroadcastChannelRouter"),
                Self::NewBroadcastChannelNameInRouter(..) => {
                    target!("NewBroadcastChannelNameInRouter")
                },
                Self::RemoveBroadcastChannelNameInRouter(..) => {
                    target!("RemoveBroadcastChannelNameInRouter")
                },
                Self::ScheduleBroadcast(..) => target!("ScheduleBroadcast"),
                Self::ForwardToEmbedder(msg) => msg.log_target(),
                Self::InitiateNavigateRequest(..) => target!("InitiateNavigateRequest"),
                Self::BroadcastStorageEvent(..) => target!("BroadcastStorageEvent"),
                Self::ChangeRunningAnimationsState(..) => target!("ChangeRunningAnimationsState"),
                Self::CreateCanvasPaintThread(..) => target!("CreateCanvasPaintThread"),
                Self::Focus => target!("Focus"),
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
                Self::ScriptNewAuxiliary(..) => target!("ScriptNewAuxiliary"),
                Self::ActivateDocument => target!("ActivateDocument"),
                Self::SetDocumentState(..) => target!("SetDocumentState"),
                Self::SetLayoutEpoch(..) => target!("SetLayoutEpoch"),
                Self::SetFinalUrl(..) => target!("SetFinalUrl"),
                Self::TouchEventProcessed(..) => target!("TouchEventProcessed"),
                Self::LogEntry(..) => target!("LogEntry"),
                Self::DiscardDocument => target!("DiscardDocument"),
                Self::DiscardTopLevelBrowsingContext => target!("DiscardTopLevelBrowsingContext"),
                Self::PipelineExited => target!("PipelineExited"),
                Self::ForwardDOMMessage(..) => target!("ForwardDOMMessage"),
                Self::ScheduleJob(..) => target!("ScheduleJob"),
                Self::GetClientWindow(..) => target!("GetClientWindow"),
                Self::GetScreenSize(..) => target!("GetScreenSize"),
                Self::GetScreenAvailSize(..) => target!("GetScreenAvailSize"),
                Self::MediaSessionEvent(..) => target!("MediaSessionEvent"),
                Self::RequestAdapter(..) => target!("RequestAdapter"),
                Self::GetWebGPUChan(..) => target!("GetWebGPUChan"),
                Self::TitleChanged(..) => target!("TitleChanged"),
            }
        }
    }

    impl LogTarget for embedder_traits::EmbedderMsg {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("ForwardToEmbedder(" $name ")")
                };
            }
            match self {
                Self::Status(..) => target_variant!("Status"),
                Self::ChangePageTitle(..) => target_variant!("ChangePageTitle"),
                Self::MoveTo(..) => target_variant!("MoveTo"),
                Self::ResizeTo(..) => target_variant!("ResizeTo"),
                Self::Prompt(..) => target_variant!("Prompt"),
                Self::ShowContextMenu(..) => target_variant!("ShowContextMenu"),
                Self::AllowNavigationRequest(..) => target_variant!("AllowNavigationRequest"),
                Self::AllowOpeningWebView(..) => target_variant!("AllowOpeningWebView"),
                Self::WebViewOpened(..) => target_variant!("WebViewOpened"),
                Self::WebViewClosed(..) => target_variant!("WebViewClosed"),
                Self::WebViewFocused(..) => target_variant!("WebViewFocused"),
                Self::WebViewBlurred => target_variant!("WebViewBlurred"),
                Self::AllowUnload(..) => target_variant!("AllowUnload"),
                Self::Keyboard(..) => target_variant!("Keyboard"),
                Self::GetClipboardContents(..) => target_variant!("GetClipboardContents"),
                Self::SetClipboardContents(..) => target_variant!("SetClipboardContents"),
                Self::SetCursor(..) => target_variant!("SetCursor"),
                Self::NewFavicon(..) => target_variant!("NewFavicon"),
                Self::HeadParsed => target_variant!("HeadParsed"),
                Self::HistoryChanged(..) => target_variant!("HistoryChanged"),
                Self::SetFullscreenState(..) => target_variant!("SetFullscreenState"),
                Self::LoadStart => target_variant!("LoadStart"),
                Self::LoadComplete => target_variant!("LoadComplete"),
                Self::Panic(..) => target_variant!("Panic"),
                Self::GetSelectedBluetoothDevice(..) => {
                    target_variant!("GetSelectedBluetoothDevice")
                },
                Self::SelectFiles(..) => target_variant!("SelectFiles"),
                Self::PromptPermission(..) => target_variant!("PromptPermission"),
                Self::ShowIME(..) => target_variant!("ShowIME"),
                Self::HideIME => target_variant!("HideIME"),
                Self::Shutdown => target_variant!("Shutdown"),
                Self::ReportProfile(..) => target_variant!("ReportProfile"),
                Self::MediaSessionEvent(..) => target_variant!("MediaSessionEvent"),
                Self::OnDevtoolsStarted(..) => target_variant!("OnDevtoolsStarted"),
                Self::ReadyToPresent(..) => target_variant!("ReadyToPresent"),
                Self::EventDelivered(..) => target_variant!("EventDelivered"),
                Self::PlayGamepadHapticEffect(..) => target_variant!("PlayGamepadHapticEffect"),
                Self::StopGamepadHapticEffect(..) => target_variant!("StopGamepadHapticEffect"),
            }
        }
    }
}

mod from_layout {
    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("constellation<layout@", $($name),+)
        };
    }

    impl LogTarget for script_traits::LayoutMsg {
        fn log_target(&self) -> &'static str {
            match self {
                Self::IFrameSizes(..) => target!("IFrameSizes"),
                Self::PendingPaintMetric(..) => target!("PendingPaintMetric"),
            }
        }
    }
}
