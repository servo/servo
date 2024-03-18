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

    impl LogTarget for winit::event::Event<'_, WakerEvent> {
        fn log_target(&self) -> &'static str {
            use winit::event::{StartCause, WindowEvent};
            event_tracing::log_target!(self {
                servoshell<winit@NewEvents(start_cause) => start_cause as StartCause {
                    ResumeTimeReached { .. },
                    WaitCancelled { .. },
                    Poll,
                    Init,
                },
                servoshell<winit@WindowEvent { event, .. } => event as WindowEvent {
                    Resized(_),
                    Moved(_),
                    CloseRequested,
                    Destroyed,
                    DroppedFile(_),
                    HoveredFile(_),
                    HoveredFileCancelled,
                    ReceivedCharacter(_),
                    Focused(_),
                    KeyboardInput { .. },
                    ModifiersChanged(_),
                    Ime(_),
                    CursorMoved { .. },
                    CursorEntered { .. },
                    CursorLeft { .. },
                    MouseWheel { .. },
                    MouseInput { .. },
                    TouchpadMagnify { .. },
                    SmartMagnify { .. },
                    TouchpadRotate { .. },
                    TouchpadPressure { .. },
                    AxisMotion { .. },
                    Touch(_),
                    ScaleFactorChanged { .. },
                    ThemeChanged(_),
                    Occluded(_),
                },
                servoshell<winit@DeviceEvent { .. },
                servoshell<winit@UserEvent(WakerEvent),
                servoshell<winit@Suspended,
                servoshell<winit@Resumed,
                servoshell<winit@MainEventsCleared,
                servoshell<winit@RedrawRequested(..),
                servoshell<winit@RedrawEventsCleared,
                servoshell<winit@LoopDestroyed,
            })
        }
    }
}

mod from_servo {
    use super::LogTarget;

    impl LogTarget for servo::embedder_traits::EmbedderMsg {
        fn log_target(&self) -> &'static str {
            event_tracing::log_target!(self {
                servoshell<servo@Status(..),
                servoshell<servo@ChangePageTitle(..),
                servoshell<servo@MoveTo(..),
                servoshell<servo@ResizeTo(..),
                servoshell<servo@Prompt(..),
                servoshell<servo@ShowContextMenu(..),
                servoshell<servo@AllowNavigationRequest(..),
                servoshell<servo@AllowOpeningWebView(..),
                servoshell<servo@WebViewOpened(..),
                servoshell<servo@WebViewClosed(..),
                servoshell<servo@WebViewFocused(..),
                servoshell<servo@WebViewBlurred,
                servoshell<servo@AllowUnload(..),
                servoshell<servo@Keyboard(..),
                servoshell<servo@GetClipboardContents(..),
                servoshell<servo@SetClipboardContents(..),
                servoshell<servo@SetCursor(..),
                servoshell<servo@NewFavicon(..),
                servoshell<servo@HeadParsed,
                servoshell<servo@HistoryChanged(..),
                servoshell<servo@SetFullscreenState(..),
                servoshell<servo@LoadStart,
                servoshell<servo@LoadComplete,
                servoshell<servo@Panic(..),
                servoshell<servo@GetSelectedBluetoothDevice(..),
                servoshell<servo@SelectFiles(..),
                servoshell<servo@PromptPermission(..),
                servoshell<servo@ShowIME(..),
                servoshell<servo@HideIME,
                servoshell<servo@Shutdown,
                servoshell<servo@ReportProfile(..),
                servoshell<servo@MediaSessionEvent(..),
                servoshell<servo@OnDevtoolsStarted(..),
                servoshell<servo@ReadyToPresent,
                servoshell<servo@EventDelivered(..),
            })
        }
    }
}

mod to_servo {
    use super::LogTarget;

    impl LogTarget for servo::compositing::windowing::EmbedderEvent {
        fn log_target(&self) -> &'static str {
            event_tracing::log_target!(self {
                servoshell>servo@Idle,
                servoshell>servo@Refresh,
                servoshell>servo@Resize,
                servoshell>servo@AllowNavigationResponse(..),
                servoshell>servo@LoadUrl(..),
                servoshell>servo@MouseWindowEventClass(..),
                servoshell>servo@MouseWindowMoveEventClass(..),
                servoshell>servo@Touch(..),
                servoshell>servo@Wheel(..),
                servoshell>servo@Scroll(..),
                servoshell>servo@Zoom(..),
                servoshell>servo@PinchZoom(..),
                servoshell>servo@ResetZoom,
                servoshell>servo@Navigation(..),
                servoshell>servo@Quit,
                servoshell>servo@ExitFullScreen(..),
                servoshell>servo@Keyboard(..),
                servoshell>servo@Reload(..),
                servoshell>servo@NewWebView(..),
                servoshell>servo@CloseWebView(..),
                servoshell>servo@SendError(..),
                servoshell>servo@FocusWebView(..),
                servoshell>servo@ToggleWebRenderDebug(..),
                servoshell>servo@CaptureWebRender,
                servoshell>servo@ClearCache,
                servoshell>servo@ToggleSamplingProfiler(..),
                servoshell>servo@MediaSessionAction(..),
                servoshell>servo@WebViewVisibilityChanged(..),
                servoshell>servo@IMEDismissed,
                servoshell>servo@InvalidateNativeSurface,
                servoshell>servo@ReplaceNativeSurface(..),
                servoshell>servo@Gamepad(..),
            })
        }
    }
}
