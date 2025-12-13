/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The interface to the `Constellation`, which prevents other crates from depending directly on
//! the `constellation` crate itself. In addition to all messages to the `Constellation`, this
//! crate is responsible for defining types that cross the process boundary from the
//! embedding/rendering layer all the way to script, thus it should have very minimal dependencies
//! on other parts of Servo.

mod from_script_message;
mod structured_data;

use std::collections::VecDeque;
use std::fmt;
use std::time::Duration;

use base::cross_process_instant::CrossProcessInstant;
use base::id::{MessagePortId, PipelineId, ScriptEventLoopId, WebViewId};
use compositing_traits::largest_contentful_paint_candidate::LargestContentfulPaintType;
use embedder_traits::{
    EmbedderControlId, EmbedderControlResponse, InputEventAndId, JavaScriptEvaluationId,
    MediaSessionActionType, PaintHitTestResult, Theme, TraversalId, ViewportDetails,
    WebDriverCommandMsg,
};
pub use from_script_message::*;
use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::MemoryReportResult;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_config::prefs::PrefValue;
use servo_url::{ImmutableOrigin, ServoUrl};
pub use structured_data::*;
use strum::IntoStaticStr;
use webrender_api::units::LayoutVector2D;
use webrender_api::{ExternalScrollId, ImageKey};

/// Messages to the Constellation from the embedding layer, whether from `ServoRenderer` or
/// from `libservo` itself.
#[derive(IntoStaticStr)]
pub enum EmbedderToConstellationMessage {
    /// Exit the constellation.
    Exit,
    /// Whether to allow script to navigate.
    AllowNavigationResponse(PipelineId, bool),
    /// Request to load a page.
    LoadUrl(WebViewId, ServoUrl),
    /// Request to traverse the joint session history of the provided browsing context.
    TraverseHistory(WebViewId, TraversalDirection, TraversalId),
    /// Inform the Constellation that a `WebView`'s [`ViewportDetails`] have changed.
    ChangeViewportDetails(WebViewId, ViewportDetails, WindowSizeType),
    /// Inform the constellation of a theme change.
    ThemeChange(WebViewId, Theme),
    /// Requests that the constellation instruct script/layout to try to layout again and tick
    /// animations.
    TickAnimation(Vec<WebViewId>),
    /// Notify the `ScriptThread` that the Servo renderer is no longer waiting on
    /// asynchronous image uploads for the given `Pipeline`. These are mainly used
    /// by canvas to perform uploads while the display list is being built.
    NoLongerWaitingOnAsynchronousImageUpdates(Vec<PipelineId>),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload a top-level browsing context.
    Reload(WebViewId),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<ScriptEventLoopId>, Option<String>, LogEntry),
    /// Create a new top level browsing context.
    NewWebView(ServoUrl, WebViewId, ViewportDetails),
    /// Close a top level browsing context.
    CloseWebView(WebViewId),
    /// Panic a top level browsing context.
    SendError(Option<WebViewId>, String),
    /// Make a webview focused. [EmbedderMsg::WebViewFocused] will be sent with
    /// the result of this operation.
    FocusWebView(WebViewId),
    /// Make none of the webviews focused.
    BlurWebView,
    /// Forward an input event to an appropriate ScriptTask.
    ForwardInputEvent(WebViewId, InputEventAndId, Option<PaintHitTestResult>),
    /// Request that the given pipeline refresh the cursor by doing a hit test at the most
    /// recently hovered cursor position and resetting the cursor. This happens after a
    /// display list update is rendered.
    RefreshCursor(PipelineId),
    /// Enable the sampling profiler, with a given sampling rate and max total sampling duration.
    ToggleProfiler(Duration, Duration),
    /// Request to exit from fullscreen mode
    ExitFullScreen(WebViewId),
    /// Media session action.
    MediaSessionAction(MediaSessionActionType),
    /// Set whether to use less resources, by stopping animations and running timers at a heavily limited rate.
    SetWebViewThrottled(WebViewId, bool),
    /// The Servo renderer scrolled and is updating the scroll states of the nodes in the
    /// given pipeline via the constellation.
    SetScrollStates(PipelineId, FxHashMap<ExternalScrollId, LayoutVector2D>),
    /// Notify the constellation that a particular paint metric event has happened for the given pipeline.
    PaintMetric(PipelineId, PaintMetricEvent),
    /// Evaluate a JavaScript string in the context of a `WebView`. When execution is complete or an
    /// error is encountered, a correpsonding message will be sent to the embedding layer.
    EvaluateJavaScript(WebViewId, JavaScriptEvaluationId, String),
    /// Create a memory report and return it via the ipc sender
    CreateMemoryReport(IpcSender<MemoryReportResult>),
    /// Sends the generated image key to the image cache associated with this pipeline.
    SendImageKeysForPipeline(PipelineId, Vec<ImageKey>),
    /// A set of preferences were updated with the given new values.
    PreferencesUpdated(Vec<(&'static str, PrefValue)>),
    /// Request preparation for a screenshot of the given WebView. The Constellation will
    /// send a message to the Embedder when the screenshot is ready to be taken.
    RequestScreenshotReadiness(WebViewId),
    /// A response to a request to show an embedder user interface control.
    EmbedderControlResponse(EmbedderControlId, EmbedderControlResponse),
}

/// A description of a paint metric that is sent from the Servo renderer to the
/// constellation.
pub enum PaintMetricEvent {
    FirstPaint(CrossProcessInstant, bool /* first_reflow */),
    FirstContentfulPaint(CrossProcessInstant, bool /* first_reflow */),
    LargestContentfulPaint(
        CrossProcessInstant,
        usize, /* area */
        LargestContentfulPaintType,
    ),
}

impl fmt::Debug for EmbedderToConstellationMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let variant_string: &'static str = self.into();
        write!(formatter, "ConstellationMsg::{variant_string}")
    }
}

