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
    use compositing_traits::ConstellationMsg;
    use script_traits::CompositorEvent;

    use super::LogTarget;

    impl LogTarget for ConstellationMsg {
        fn log_target(&self) -> &'static str {
            event_tracing::log_target!(self {
                constellation<compositor@Exit,
                constellation<none@GetBrowsingContext(..),
                constellation<none@GetPipeline(..),
                constellation<webdriver@GetFocusTopLevelBrowsingContext(..),
                constellation<compositor@IsReadyToSaveImage(..),
                constellation<embedder@Keyboard(..),
                constellation<embedder@AllowNavigationResponse(..),
                constellation<embedder@LoadUrl(..),
                constellation<embedder@ClearCache,
                constellation<embedder@TraverseHistory(..),
                constellation<compositor@WindowSize(..),
                constellation<compositor@TickAnimation(..),
                constellation<webdriver@WebDriverCommand(..),
                constellation<embedder@Reload(..),
                constellation<unknown@LogEntry(..),
                constellation<embedder@NewWebView(..),
                constellation<embedder@CloseWebView(..),
                constellation<embedder@SendError(..),
                constellation<embedder@FocusWebView(..),
                constellation<embedder@BlurWebView,
                constellation<compositor@ForwardEvent(_, event) => event as CompositorEvent {
                    ResizeEvent(..),
                    MouseButtonEvent(..),
                    MouseMoveEvent(..),
                    TouchEvent(..),
                    WheelEvent(..),
                    KeyboardEvent(..),
                    CompositionEvent(..),
                    IMEDismissedEvent,
                    GamepadEvent(..),
                },
                constellation<compositor@SetCursor(..),
                constellation<embedder@EnableProfiler(..),
                constellation<embedder@DisableProfiler,
                constellation<embedder@ExitFullScreen(..),
                constellation<embedder@MediaSessionAction(..),
                constellation<embedder@WebViewVisibilityChanged(..),
                constellation<embedder@IMEDismissed,
                constellation<compositor@ReadyToPresent(..),
                constellation<embedder@Gamepad(..),
            })
        }
    }
}

mod from_script {
    use embedder_traits::EmbedderMsg;

    use super::LogTarget;

    impl LogTarget for script_traits::ScriptMsg {
        fn log_target(&self) -> &'static str {
            event_tracing::log_target!(self {
                constellation<script@CompleteMessagePortTransfer(..),
                constellation<script@MessagePortTransferResult(..),
                constellation<script@NewMessagePort(..),
                constellation<script@NewMessagePortRouter(..),
                constellation<script@RemoveMessagePortRouter(..),
                constellation<script@RerouteMessagePort(..),
                constellation<script@MessagePortShipped(..),
                constellation<script@RemoveMessagePort(..),
                constellation<script@EntanglePorts(..),
                constellation<script@NewBroadcastChannelRouter(..),
                constellation<script@RemoveBroadcastChannelRouter(..),
                constellation<script@NewBroadcastChannelNameInRouter(..),
                constellation<script@RemoveBroadcastChannelNameInRouter(..),
                constellation<script@ScheduleBroadcast(..),
                constellation<script@ForwardToEmbedder(msg) => msg as EmbedderMsg {
                    Status(..),
                    ChangePageTitle(..),
                    MoveTo(..),
                    ResizeTo(..),
                    Prompt(..),
                    ShowContextMenu(..),
                    AllowNavigationRequest(..),
                    AllowOpeningWebView(..),
                    WebViewOpened(..),
                    WebViewClosed(..),
                    WebViewFocused(..),
                    WebViewBlurred,
                    AllowUnload(..),
                    Keyboard(..),
                    GetClipboardContents(..),
                    SetClipboardContents(..),
                    SetCursor(..),
                    NewFavicon(..),
                    HeadParsed,
                    HistoryChanged(..),
                    SetFullscreenState(..),
                    LoadStart,
                    LoadComplete,
                    Panic(..),
                    GetSelectedBluetoothDevice(..),
                    SelectFiles(..),
                    PromptPermission(..),
                    ShowIME(..),
                    HideIME,
                    Shutdown,
                    ReportProfile(..),
                    MediaSessionEvent(..),
                    OnDevtoolsStarted(..),
                    ReadyToPresent,
                    EventDelivered(..),
                },
                constellation<script@InitiateNavigateRequest(..),
                constellation<script@BroadcastStorageEvent(..),
                constellation<script@ChangeRunningAnimationsState(..),
                constellation<script@CreateCanvasPaintThread(..),
                constellation<script@Focus,
                constellation<script@GetTopForBrowsingContext(..),
                constellation<script@GetBrowsingContextInfo(..),
                constellation<script@GetChildBrowsingContextId(..),
                constellation<script@LoadComplete,
                constellation<script@LoadUrl(..),
                constellation<script@AbortLoadUrl,
                constellation<script@PostMessage { .. },
                constellation<script@NavigatedToFragment(..),
                constellation<script@TraverseHistory(..),
                constellation<script@PushHistoryState(..),
                constellation<script@ReplaceHistoryState(..),
                constellation<script@JointSessionHistoryLength(..),
                constellation<script@RemoveIFrame(..),
                constellation<script@VisibilityChangeComplete(..),
                constellation<script@ScriptLoadedURLInIFrame(..),
                constellation<script@ScriptNewIFrame(..),
                constellation<script@ScriptNewAuxiliary(..),
                constellation<script@ActivateDocument,
                constellation<script@SetDocumentState(..),
                constellation<script@SetLayoutEpoch(..),
                constellation<script@SetFinalUrl(..),
                constellation<script@TouchEventProcessed(..),
                constellation<script@LogEntry(..),
                constellation<script@DiscardDocument,
                constellation<script@DiscardTopLevelBrowsingContext,
                constellation<script@PipelineExited,
                constellation<script@ForwardDOMMessage(..),
                constellation<script@ScheduleJob(..),
                constellation<script@GetClientWindow(..),
                constellation<script@GetScreenSize(..),
                constellation<script@GetScreenAvailSize(..),
                constellation<script@MediaSessionEvent(..),
                constellation<script@RequestAdapter(..),
                constellation<script@GetWebGPUChan(..),
                constellation<script@TitleChanged(..),
            })
        }
    }
}

mod from_layout {
    use super::LogTarget;

    impl LogTarget for script_traits::LayoutMsg {
        fn log_target(&self) -> &'static str {
            event_tracing::log_target!(self {
                constellation<layout@IFrameSizes(..),
                constellation<layout@PendingPaintMetric(..),
            })
        }
    }
}
