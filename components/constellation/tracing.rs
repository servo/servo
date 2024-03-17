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
                Self::GetBrowsingContext(_, _) => target!("GetBrowsingContext"),
                Self::GetPipeline(_, _) => target!("GetPipeline"),
                Self::GetFocusTopLevelBrowsingContext(_) => {
                    target!("GetFocusTopLevelBrowsingContext")
                },
                Self::IsReadyToSaveImage(_) => target!("IsReadyToSaveImage"),
                Self::Keyboard(_) => target!("Keyboard"),
                Self::AllowNavigationResponse(_, _) => target!("AllowNavigationResponse"),
                Self::LoadUrl(_, _) => target!("LoadUrl"),
                Self::ClearCache => target!("ClearCache"),
                Self::TraverseHistory(_, _) => target!("TraverseHistory"),
                Self::WindowSize(_, _, _) => target!("WindowSize"),
                Self::TickAnimation(_, _) => target!("TickAnimation"),
                Self::WebDriverCommand(_) => target!("WebDriverCommand"),
                Self::Reload(_) => target!("Reload"),
                Self::LogEntry(_, _, _) => target!("LogEntry"),
                Self::NewWebView(_, _) => target!("NewWebView"),
                Self::CloseWebView(_) => target!("CloseWebView"),
                Self::SendError(_, _) => target!("SendError"),
                Self::FocusWebView(_) => target!("FocusWebView"),
                Self::BlurWebView => target!("BlurWebView"),
                Self::ForwardEvent(_, event) => event.log_target(),
                Self::SetCursor(_) => target!("SetCursor"),
                Self::EnableProfiler(_, _) => target!("EnableProfiler"),
                Self::DisableProfiler => target!("DisableProfiler"),
                Self::ExitFullScreen(_) => target!("ExitFullScreen"),
                Self::MediaSessionAction(_) => target!("MediaSessionAction"),
                Self::WebViewVisibilityChanged(_, _) => target!("WebViewVisibilityChanged"),
                Self::IMEDismissed => target!("IMEDismissed"),
                Self::ReadyToPresent(_) => target!("ReadyToPresent"),
                Self::Gamepad(_) => target!("Gamepad"),
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
                Self::ResizeEvent(_, _) => target_variant!("ResizeEvent"),
                Self::MouseButtonEvent(_, _, _, _, _, _) => target_variant!("MouseButtonEvent"),
                Self::MouseMoveEvent(_, _, _) => target_variant!("MouseMoveEvent"),
                Self::TouchEvent(_, _, _, _) => target_variant!("TouchEvent"),
                Self::WheelEvent(_, _, _) => target_variant!("WheelEvent"),
                Self::KeyboardEvent(_) => target_variant!("KeyboardEvent"),
                Self::CompositionEvent(_) => target_variant!("CompositionEvent"),
                Self::IMEDismissedEvent => target_variant!("IMEDismissedEvent"),
                Self::GamepadEvent(_) => target_variant!("GamepadEvent"),
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
                Self::CompleteMessagePortTransfer(_, _) => target!("CompleteMessagePortTransfer"),
                Self::MessagePortTransferResult(_, _, _) => target!("MessagePortTransferResult"),
                Self::NewMessagePort(_, _) => target!("NewMessagePort"),
                Self::NewMessagePortRouter(_, _) => target!("NewMessagePortRouter"),
                Self::RemoveMessagePortRouter(_) => target!("RemoveMessagePortRouter"),
                Self::RerouteMessagePort(_, _) => target!("RerouteMessagePort"),
                Self::MessagePortShipped(_) => target!("MessagePortShipped"),
                Self::RemoveMessagePort(_) => target!("RemoveMessagePort"),
                Self::EntanglePorts(_, _) => target!("EntanglePorts"),
                Self::NewBroadcastChannelRouter(_, _, _) => target!("NewBroadcastChannelRouter"),
                Self::RemoveBroadcastChannelRouter(_, _) => target!("RemoveBroadcastChannelRouter"),
                Self::NewBroadcastChannelNameInRouter(_, _, _) => {
                    target!("NewBroadcastChannelNameInRouter")
                },
                Self::RemoveBroadcastChannelNameInRouter(_, _, _) => {
                    target!("RemoveBroadcastChannelNameInRouter")
                },
                Self::ScheduleBroadcast(_, _) => target!("ScheduleBroadcast"),
                Self::ForwardToEmbedder(msg) => msg.log_target(),
                Self::InitiateNavigateRequest(_, _) => target!("InitiateNavigateRequest"),
                Self::BroadcastStorageEvent(_, _, _, _, _) => target!("BroadcastStorageEvent"),
                Self::ChangeRunningAnimationsState(_) => target!("ChangeRunningAnimationsState"),
                Self::CreateCanvasPaintThread(_, _) => target!("CreateCanvasPaintThread"),
                Self::Focus => target!("Focus"),
                Self::GetTopForBrowsingContext(_, _) => target!("GetTopForBrowsingContext"),
                Self::GetBrowsingContextInfo(_, _) => target!("GetBrowsingContextInfo"),
                Self::GetChildBrowsingContextId(_, _, _) => target!("GetChildBrowsingContextId"),
                Self::LoadComplete => target!("LoadComplete"),
                Self::LoadUrl(_, _) => target!("LoadUrl"),
                Self::AbortLoadUrl => target!("AbortLoadUrl"),
                Self::PostMessage { .. } => target!("PostMessage"),
                Self::NavigatedToFragment(_, _) => target!("NavigatedToFragment"),
                Self::TraverseHistory(_) => target!("TraverseHistory"),
                Self::PushHistoryState(_, _) => target!("PushHistoryState"),
                Self::ReplaceHistoryState(_, _) => target!("ReplaceHistoryState"),
                Self::JointSessionHistoryLength(_) => target!("JointSessionHistoryLength"),
                Self::RemoveIFrame(_, _) => target!("RemoveIFrame"),
                Self::VisibilityChangeComplete(_) => target!("VisibilityChangeComplete"),
                Self::ScriptLoadedURLInIFrame(_) => target!("ScriptLoadedURLInIFrame"),
                Self::ScriptNewIFrame(_) => target!("ScriptNewIFrame"),
                Self::ScriptNewAuxiliary(_) => target!("ScriptNewAuxiliary"),
                Self::ActivateDocument => target!("ActivateDocument"),
                Self::SetDocumentState(_) => target!("SetDocumentState"),
                Self::SetLayoutEpoch(_, _) => target!("SetLayoutEpoch"),
                Self::SetFinalUrl(_) => target!("SetFinalUrl"),
                Self::TouchEventProcessed(_) => target!("TouchEventProcessed"),
                Self::LogEntry(_, _) => target!("LogEntry"),
                Self::DiscardDocument => target!("DiscardDocument"),
                Self::DiscardTopLevelBrowsingContext => target!("DiscardTopLevelBrowsingContext"),
                Self::PipelineExited => target!("PipelineExited"),
                Self::ForwardDOMMessage(_, _) => target!("ForwardDOMMessage"),
                Self::ScheduleJob(_) => target!("ScheduleJob"),
                Self::GetClientWindow(_) => target!("GetClientWindow"),
                Self::GetScreenSize(_) => target!("GetScreenSize"),
                Self::GetScreenAvailSize(_) => target!("GetScreenAvailSize"),
                Self::MediaSessionEvent(_, _) => target!("MediaSessionEvent"),
                Self::RequestAdapter(_, _, _) => target!("RequestAdapter"),
                Self::GetWebGPUChan(_) => target!("GetWebGPUChan"),
                Self::TitleChanged(_, _) => target!("TitleChanged"),
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
                Self::Status(_) => target_variant!("Status"),
                Self::ChangePageTitle(_) => target_variant!("ChangePageTitle"),
                Self::MoveTo(_) => target_variant!("MoveTo"),
                Self::ResizeTo(_) => target_variant!("ResizeTo"),
                Self::Prompt(_, _) => target_variant!("Prompt"),
                Self::ShowContextMenu(_, _, _) => target_variant!("ShowContextMenu"),
                Self::AllowNavigationRequest(_, _) => target_variant!("AllowNavigationRequest"),
                Self::AllowOpeningWebView(_) => target_variant!("AllowOpeningWebView"),
                Self::WebViewOpened(_) => target_variant!("WebViewOpened"),
                Self::WebViewClosed(_) => target_variant!("WebViewClosed"),
                Self::WebViewFocused(_) => target_variant!("WebViewFocused"),
                Self::WebViewBlurred => target_variant!("WebViewBlurred"),
                Self::AllowUnload(_) => target_variant!("AllowUnload"),
                Self::Keyboard(_) => target_variant!("Keyboard"),
                Self::GetClipboardContents(_) => target_variant!("GetClipboardContents"),
                Self::SetClipboardContents(_) => target_variant!("SetClipboardContents"),
                Self::SetCursor(_) => target_variant!("SetCursor"),
                Self::NewFavicon(_) => target_variant!("NewFavicon"),
                Self::HeadParsed => target_variant!("HeadParsed"),
                Self::HistoryChanged(_, _) => target_variant!("HistoryChanged"),
                Self::SetFullscreenState(_) => target_variant!("SetFullscreenState"),
                Self::LoadStart => target_variant!("LoadStart"),
                Self::LoadComplete => target_variant!("LoadComplete"),
                Self::Panic(_, _) => target_variant!("Panic"),
                Self::GetSelectedBluetoothDevice(_, _) => {
                    target_variant!("GetSelectedBluetoothDevice")
                },
                Self::SelectFiles(_, _, _) => target_variant!("SelectFiles"),
                Self::PromptPermission(_, _) => target_variant!("PromptPermission"),
                Self::ShowIME(_, _, _, _) => target_variant!("ShowIME"),
                Self::HideIME => target_variant!("HideIME"),
                Self::Shutdown => target_variant!("Shutdown"),
                Self::ReportProfile(_) => target_variant!("ReportProfile"),
                Self::MediaSessionEvent(_) => target_variant!("MediaSessionEvent"),
                Self::OnDevtoolsStarted(_, _) => target_variant!("OnDevtoolsStarted"),
                Self::ReadyToPresent => target_variant!("ReadyToPresent"),
                Self::EventDelivered(_) => target_variant!("EventDelivered"),
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
                Self::IFrameSizes(_) => target!("IFrameSizes"),
                Self::PendingPaintMetric(_, _) => target!("PendingPaintMetric"),
            }
        }
    }
}