/// A log entry reported to the constellation
/// We don't report all log entries, just serious ones.
/// We need a separate type for this because `LogLevel` isn't serializable.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntry {
    /// Panic, with a reason and backtrace
    Panic(String, String),
    /// Error, with a reason
    Error(String),
    /// warning, with a reason
    Warn(String),
}

/// The type of window size change.
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum WindowSizeType {
    /// Initial load.
    Initial,
    /// Window resize.
    Resize,
}

/// The direction of a history traversal
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TraversalDirection {
    /// Travel forward the given number of documents.
    Forward(usize),
    /// Travel backward the given number of documents.
    Back(usize),
}

/// A task on the <https://html.spec.whatwg.org/multipage/#port-message-queue>
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PortMessageTask {
    /// The origin of this task.
    pub origin: ImmutableOrigin,
    /// A data-holder for serialized data and transferred objects.
    pub data: StructuredSerializedData,
}

/// The information needed by a global to process the transfer of a port.
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PortTransferInfo {
    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    pub port_message_queue: VecDeque<PortMessageTask>,
    /// A boolean indicating whether the port has been disentangled while in transfer,
    /// if so, the disentanglement should be completed along with the transfer.
    /// <https://html.spec.whatwg.org/multipage/#disentangle>
    pub disentangled: bool,
}

/// Messages for communication between the constellation and a global managing ports.
#[derive(Debug, Deserialize, Serialize)]
#[expect(clippy::large_enum_variant)]
pub enum MessagePortMsg {
    /// Complete the transfer for a batch of ports.
    CompleteTransfer(FxHashMap<MessagePortId, PortTransferInfo>),
    /// Complete the transfer of a single port,
    /// whose transfer was pending because it had been requested
    /// while a previous failed transfer was being rolled-back.
    CompletePendingTransfer(MessagePortId, PortTransferInfo),
    /// <https://html.spec.whatwg.org/multipage/#disentangle>
    CompleteDisentanglement(MessagePortId),
    /// Handle a new port-message-task.
    NewTask(MessagePortId, PortMessageTask),
}
