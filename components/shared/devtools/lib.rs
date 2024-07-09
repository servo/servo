/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use std::net::TcpStream;
use std::time::{Duration, SystemTime};

use base::id::{BrowsingContextId, PipelineId};
use bitflags::bitflags;
use http::{HeaderMap, Method};
use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use uuid::Uuid;

// Information would be attached to NewGlobal to be received and show in devtools.
// Extend these fields if we need more information.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DevtoolsPageInfo {
    pub title: String,
    pub url: ServoUrl,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct CSSError {
    pub filename: String,
    pub line: u32,
    pub column: u32,
    pub msg: String,
}

/// Messages to instruct the devtools server to update its known actors/state
/// according to changes in the browser.
#[derive(Debug)]
pub enum DevtoolsControlMsg {
    /// Messages from threads in the chrome process (resource/constellation/devtools)
    FromChrome(ChromeToDevtoolsControlMsg),
    /// Messages from script threads
    FromScript(ScriptToDevtoolsControlMsg),
}

/// Events that the devtools server must act upon.
#[derive(Debug)]
pub enum ChromeToDevtoolsControlMsg {
    /// A new client has connected to the server.
    AddClient(TcpStream),
    /// The browser is shutting down.
    ServerExitMsg,
    /// A network event occurred (request, reply, etc.). The actor with the
    /// provided name should be notified.
    NetworkEvent(String, NetworkEvent),
}

/// The state of a page navigation.
#[derive(Debug, Deserialize, Serialize)]
pub enum NavigationState {
    /// A browsing context is about to navigate to a given URL.
    Start(ServoUrl),
    /// A browsing context has completed navigating to the provided pipeline.
    Stop(PipelineId, DevtoolsPageInfo),
}

#[derive(Debug, Deserialize, Serialize)]
/// Events that the devtools server must act upon.
pub enum ScriptToDevtoolsControlMsg {
    /// A new global object was created, associated with a particular pipeline.
    /// The means of communicating directly with it are provided.
    NewGlobal(
        (BrowsingContextId, PipelineId, Option<WorkerId>),
        IpcSender<DevtoolScriptControlMsg>,
        DevtoolsPageInfo,
    ),
    /// The given browsing context is performing a navigation.
    Navigate(BrowsingContextId, NavigationState),
    /// A particular page has invoked the console API.
    ConsoleAPI(PipelineId, ConsoleMessage, Option<WorkerId>),
    /// An animation frame with the given timestamp was processed in a script thread.
    /// The actor with the provided name should be notified.
    FramerateTick(String, f64),

    /// Report a CSS parse error for the given pipeline
    ReportCSSError(PipelineId, CSSError),

    /// Report a page error for the given pipeline
    ReportPageError(PipelineId, PageError),

    /// Report a page title change
    TitleChanged(PipelineId, String),
}

