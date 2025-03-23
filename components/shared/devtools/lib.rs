/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use core::fmt;
use std::collections::HashMap;
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base::cross_process_instant::CrossProcessInstant;
use base::id::{BrowsingContextId, PipelineId, WebViewId};
use bitflags::bitflags;
use http::{HeaderMap, Method};
use ipc_channel::ipc::IpcSender;
use malloc_size_of_derive::MallocSizeOf;
use net_traits::http_status::HttpStatus;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use uuid::Uuid;

// Information would be attached to NewGlobal to be received and show in devtools.
// Extend these fields if we need more information.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DevtoolsPageInfo {
    pub title: String,
    pub url: ServoUrl,
    pub is_top_level_global: bool,
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
// FIXME: https://github.com/servo/servo/issues/34591
#[expect(clippy::large_enum_variant)]
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
        (BrowsingContextId, PipelineId, Option<WorkerId>, WebViewId),
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
    pub host: Option<String>,
    #[serde(rename = "baseURI")]
    pub base_uri: String,
    pub parent: String,
    pub node_type: u16,
    pub node_name: String,
    pub node_value: Option<String>,
    pub num_children: usize,
    pub attrs: Vec<AttrInfo>,
    pub is_top_level_document: bool,
    pub shadow_root_mode: Option<ShadowRootMode>,
    pub is_shadow_host: bool,
    pub display: Option<String>,
}

pub struct StartedTimelineMarker {
    name: String,
    start_time: CrossProcessInstant,
    start_stack: Option<Vec<()>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimelineMarker {
    pub name: String,
    pub start_time: CrossProcessInstant,
    pub start_stack: Option<Vec<()>>,
    pub end_time: CrossProcessInstant,
    pub end_stack: Option<Vec<()>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum TimelineMarkerType {
    Reflow,
    DOMEvent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStyle {
    pub name: String,
    pub value: String,
    pub priority: String,
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
    /// Retrieve the CSS style properties defined in the attribute tag for the given node.
    GetAttributeStyle(PipelineId, String, IpcSender<Option<Vec<NodeStyle>>>),
    /// Retrieve the CSS style properties defined in an stylesheet for the given selector.
    GetStylesheetStyle(
        PipelineId,
        String,
        String,
        usize,
        IpcSender<Option<Vec<NodeStyle>>>,
    ),
    /// Retrieves the CSS selectors for the given node. A selector is comprised of the text
    /// of the selector and the id of the stylesheet that contains it.
    GetSelectors(PipelineId, String, IpcSender<Option<Vec<(String, usize)>>>),
    /// Retrieve the computed CSS style properties for the given node.
    GetComputedStyle(PipelineId, String, IpcSender<Option<Vec<NodeStyle>>>),
    /// Retrieve the computed layout properties of the given node in the given pipeline.
    GetLayout(PipelineId, String, IpcSender<Option<ComputedNodeLayout>>),
    /// Update a given node's attributes with a list of modifications.
    ModifyAttribute(PipelineId, String, Vec<AttrModification>),
    /// Update a given node's style rules with a list of modifications.
    ModifyRule(PipelineId, String, Vec<RuleModification>),
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
    /// Gets the list of all allowed CSS rules and possible values.
    GetCssDatabase(IpcSender<HashMap<String, CssDatabaseProperty>>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttrModification {
    pub attribute_name: String,
    pub new_value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleModification {
    #[serde(rename = "type")]
    pub type_: String,
    pub index: u32,
    pub name: String,
    pub value: String,
    pub priority: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
    Clear,
    Trace,
}

/// A console message as it is sent from script to the constellation
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleMessage {
    pub log_level: LogLevel,
    pub filename: String,
    pub line_number: usize,
    pub column_number: usize,
    pub arguments: Vec<ConsoleMessageArgument>,
    pub stacktrace: Option<Vec<StackFrame>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConsoleMessageArgument {
    String(String),
    Integer(i32),
    Number(f64),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StackFrame {
    pub filename: String,

    #[serde(rename = "functionName")]
    pub function_name: String,

    #[serde(rename = "columnNumber")]
    pub column_number: u32,

    #[serde(rename = "lineNumber")]
    pub line_number: u32,
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

/// Represents a console message as it is sent to the devtools
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsoleLog {
    pub level: String,
    pub filename: String,
    pub line_number: u32,
    pub column_number: u32,
    pub time_stamp: u64,
    pub arguments: Vec<ConsoleArgument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stacktrace: Option<Vec<StackFrame>>,
}

impl From<ConsoleMessage> for ConsoleLog {
    fn from(value: ConsoleMessage) -> Self {
        let level = match value.log_level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Clear => "clear",
            LogLevel::Trace => "trace",
            LogLevel::Log => "log",
        }
        .to_owned();

        let time_stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            level,
            filename: value.filename,
            line_number: value.line_number as u32,
            column_number: value.column_number as u32,
            time_stamp,
            arguments: value.arguments.into_iter().map(|arg| arg.into()).collect(),
            stacktrace: value.stacktrace,
        }
    }
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
    pub connect_time: Duration,
    pub send_time: Duration,
    pub is_xhr: bool,
}

#[derive(Debug, PartialEq)]
pub struct HttpResponse {
    pub headers: Option<HeaderMap>,
    pub status: HttpStatus,
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
            start_time: CrossProcessInstant::now(),
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
            end_time: CrossProcessInstant::now(),
            end_stack: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct WorkerId(pub Uuid);

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssDatabaseProperty {
    pub is_inherited: bool,
    pub values: Vec<String>,
    pub supports: Vec<String>,
    pub subproperties: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConsoleArgument {
    String(String),
    Integer(i32),
    Number(f64),
}

impl From<ConsoleMessageArgument> for ConsoleArgument {
    fn from(value: ConsoleMessageArgument) -> Self {
        match value {
            ConsoleMessageArgument::String(string) => Self::String(string),
            ConsoleMessageArgument::Integer(integer) => Self::Integer(integer),
            ConsoleMessageArgument::Number(number) => Self::Number(number),
        }
    }
}

impl From<String> for ConsoleMessageArgument {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

pub struct ConsoleMessageBuilder {
    level: LogLevel,
    filename: String,
    line_number: u32,
    column_number: u32,
    arguments: Vec<ConsoleMessageArgument>,
    stack_trace: Option<Vec<StackFrame>>,
}

impl ConsoleMessageBuilder {
    pub fn new(level: LogLevel, filename: String, line_number: u32, column_number: u32) -> Self {
        Self {
            level,
            filename,
            line_number,
            column_number,
            arguments: vec![],
            stack_trace: None,
        }
    }

    pub fn attach_stack_trace(&mut self, stack_trace: Vec<StackFrame>) -> &mut Self {
        self.stack_trace = Some(stack_trace);
        self
    }

    pub fn add_argument(&mut self, argument: ConsoleMessageArgument) -> &mut Self {
        self.arguments.push(argument);
        self
    }

    pub fn finish(self) -> ConsoleMessage {
        ConsoleMessage {
            log_level: self.level,
            filename: self.filename,
            line_number: self.line_number as usize,
            column_number: self.column_number as usize,
            arguments: self.arguments,
            stacktrace: self.stack_trace,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ShadowRootMode {
    Open,
    Closed,
}

impl fmt::Display for ShadowRootMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Closed => write!(f, "close"),
        }
    }
}
