/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Log an event from winit ([winit::event::Event]) at trace level.
/// - To disable tracing: RUST_LOG='servoshell<winit@=off'
/// - To enable tracing: RUST_LOG='servoshell<winit@'
/// - Recommended filters when tracing is enabled:
///   - servoshell<winit@DeviceEvent=off
///   - servoshell<winit@MainEventsCleared=off
///   - servoshell<winit@NewEvents(WaitCancelled)=off
///   - servoshell<winit@RedrawEventsCleared=off
///   - servoshell<winit@RedrawRequested=off
///   - servoshell<winit@UserEvent(WakerEvent)=off
///   - servoshell<winit@WindowEvent(AxisMotion)=off
///   - servoshell<winit@WindowEvent(CursorMoved)=off
macro_rules! trace_winit_event {
    // This macro only exists to put the docs in the same file as the target prefix,
    // so the macro definition is always the same.
    ($event:expr, $($rest:tt)+) => {
        ::log::trace!(target: $crate::desktop::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

/// Log an event from servo ([servo::embedder_traits::EmbedderMsg]) at trace level.
/// - To disable tracing: RUST_LOG='servoshell<servo@=off'
/// - To enable tracing: RUST_LOG='servoshell<servo@'
/// - Recommended filters when tracing is enabled:
///   - servoshell<servo@EventDelivered=off
///   - servoshell<servo@ReadyToPresent=off
macro_rules! trace_embedder_msg {
    // This macro only exists to put the docs in the same file as the target prefix,
    // so the macro definition is always the same.
    ($event:expr, $($rest:tt)+) => {
        ::log::trace!(target: $crate::desktop::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

/// Log an event to servo ([servo::compositing::windowing::EmbedderEvent]) at trace level.
/// - To disable tracing: RUST_LOG='servoshell>servo@=off'
/// - To enable tracing: RUST_LOG='servoshell>servo@'
/// - Recommended filters when tracing is enabled:
///   - servoshell>servo@Idle=off
///   - servoshell>servo@MouseWindowMoveEventClass=off
macro_rules! trace_embedder_event {
    // This macro only exists to put the docs in the same file as the target prefix,
    // so the macro definition is always the same.
    ($event:expr, $($rest:tt)+) => {
        ::log::trace!(target: $crate::desktop::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

pub(crate) use {trace_embedder_event, trace_embedder_msg, trace_winit_event};

/// Get the log target for an event, as a static string.
pub(crate) trait LogTarget {
    fn log_target(&self) -> &'static str;
}

mod from_winit {
    use super::LogTarget;
    use crate::desktop::events_loop::WakerEvent;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("servoshell<winit@", $($name),+)
        };
    }

    impl LogTarget for winit::event::Event<WakerEvent> {
        fn log_target(&self) -> &'static str {
            use winit::event::StartCause;
            match self {
                Self::NewEvents(start_cause) => match start_cause {
                    StartCause::ResumeTimeReached { .. } => target!("NewEvents(ResumeTimeReached)"),
                    StartCause::WaitCancelled { .. } => target!("NewEvents(WaitCancelled)"),
                    StartCause::Poll => target!("NewEvents(Poll)"),
                    StartCause::Init => target!("NewEvents(Init)"),
                },
                Self::WindowEvent { event, .. } => event.log_target(),
                Self::DeviceEvent { .. } => target!("DeviceEvent"),
                Self::UserEvent(WakerEvent) => target!("UserEvent(WakerEvent)"),
                Self::Suspended => target!("Suspended"),
                Self::Resumed => target!("Resumed"),
                Self::AboutToWait => target!("AboutToWait"),
                Self::LoopExiting => target!("LoopExiting"),
                Self::MemoryWarning => target!("MemoryWarning"),
            }
        }
    }

    impl LogTarget for winit::event::WindowEvent {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("WindowEvent(" $name ")")
                };
            }
            match self {
                Self::ActivationTokenDone { .. } => target!("ActivationTokenDone"),
                Self::Resized(..) => target_variant!("Resized"),
                Self::Moved(..) => target_variant!("Moved"),
                Self::CloseRequested => target_variant!("CloseRequested"),
                Self::Destroyed => target_variant!("Destroyed"),
                Self::DroppedFile(..) => target_variant!("DroppedFile"),
                Self::HoveredFile(..) => target_variant!("HoveredFile"),
                Self::HoveredFileCancelled => target_variant!("HoveredFileCancelled"),
                Self::Focused(..) => target_variant!("Focused"),
                Self::KeyboardInput { .. } => target_variant!("KeyboardInput"),
                Self::ModifiersChanged(..) => target_variant!("ModifiersChanged"),
                Self::Ime(..) => target_variant!("Ime"),
                Self::CursorMoved { .. } => target_variant!("CursorMoved"),
                Self::CursorEntered { .. } => target_variant!("CursorEntered"),
                Self::CursorLeft { .. } => target_variant!("CursorLeft"),
                Self::MouseWheel { .. } => target_variant!("MouseWheel"),
                Self::MouseInput { .. } => target_variant!("MouseInput"),
                Self::TouchpadMagnify { .. } => target_variant!("TouchpadMagnify"),
                Self::SmartMagnify { .. } => target_variant!("SmartMagnify"),
                Self::TouchpadRotate { .. } => target_variant!("TouchpadRotate"),
                Self::TouchpadPressure { .. } => target_variant!("TouchpadPressure"),
                Self::AxisMotion { .. } => target_variant!("AxisMotion"),
                Self::Touch(..) => target_variant!("Touch"),
                Self::ScaleFactorChanged { .. } => target_variant!("ScaleFactorChanged"),
                Self::ThemeChanged(..) => target_variant!("ThemeChanged"),
                Self::Occluded(..) => target_variant!("Occluded"),
                Self::RedrawRequested => target!("RedrawRequested"),
            }
        }
    }
}

mod from_servo {
    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("servoshell<servo@", $($name),+)
        };
    }

    impl LogTarget for servo::embedder_traits::EmbedderMsg {
        fn log_target(&self) -> &'static str {
            match self {
                Self::Status(..) => target!("Status"),
                Self::ChangePageTitle(..) => target!("ChangePageTitle"),
                Self::MoveTo(..) => target!("MoveTo"),
                Self::ResizeTo(..) => target!("ResizeTo"),
                Self::Prompt(..) => target!("Prompt"),
                Self::ShowContextMenu(..) => target!("ShowContextMenu"),
                Self::AllowNavigationRequest(..) => target!("AllowNavigationRequest"),
                Self::AllowOpeningWebView(..) => target!("AllowOpeningWebView"),
                Self::WebViewOpened(..) => target!("WebViewOpened"),
                Self::WebViewClosed(..) => target!("WebViewClosed"),
                Self::WebViewFocused(..) => target!("WebViewFocused"),
                Self::WebViewBlurred => target!("WebViewBlurred"),
                Self::AllowUnload(..) => target!("AllowUnload"),
                Self::Keyboard(..) => target!("Keyboard"),
                Self::GetClipboardContents(..) => target!("GetClipboardContents"),
                Self::SetClipboardContents(..) => target!("SetClipboardContents"),
                Self::SetCursor(..) => target!("SetCursor"),
                Self::NewFavicon(..) => target!("NewFavicon"),
                Self::HeadParsed => target!("HeadParsed"),
                Self::HistoryChanged(..) => target!("HistoryChanged"),
                Self::SetFullscreenState(..) => target!("SetFullscreenState"),
                Self::LoadStart => target!("LoadStart"),
                Self::LoadComplete => target!("LoadComplete"),
                Self::Panic(..) => target!("Panic"),
                Self::GetSelectedBluetoothDevice(..) => target!("GetSelectedBluetoothDevice"),
                Self::SelectFiles(..) => target!("SelectFiles"),
                Self::PromptPermission(..) => target!("PromptPermission"),
                Self::ShowIME(..) => target!("ShowIME"),
                Self::HideIME => target!("HideIME"),
                Self::Shutdown => target!("Shutdown"),
                Self::ReportProfile(..) => target!("ReportProfile"),
                Self::MediaSessionEvent(..) => target!("MediaSessionEvent"),
                Self::OnDevtoolsStarted(..) => target!("OnDevtoolsStarted"),
                Self::ReadyToPresent(..) => target!("ReadyToPresent"),
                Self::EventDelivered(..) => target!("EventDelivered"),
                Self::PlayGamepadHapticEffect(..) => target!("PlayGamepadHapticEffect"),
                Self::StopGamepadHapticEffect(..) => target!("StopGamepadHapticEffect"),
            }
        }
    }
}

