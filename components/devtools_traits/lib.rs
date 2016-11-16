/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![allow(non_snake_case)]
#![deny(unsafe_code)]

#![feature(proc_macro)]

#[allow(unused_extern_crates)]
#[macro_use]
extern crate bitflags;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate hyper;
extern crate ipc_channel;
extern crate msg;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate servo_url;
extern crate time;

use hyper::header::Headers;
use hyper::method::Method;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use servo_url::ServoUrl;
use std::net::TcpStream;
use time::Duration;
use time::Tm;

// Information would be attached to NewGlobal to be received and show in devtools.
// Extend these fields if we need more information.
#[derive(Debug, Deserialize, Serialize)]
pub struct DevtoolsPageInfo {
    pub title: String,
    pub url: ServoUrl,
}

#[derive(Debug, Deserialize, HeapSizeOf, Serialize, Clone)]
pub struct CSSError {
    pub filename: String,
    pub line: usize,
    pub column: usize,
    pub msg: String
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

#[derive(Debug, Deserialize, Serialize)]
/// Events that the devtools server must act upon.
pub enum ScriptToDevtoolsControlMsg {
    /// A new global object was created, associated with a particular pipeline.
    /// The means of communicating directly with it are provided.
    NewGlobal((PipelineId, Option<WorkerId>),
              IpcSender<DevtoolScriptControlMsg>,
              DevtoolsPageInfo),
    /// A particular page has invoked the console API.
    ConsoleAPI(PipelineId, ConsoleMessage, Option<WorkerId>),
    /// An animation frame with the given timestamp was processed in a script thread.
    /// The actor with the provided name should be notified.
    FramerateTick(String, f64),

    /// Report a CSS parse error for the given pipeline
    ReportCSSError(PipelineId, CSSError),
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
pub struct NodeInfo {
    pub uniqueId: String,
    pub baseURI: String,
    pub parent: String,
    pub nodeType: u16,
    pub namespaceURI: String,
    pub nodeName: String,
    pub numChildren: usize,

    pub name: String,
    pub publicId: String,
    pub systemId: String,

    pub attrs: Vec<AttrInfo>,

    pub isDocumentElement: bool,

    pub shortValue: String,
    pub incompleteValue: bool,
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum TimelineMarkerType {
    Reflow,
    DOMEvent,
}

/// The properties of a DOM node as computed by layout.
#[derive(Debug, Deserialize, Serialize)]
pub struct ComputedNodeLayout {
    pub display: String,
    pub position: String,
    pub zIndex: String,
    pub boxSizing: String,

    pub autoMargins: AutoMargins,
    pub marginTop: String,
    pub marginRight: String,
    pub marginBottom: String,
    pub marginLeft: String,

    pub borderTopWidth: String,
    pub borderRightWidth: String,
    pub borderBottomWidth: String,
    pub borderLeftWidth: String,

    pub paddingTop: String,
    pub paddingRight: String,
    pub paddingBottom: String,
    pub paddingLeft: String,

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
    /// Retrieve all stored console messages for the given pipeline.
    GetCachedMessages(PipelineId, CachedConsoleMessageTypes, IpcSender<Vec<CachedConsoleMessage>>),
    /// Update a given node's attributes with a list of modifications.
    ModifyAttribute(PipelineId, String, Vec<Modification>),
    /// Request live console messages for a given pipeline (true if desired, false otherwise).
    WantsLiveNotifications(PipelineId, bool),
    /// Request live notifications for a given set of timeline events for a given pipeline.
    SetTimelineMarkers(PipelineId, Vec<TimelineMarkerType>, IpcSender<Option<TimelineMarker>>),
    /// Withdraw request for live timeline notifications for a given pipeline.
    DropTimelineMarkers(PipelineId, Vec<TimelineMarkerType>),
    /// Request a callback directed at the given actor name from the next animation frame
    /// executed in the given pipeline.
    RequestAnimationFrame(PipelineId, String),
    /// Direct the given pipeline to reload the current page.
    Reload(PipelineId),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Modification {
    pub attributeName: String,
    pub newValue: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConsoleMessage {
    pub message: String,
    pub logLevel: LogLevel,
    pub filename: String,
    pub lineNumber: usize,
    pub columnNumber: usize,
}

bitflags! {
    #[derive(Deserialize, Serialize)]
    pub flags CachedConsoleMessageTypes: u8 {
        const PAGE_ERROR  = 1 << 0,
        const CONSOLE_API = 1 << 1,
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageError {
    #[serde(rename = "_type")]
    pub type_: String,
    pub errorMessage: String,
    pub sourceName: String,
    pub lineText: String,
    pub lineNumber: u32,
    pub columnNumber: u32,
    pub category: String,
    pub timeStamp: u64,
    pub error: bool,
    pub warning: bool,
    pub exception: bool,
    pub strict: bool,
    pub private: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConsoleAPI {
    #[serde(rename = "_type")]
    pub type_: String,
    pub level: String,
    pub filename: String,
    pub lineNumber: u32,
    pub functionName: String,
    pub timeStamp: u64,
    pub private: bool,
    pub arguments: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CachedConsoleMessage {
    PageError(PageError),
    ConsoleAPI(ConsoleAPI),
}

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    pub url: ServoUrl,
    pub method: Method,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
    pub pipeline_id: PipelineId,
    pub startedDateTime: Tm,
    pub timeStamp: i64,
    pub connect_time: u64,
    pub send_time: u64,
    pub is_xhr: bool,
}

#[derive(Debug, PartialEq)]
pub struct HttpResponse {
    pub headers: Option<Headers>,
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
            name: name,
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
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct PreciseTime(u64);

impl PreciseTime {
    pub fn now() -> PreciseTime {
        PreciseTime(time::precise_time_ns())
    }

    pub fn to(&self, later: PreciseTime) -> Duration {
        Duration::nanoseconds((later.0 - self.0) as i64)
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct WorkerId(pub u32);
