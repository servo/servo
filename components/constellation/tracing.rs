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
    use embedder_traits::InputEvent;

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
                Self::AllowNavigationResponse(..) => target!("AllowNavigationResponse"),
                Self::LoadUrl(..) => target!("LoadUrl"),
                Self::ClearCache => target!("ClearCache"),
                Self::TraverseHistory(..) => target!("TraverseHistory"),
                Self::WindowSize(..) => target!("WindowSize"),
                Self::ThemeChange(..) => target!("ThemeChange"),
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
                Self::ForwardInputEvent(event, ..) => event.log_target(),
                Self::SetCursor(..) => target!("SetCursor"),
                Self::ToggleProfiler(..) => target!("EnableProfiler"),
                Self::ExitFullScreen(_) => target!("ExitFullScreen"),
                Self::MediaSessionAction(_) => target!("MediaSessionAction"),
                Self::SetWebViewThrottled(_, _) => target!("SetWebViewThrottled"),
            }
        }
    }

    impl LogTarget for InputEvent {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("ForwardInputEvent(" $name ")")
                };
            }
            match self {
                InputEvent::EditingAction(..) => target_variant!("EditingAction"),
                InputEvent::Gamepad(..) => target_variant!("Gamepad"),
                InputEvent::Ime(..) => target_variant!("Ime"),
                InputEvent::Keyboard(..) => target_variant!("Keyboard"),
                InputEvent::MouseButton(..) => target_variant!("MouseButton"),
                InputEvent::MouseMove(..) => target_variant!("MouseMove"),
                InputEvent::Touch(..) => target_variant!("Touch"),
                InputEvent::Wheel(..) => target_variant!("Wheel"),
            }
        }
    }
}

mod from_script {
    use embedder_traits::LoadStatus;

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
                Self::MediaSessionEvent(..) => target!("MediaSessionEvent"),
                #[cfg(feature = "webgpu")]
                Self::RequestAdapter(..) => target!("RequestAdapter"),
                #[cfg(feature = "webgpu")]
                Self::GetWebGPUChan(..) => target!("GetWebGPUChan"),
                Self::TitleChanged(..) => target!("TitleChanged"),
                Self::IFrameSizes(..) => target!("IFrameSizes"),
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
                Self::RequestAuthentication(..) => target_variant!("RequestAuthentication"),
                Self::ShowContextMenu(..) => target_variant!("ShowContextMenu"),
                Self::AllowNavigationRequest(..) => target_variant!("AllowNavigationRequest"),
                Self::AllowOpeningWebView(..) => target_variant!("AllowOpeningWebView"),
                Self::WebViewOpened(..) => target_variant!("WebViewOpened"),
                Self::WebViewClosed(..) => target_variant!("WebViewClosed"),
                Self::WebViewFocused(..) => target_variant!("WebViewFocused"),
                Self::WebViewBlurred => target_variant!("WebViewBlurred"),
                Self::WebResourceRequested(..) => target_variant!("WebResourceRequested"),
                Self::AllowUnload(..) => target_variant!("AllowUnload"),
                Self::Keyboard(..) => target_variant!("Keyboard"),
                Self::ClearClipboard(..) => target_variant!("ClearClipboard"),
                Self::GetClipboardText(..) => target_variant!("GetClipboardText"),
                Self::SetClipboardText(..) => target_variant!("SetClipboardText"),
                Self::SetCursor(..) => target_variant!("SetCursor"),
                Self::NewFavicon(..) => target_variant!("NewFavicon"),
                Self::HistoryChanged(..) => target_variant!("HistoryChanged"),
                Self::NotifyFullscreenStateChanged(..) => {
                    target_variant!("NotifyFullscreenStateChanged")
                },
                Self::NotifyLoadStatusChanged(_, LoadStatus::Started) => {
                    target_variant!("NotifyLoadStatusChanged(LoadStatus::Started)")
                },
                Self::NotifyLoadStatusChanged(_, LoadStatus::HeadParsed) => {
                    target_variant!("NotifyLoadStatusChanged(LoadStatus::HeadParsed)")
                },
                Self::NotifyLoadStatusChanged(_, LoadStatus::Complete) => {
                    target_variant!("NotifyLoadStatusChanged(LoadStatus::Complete")
                },
                Self::Panic(..) => target_variant!("Panic"),
                #[cfg(feature = "bluetooth")]
                Self::GetSelectedBluetoothDevice(..) => {
                    target_variant!("GetSelectedBluetoothDevice")
                },
                Self::SelectFiles(..) => target_variant!("SelectFiles"),
                Self::PromptPermission(..) => target_variant!("PromptPermission"),
                Self::ShowIME(..) => target_variant!("ShowIME"),
                Self::HideIME(..) => target_variant!("HideIME"),
                Self::ReportProfile(..) => target_variant!("ReportProfile"),
                Self::MediaSessionEvent(..) => target_variant!("MediaSessionEvent"),
                Self::OnDevtoolsStarted(..) => target_variant!("OnDevtoolsStarted"),
                Self::RequestDevtoolsConnection(..) => target_variant!("RequestDevtoolsConnection"),
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
                Self::PendingPaintMetric(..) => target!("PendingPaintMetric"),
            }
        }
    }
}