mod to_servo {
    use super::LogTarget;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("servoshell>servo@", $($name),+)
        };
    }

    impl LogTarget for servo::compositing::windowing::EmbedderEvent {
        fn log_target(&self) -> &'static str {
            match self {
                Self::Idle => target!("Idle"),
                Self::Refresh => target!("Refresh"),
                Self::WindowResize => target!("WindowResize"),
                Self::AllowNavigationResponse(..) => target!("AllowNavigationResponse"),
                Self::LoadUrl(..) => target!("LoadUrl"),
                Self::MouseWindowEventClass(..) => target!("MouseWindowEventClass"),
                Self::MouseWindowMoveEventClass(..) => target!("MouseWindowMoveEventClass"),
                Self::Touch(..) => target!("Touch"),
                Self::Wheel(..) => target!("Wheel"),
                Self::Scroll(..) => target!("Scroll"),
                Self::Zoom(..) => target!("Zoom"),
                Self::PinchZoom(..) => target!("PinchZoom"),
                Self::ResetZoom => target!("ResetZoom"),
                Self::Navigation(..) => target!("Navigation"),
                Self::Quit => target!("Quit"),
                Self::ExitFullScreen(..) => target!("ExitFullScreen"),
                Self::Keyboard(..) => target!("Keyboard"),
                Self::Reload(..) => target!("Reload"),
                Self::NewWebView(..) => target!("NewWebView"),
                Self::CloseWebView(..) => target!("CloseWebView"),
                Self::SendError(..) => target!("SendError"),
                Self::MoveResizeWebView(..) => target!("MoveResizeWebView"),
                Self::ShowWebView(..) => target!("ShowWebView"),
                Self::HideWebView(..) => target!("HideWebView"),
                Self::RaiseWebViewToTop(..) => target!("RaiseWebViewToTop"),
                Self::FocusWebView(..) => target!("FocusWebView"),
                Self::BlurWebView => target!("BlurWebView"),
                Self::ToggleWebRenderDebug(..) => target!("ToggleWebRenderDebug"),
                Self::CaptureWebRender => target!("CaptureWebRender"),
                Self::ClearCache => target!("ClearCache"),
                Self::ToggleSamplingProfiler(..) => target!("ToggleSamplingProfiler"),
                Self::MediaSessionAction(..) => target!("MediaSessionAction"),
                Self::SetWebViewThrottled(..) => target!("SetWebViewThrottled"),
                Self::IMEDismissed => target!("IMEDismissed"),
                Self::InvalidateNativeSurface => target!("InvalidateNativeSurface"),
                Self::ReplaceNativeSurface(..) => target!("ReplaceNativeSurface"),
                Self::Gamepad(..) => target!("Gamepad"),
            }
        }
    }
}
