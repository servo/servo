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

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use embedder_traits::user_contents::{
    UserContentManagerId, UserScript, UserScriptId, UserStyleSheet, UserStyleSheetId,
};
use embedder_traits::{
    EmbedderControlId, EmbedderControlResponse, InputEventAndId, JavaScriptEvaluationId,
    MediaSessionActionType, NewWebViewDetails, PaintHitTestResult, Theme, TraversalId,
    ViewportDetails, WebDriverCommandMsg,
};
pub use from_script_message::*;
use http::HeaderMap;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use paint_api::PinchZoomInfos;
use profile_traits::mem::MemoryReportResult;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_base::generic_channel::GenericCallback;
use servo_base::id::{MessagePortId, PipelineId, ScriptEventLoopId, WebViewId};
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
    /// Request to load a page, with optionally additional data in [`URLRequest`].
    LoadUrl(WebViewId, UrlRequest),
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
    NewWebView(ServoUrl, NewWebViewDetails),
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
    SetScrollStates(PipelineId, ScrollStateUpdate),
    /// Notify the constellation that a particular paint metric event has happened for the given pipeline.
    PaintMetric(PipelineId, PaintMetricEvent),
    /// Evaluate a JavaScript string in the context of a `WebView`. When execution is complete or an
    /// error is encountered, a correpsonding message will be sent to the embedding layer.
    EvaluateJavaScript(WebViewId, JavaScriptEvaluationId, String),
    /// Create a memory report and return it via the [`GenericCallback`]
    CreateMemoryReport(GenericCallback<MemoryReportResult>),
    /// Sends the generated image key to the image cache associated with this pipeline.
    SendImageKeysForPipeline(PipelineId, Vec<ImageKey>),
    /// A set of preferences were updated with the given new values.
    PreferencesUpdated(Vec<(&'static str, PrefValue)>),
    /// Request preparation for a screenshot of the given WebView. The Constellation will
    /// send a message to the Embedder when the screenshot is ready to be taken.
    RequestScreenshotReadiness(WebViewId),
    /// A response to a request to show an embedder user interface control.
    EmbedderControlResponse(EmbedderControlId, EmbedderControlResponse),
    /// An action to perform on the given `UserContentManagerId`.
    UserContentManagerAction(UserContentManagerId, UserContentManagerAction),
    /// Update pinch zoom details stored in the top level window
    UpdatePinchZoomInfos(PipelineId, PinchZoomInfos),
    /// Activate or deactivate accessibility features for the given `WebView`.
    SetAccessibilityActive(WebViewId, bool),
}

pub enum UserContentManagerAction {
    AddUserScript(UserScript),
    DestroyUserContentManager,
    RemoveUserScript(UserScriptId),
    AddUserStyleSheet(UserStyleSheet),
    RemoveUserStyleSheet(UserStyleSheetId),
}

#[derive(Serialize, Deserialize, Debug)]
/// Data that can be additionally loaded for a given url such as extra headers
/// or other data.
/// ```
/// let url_request = URLRequest::new().headers(headers);
/// webview.load_url(url, Some(url_request));
/// ```
pub struct UrlRequest {
    url: ServoUrl,
    #[serde(
        deserialize_with = "hyper_serde::deserialize",
        serialize_with = "hyper_serde::serialize"
    )]
    headers: HeaderMap,
    data: Option<Vec<u8>>,
    mime_type: Option<String>,
    encoding: Option<String>,
    base_url: Option<String>,
}

impl UrlRequest {
    pub fn new(url: ServoUrl) -> Self {
        UrlRequest {
            url,
            headers: HeaderMap::new(),
            data: None,
            mime_type: None,
            encoding: None,
            base_url: None,
        }
    }

    /// Set headers that will be added to the Headers
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Loads the given data into the webview with `mime_type` and using the `encoding`
    /// using `base_url` as the base url.
    pub fn data(
        mut self,
        data: Vec<u8>,
        mime_type: Option<String>,
        encoding: String,
        base_url: String,
    ) -> Self {
        self.data = Some(data);
        self.mime_type = Some(mime_type.unwrap_or_else(|| String::from("text/html")));
        self.encoding = Some(encoding);
        self.base_url = Some(base_url);
        self
    }

    /// Create a [`LoadData`] struct to be used by the constellation to load a webview.
    pub fn load_data(self) -> Option<LoadData> {
        let mut load_data = if let Some(data) = self.data {
            let data_url_string = format!(
                "data:{};charset={};base64,{}",
                self.mime_type.unwrap(),
                self.encoding.unwrap(),
                BASE64_STANDARD.encode(data)
            );

            let Ok(url) = ServoUrl::parse(&data_url_string) else {
                warn!("LoadUrl failed with failure to construct data URL");
                return None;
            };
            let mut load_data = LoadData::new_for_new_unrelated_webview(url);
            load_data.about_base_url = ServoUrl::parse(&self.base_url.unwrap()).ok();
            load_data
        } else {
            LoadData::new_for_new_unrelated_webview(self.url)
        };

        if !self.headers.is_empty() {
            load_data.headers.extend(self.headers);
        }

        Some(load_data)
    }
}

/// A description of a paint metric that is sent from the Servo renderer to the
/// constellation.
pub enum PaintMetricEvent {
    FirstPaint(CrossProcessInstant, bool /* first_reflow */),
    FirstContentfulPaint(CrossProcessInstant, bool /* first_reflow */),
    LargestContentfulPaint(CrossProcessInstant, usize /* area */, Option<ServoUrl>),
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

/// A data structure which contains information for the pipeline after a scroll happens in the
/// embedder-side `WebView`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ScrollStateUpdate {
    /// The [`ExternalScrollId`] of the node that that was scrolled.
    pub scrolled_node: ExternalScrollId,
    /// A map containing the scroll offsets of the entire scroll tree. This is necessary,
    /// because scroll events can cause other nodes to scroll due to sticky positioning.
    pub offsets: FxHashMap<ExternalScrollId, LayoutVector2D>,
}