/// Serialized JS return values
/// TODO: generalize this beyond the EvaluateJS message?
#[derive(Debug, Deserialize, Serialize)]
pub enum EvaluateJSReply {
    VoidValue,
    NullValue,
    BooleanValue(bool),
    NumberValue(f64),
    StringValue(String),
    ActorValue { class: String, uuid: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttrInfo {
    pub namespace: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub unique_id: String,
    #[serde(rename = "baseURI")]
    pub base_uri: String,
    pub parent: String,
    pub node_type: u16,
    #[serde(rename = "namespaceURI")]
    pub namespace_uri: String,
    pub node_name: String,
    pub num_children: usize,
    pub name: String,
    pub public_id: String,
    pub system_id: String,
    pub attrs: Vec<AttrInfo>,
    pub is_document_element: bool,
    pub short_value: String,
    pub incomplete_value: bool,
}

pub struct StartedTimelineMarker {
    name: String,
    start_time: PreciseTime,
    start_stack: Option<Vec<()>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimelineMarker {
    pub name: String,
    pub start_time: PreciseTime,
    pub start_stack: Option<Vec<()>>,
    pub end_time: PreciseTime,
    pub end_stack: Option<Vec<()>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum TimelineMarkerType {
    Reflow,
    DOMEvent,
}

/// The properties of a DOM node as computed by layout.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedNodeLayout {
    pub display: String,
    pub position: String,
    pub z_index: String,
    pub box_sizing: String,

    pub auto_margins: AutoMargins,
    pub margin_top: String,
    pub margin_right: String,
    pub margin_bottom: String,
    pub margin_left: String,

    pub border_top_width: String,
    pub border_right_width: String,
    pub border_bottom_width: String,
    pub border_left_width: String,

    pub padding_top: String,
    pub padding_right: String,
    pub padding_bottom: String,
    pub padding_left: String,

    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutoMargins {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

/// Messages to process in a particular script thread, as instructed by a devtools client.
/// TODO: better error handling, e.g. if pipeline id lookup fails?
#[derive(Debug, Deserialize, Serialize)]
pub enum DevtoolScriptControlMsg {
    /// Evaluate a JS snippet in the context of the global for the given pipeline.
    EvaluateJS(PipelineId, String, IpcSender<EvaluateJSReply>),
    /// Retrieve the details of the root node (ie. the document) for the given pipeline.
    GetRootNode(PipelineId, IpcSender<Option<NodeInfo>>),
    /// Retrieve the details of the document element for the given pipeline.
    GetDocumentElement(PipelineId, IpcSender<Option<NodeInfo>>),
    /// Retrieve the details of the child nodes of the given node in the given pipeline.
    GetChildren(PipelineId, String, IpcSender<Option<Vec<NodeInfo>>>),
    /// Retrieve the computed layout properties of the given node in the given pipeline.
    GetLayout(PipelineId, String, IpcSender<Option<ComputedNodeLayout>>),
    /// Update a given node's attributes with a list of modifications.
    ModifyAttribute(PipelineId, String, Vec<Modification>),
    /// Request live console messages for a given pipeline (true if desired, false otherwise).
    WantsLiveNotifications(PipelineId, bool),
    /// Request live notifications for a given set of timeline events for a given pipeline.
    SetTimelineMarkers(
        PipelineId,
        Vec<TimelineMarkerType>,
        IpcSender<Option<TimelineMarker>>,
    ),
    /// Withdraw request for live timeline notifications for a given pipeline.
    DropTimelineMarkers(PipelineId, Vec<TimelineMarkerType>),
    /// Request a callback directed at the given actor name from the next animation frame
    /// executed in the given pipeline.
    RequestAnimationFrame(PipelineId, String),
    /// Direct the given pipeline to reload the current page.
    Reload(PipelineId),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Modification {
    pub attribute_name: String,
    pub new_value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
    Clear,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleMessage {
    pub message: String,
    pub log_level: LogLevel,
    pub filename: String,
    pub line_number: usize,
    pub column_number: usize,
}

bitflags! {
    #[derive(Deserialize, Serialize)]
    pub struct CachedConsoleMessageTypes: u8 {
        const PAGE_ERROR  = 1 << 0;
        const CONSOLE_API = 1 << 1;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageError {
    #[serde(rename = "_type")]
    pub type_: String,
    pub error_message: String,
    pub source_name: String,
    pub line_text: String,
    pub line_number: u32,
    pub column_number: u32,
    pub category: String,
    pub time_stamp: u64,
    pub error: bool,
    pub warning: bool,
    pub exception: bool,
    pub strict: bool,
    pub private: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsoleLog {
    pub level: String,
    pub filename: String,
    pub line_number: u32,
    pub column_number: u32,
    pub time_stamp: u64,
    pub arguments: Vec<String>,
    // pub stacktrace: Vec<...>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CachedConsoleMessage {
    PageError(PageError),
    ConsoleLog(ConsoleLog),
}

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    pub url: ServoUrl,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
    pub pipeline_id: PipelineId,
    pub started_date_time: SystemTime,
    pub time_stamp: i64,
    pub connect_time: u64,
    pub send_time: u64,
    pub is_xhr: bool,
}

#[derive(Debug, PartialEq)]
pub struct HttpResponse {
    pub headers: Option<HeaderMap>,
    pub status: Option<(u16, Vec<u8>)>,
    pub body: Option<Vec<u8>>,
    pub pipeline_id: PipelineId,
}

#[derive(Debug)]
pub enum NetworkEvent {
    HttpRequest(HttpRequest),
    HttpResponse(HttpResponse),
}

impl TimelineMarker {
    pub fn start(name: String) -> StartedTimelineMarker {
        StartedTimelineMarker {
            name,
            start_time: PreciseTime::now(),
            start_stack: None,
        }
    }
}

impl StartedTimelineMarker {
    pub fn end(self) -> TimelineMarker {
        TimelineMarker {
            name: self.name,
            start_time: self.start_time,
            start_stack: self.start_stack,
            end_time: PreciseTime::now(),
            end_stack: None,
        }
    }
}

/// A replacement for `time::PreciseTime` that isn't opaque, so we can serialize it.
///
/// The reason why this doesn't go upstream is that `time` is slated to be part of Rust's standard
/// library, which definitely can't have any dependencies on `serde`. But `serde` can't implement
/// `Deserialize` and `Serialize` itself, because `time::PreciseTime` is opaque! A Catch-22. So I'm
/// duplicating the definition here.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct PreciseTime(u64);

impl PreciseTime {
    pub fn now() -> PreciseTime {
        PreciseTime(time::precise_time_ns())
    }

    pub fn to(&self, later: PreciseTime) -> Duration {
        Duration::from_nanos(later.0 - self.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct WorkerId(pub Uuid);
