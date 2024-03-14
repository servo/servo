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
        ::log::trace!(target: $crate::tracing::LogTarget::log_target(&$event), $($rest)+)
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
        ::log::trace!(target: $crate::tracing::LogTarget::log_target(&$event), $($rest)+)
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
        ::log::trace!(target: $crate::tracing::LogTarget::log_target(&$event), $($rest)+)
    };
}

/// Get the log target for an event, as a static string.
pub(crate) trait LogTarget {
    fn log_target(&self) -> &'static str;
}

mod from_winit {
    use super::LogTarget;
    use crate::events_loop::WakerEvent;

    macro_rules! target {
        ($($name:literal)+) => {
            concat!("servoshell<winit@", $($name),+)
        };
    }

    impl LogTarget for winit::event::Event<'_, WakerEvent> {
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
                Self::MainEventsCleared => target!("MainEventsCleared"),
                Self::RedrawRequested(_) => target!("RedrawRequested"),
                Self::RedrawEventsCleared => target!("RedrawEventsCleared"),
                Self::LoopDestroyed => target!("LoopDestroyed"),
            }
        }
    }

    impl LogTarget for winit::event::WindowEvent<'_> {
        fn log_target(&self) -> &'static str {
            macro_rules! target_variant {
                ($name:literal) => {
                    target!("WindowEvent(" $name ")")
                };
            }
            match self {
                Self::Resized(_) => target_variant!("Resized"),
                Self::Moved(_) => target_variant!("Moved"),
                Self::CloseRequested => target_variant!("CloseRequested"),
                Self::Destroyed => target_variant!("Destroyed"),
                Self::DroppedFile(_) => target_variant!("DroppedFile"),
                Self::HoveredFile(_) => target_variant!("HoveredFile"),
                Self::HoveredFileCancelled => target_variant!("HoveredFileCancelled"),
                Self::ReceivedCharacter(_) => target_variant!("ReceivedCharacter"),
                Self::Focused(_) => target_variant!("Focused"),
                Self::KeyboardInput { .. } => target_variant!("KeyboardInput"),
                Self::ModifiersChanged(_) => target_variant!("ModifiersChanged"),
                Self::Ime(_) => target_variant!("Ime"),
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
                Self::Touch(_) => target_variant!("Touch"),
                Self::ScaleFactorChanged { .. } => target_variant!("ScaleFactorChanged"),
                Self::ThemeChanged(_) => target_variant!("ThemeChanged"),
                Self::Occluded(_) => target_variant!("Occluded"),
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
                Self::Status(_) => target!("Status"),
                Self::ChangePageTitle(_) => target!("ChangePageTitle"),
                Self::MoveTo(_) => target!("MoveTo"),
                Self::ResizeTo(_) => target!("ResizeTo"),
                Self::Prompt(_, _) => target!("Prompt"),
                Self::ShowContextMenu(_, _, _) => target!("ShowContextMenu"),
                Self::AllowNavigationRequest(_, _) => target!("AllowNavigationRequest"),
                Self::AllowOpeningWebView(_) => target!("AllowOpeningWebView"),
                Self::WebViewOpened(_) => target!("WebViewOpened"),
                Self::WebViewClosed(_) => target!("WebViewClosed"),
                Self::WebViewFocused(_) => target!("WebViewFocused"),
                Self::WebViewBlurred => target!("WebViewBlurred"),
                Self::AllowUnload(_) => target!("AllowUnload"),
                Self::Keyboard(_) => target!("Keyboard"),
                Self::GetClipboardContents(_) => target!("GetClipboardContents"),
                Self::SetClipboardContents(_) => target!("SetClipboardContents"),
                Self::SetCursor(_) => target!("SetCursor"),
                Self::NewFavicon(_) => target!("NewFavicon"),
                Self::HeadParsed => target!("HeadParsed"),
                Self::HistoryChanged(_, _) => target!("HistoryChanged"),
                Self::SetFullscreenState(_) => target!("SetFullscreenState"),
                Self::LoadStart => target!("LoadStart"),
                Self::LoadComplete => target!("LoadComplete"),
                Self::Panic(_, _) => target!("Panic"),
                Self::GetSelectedBluetoothDevice(_, _) => target!("GetSelectedBluetoothDevice"),
                Self::SelectFiles(_, _, _) => target!("SelectFiles"),
                Self::PromptPermission(_, _) => target!("PromptPermission"),
                Self::ShowIME(_, _, _, _) => target!("ShowIME"),
                Self::HideIME => target!("HideIME"),
                Self::Shutdown => target!("Shutdown"),
                Self::ReportProfile(_) => target!("ReportProfile"),
                Self::MediaSessionEvent(_) => target!("MediaSessionEvent"),
                Self::OnDevtoolsStarted(_, _) => target!("OnDevtoolsStarted"),
                Self::ReadyToPresent => target!("ReadyToPresent"),
                Self::EventDelivered(_) => target!("EventDelivered"),
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
                Self::Resize => target!("Resize"),
                Self::AllowNavigationResponse(_, _) => target!("AllowNavigationResponse"),
                Self::LoadUrl(_, _) => target!("LoadUrl"),
                Self::MouseWindowEventClass(_) => target!("MouseWindowEventClass"),
                Self::MouseWindowMoveEventClass(_) => target!("MouseWindowMoveEventClass"),
                Self::Touch(_, _, _) => target!("Touch"),
                Self::Wheel(_, _) => target!("Wheel"),
                Self::Scroll(_, _, _) => target!("Scroll"),
                Self::Zoom(_) => target!("Zoom"),
                Self::PinchZoom(_) => target!("PinchZoom"),
                Self::ResetZoom => target!("ResetZoom"),
                Self::Navigation(_, _) => target!("Navigation"),
                Self::Quit => target!("Quit"),
                Self::ExitFullScreen(_) => target!("ExitFullScreen"),
                Self::Keyboard(_) => target!("Keyboard"),
                Self::Reload(_) => target!("Reload"),
                Self::NewWebView(_, _) => target!("NewWebView"),
                Self::CloseWebView(_) => target!("CloseWebView"),
                Self::SendError(_, _) => target!("SendError"),
                Self::FocusWebView(_) => target!("FocusWebView"),
                Self::ToggleWebRenderDebug(_) => target!("ToggleWebRenderDebug"),
                Self::CaptureWebRender => target!("CaptureWebRender"),
                Self::ClearCache => target!("ClearCache"),
                Self::ToggleSamplingProfiler(_, _) => target!("ToggleSamplingProfiler"),
                Self::MediaSessionAction(_) => target!("MediaSessionAction"),
                Self::WebViewVisibilityChanged(_, _) => target!("WebViewVisibilityChanged"),
                Self::IMEDismissed => target!("IMEDismissed"),
                Self::InvalidateNativeSurface => target!("InvalidateNativeSurface"),
                Self::ReplaceNativeSurface(_, _) => target!("ReplaceNativeSurface"),
                Self::Gamepad(_) => target!("Gamepad"),
            }
        }
    }
}
