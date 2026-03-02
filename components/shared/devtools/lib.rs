/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.
//!
//! Since these types can be sent through the IPC channel and use non
//! self-describing serializers, the `flatten`, `skip*`, `tag` and `untagged`
//! serde annotations are not supported. Types like `serde_json::Value` aren't
//! supported either. For JSON serialization it is preferred to use a wrapper
//! struct in the devtools crate instead.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use core::fmt;
use std::collections::HashMap;
use std::fmt::Display;
use std::net::TcpStream;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel::GenericSender;
use base::id::{BrowsingContextId, PipelineId, WebViewId};
pub use embedder_traits::ConsoleLogLevel;
use embedder_traits::Theme;
use http::{HeaderMap, Method};
use malloc_size_of_derive::MallocSizeOf;
use net_traits::http_status::HttpStatus;
use net_traits::request::Destination;
use net_traits::{DebugVec, TlsSecurityInfo};
use profile_traits::mem::ReportsChan;
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
    /// Sent when a devtools client thread terminates.
    ClientExited,
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
    /// Perform a memory report.
    CollectMemoryReport(ReportsChan),
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
        GenericSender<DevtoolScriptControlMsg>,
        DevtoolsPageInfo,
    ),
    /// The given browsing context is performing a navigation.
    Navigate(BrowsingContextId, NavigationState),
    /// A particular page has invoked the console API.
    ConsoleAPI(PipelineId, ConsoleMessage, Option<WorkerId>),
    /// Request to clear the console for a given pipeline.
    ClearConsole(PipelineId, Option<WorkerId>),
    /// An animation frame with the given timestamp was processed in a script thread.
    /// The actor with the provided name should be notified.
    FramerateTick(String, f64),

    /// Report a CSS parse error for the given pipeline
    ReportCSSError(PipelineId, CSSError),

    /// Report a page error for the given pipeline
    ReportPageError(PipelineId, PageError),

    /// Report a page title change
    TitleChanged(PipelineId, String),

    /// Get source information from script
    CreateSourceActor(
        GenericSender<DevtoolScriptControlMsg>,
        PipelineId,
        SourceInfo,
    ),

    UpdateSourceContent(PipelineId, String),

    DomMutation(PipelineId, DomMutation),

    /// The debugger is paused, sending frame information.
    DebuggerPause(PipelineId, FrameOffset, PauseReason),

    /// Get frame information from script
    CreateFrameActor(GenericSender<String>, PipelineId, FrameInfo),
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum DomMutation {
    AttributeModified {
        node: String,
        attribute_name: String,
        new_value: Option<String>,
    },
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
    /// Whether this node is currently displayed.
    ///
    /// For example, the node might have `display: none`.
    pub is_displayed: bool,

    /// The `DOCTYPE` name if this is a `DocumentType` node, `None` otherwise
    pub doctype_name: Option<String>,

    /// The `DOCTYPE` public identifier if this is a `DocumentType` node , `None` otherwise
    pub doctype_public_identifier: Option<String>,

    /// The `DOCTYPE` system identifier if this is a `DocumentType` node, `None` otherwise
    pub doctype_system_identifier: Option<String>,

    pub has_event_listeners: bool,
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
#[serde(rename_all = "kebab-case")]
pub struct ComputedNodeLayout {
    pub display: String,
    pub position: String,
    pub z_index: String,
    pub box_sizing: String,

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

#[derive(Debug, Default, Deserialize, Serialize)]
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
    EvaluateJS(PipelineId, String, GenericSender<EvaluateJSReply>),
    /// Retrieve the details of the root node (ie. the document) for the given pipeline.
    GetRootNode(PipelineId, GenericSender<Option<NodeInfo>>),
    /// Retrieve the details of the document element for the given pipeline.
    GetDocumentElement(PipelineId, GenericSender<Option<NodeInfo>>),
    /// Retrieve the details of the child nodes of the given node in the given pipeline.
    GetChildren(PipelineId, String, GenericSender<Option<Vec<NodeInfo>>>),
    /// Retrieve the CSS style properties defined in the attribute tag for the given node.
    GetAttributeStyle(PipelineId, String, GenericSender<Option<Vec<NodeStyle>>>),
    /// Retrieve the CSS style properties defined in an stylesheet for the given selector.
    GetStylesheetStyle(
        PipelineId,
        String,
        String,
        usize,
        GenericSender<Option<Vec<NodeStyle>>>,
    ),
    /// Retrieves the CSS selectors for the given node. A selector is comprised of the text
    /// of the selector and the id of the stylesheet that contains it.
    GetSelectors(
        PipelineId,
        String,
        GenericSender<Option<Vec<(String, usize)>>>,
    ),
    /// Retrieve the computed CSS style properties for the given node.
    GetComputedStyle(PipelineId, String, GenericSender<Option<Vec<NodeStyle>>>),
    /// Get information about event listeners on a node.
    GetEventListenerInfo(PipelineId, String, GenericSender<Vec<EventListenerInfo>>),
    /// Retrieve the computed layout properties of the given node in the given pipeline.
    GetLayout(
        PipelineId,
        String,
        GenericSender<Option<(ComputedNodeLayout, AutoMargins)>>,
    ),
    /// Get a unique XPath selector for the node.
    GetXPath(PipelineId, String, GenericSender<String>),
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
        GenericSender<Option<TimelineMarker>>,
    ),
    /// Withdraw request for live timeline notifications for a given pipeline.
    DropTimelineMarkers(PipelineId, Vec<TimelineMarkerType>),
    /// Request a callback directed at the given actor name from the next animation frame
    /// executed in the given pipeline.
    RequestAnimationFrame(PipelineId, String),
    /// Direct the given pipeline to reload the current page.
    Reload(PipelineId),
    /// Gets the list of all allowed CSS rules and possible values.
    GetCssDatabase(GenericSender<HashMap<String, CssDatabaseProperty>>),
    /// Simulates a light or dark color scheme for the given pipeline
    SimulateColorScheme(PipelineId, Theme),
    /// Highlight the given DOM node
    HighlightDomNode(PipelineId, Option<String>),

    Eval(String, PipelineId, GenericSender<EvaluateJSReply>),
    GetPossibleBreakpoints(u32, GenericSender<Vec<RecommendedBreakpointLocation>>),
    SetBreakpoint(u32, u32, u32),
    ClearBreakpoint(u32, u32, u32),
    Interrupt,
    Resume(Option<String>, Option<String>),
}

#[derive(Clone, Debug, Deserialize, Serialize, MallocSizeOf)]
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

#[derive(Clone, Debug, Deserialize, Serialize, MallocSizeOf)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub filename: String,
    pub function_name: String,
    pub column_number: u32,
    pub line_number: u32,
    // Not implemented in Servo
    // source_id
}

pub fn get_time_stamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Debug, Deserialize, Serialize, MallocSizeOf)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleMessageFields {
    pub level: ConsoleLogLevel,
    pub filename: String,
    pub line_number: u32,
    pub column_number: u32,
    pub time_stamp: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConsoleArgument {
    String(String),
    Integer(i32),
    Number(f64),
    Boolean(bool),
    Object(ConsoleArgumentObject),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsoleArgumentObject {
    pub class: String,
    pub own_properties: Vec<ConsoleArgumentPropertyValue>,
}

/// A property on a JS object passed as a console argument.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsoleArgumentPropertyValue {
    pub key: String,
    pub configurable: bool,
    pub enumerable: bool,
    pub writable: bool,
    pub value: ConsoleArgument,
}

impl From<String> for ConsoleArgument {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsoleMessage {
    pub fields: ConsoleMessageFields,
    pub arguments: Vec<ConsoleArgument>,
    pub stacktrace: Option<Vec<StackFrame>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, MallocSizeOf)]
#[serde(rename_all = "camelCase")]
pub struct PageError {
    pub error_message: String,
    pub source_name: String,
    pub line_number: u32,
    pub column_number: u32,
    pub time_stamp: u64,
}

#[derive(Debug, PartialEq, MallocSizeOf)]
pub struct HttpRequest {
    pub url: ServoUrl,
    #[ignore_malloc_size_of = "http type"]
    pub method: Method,
    #[ignore_malloc_size_of = "http type"]
    pub headers: HeaderMap,
    pub body: Option<DebugVec>,
    pub pipeline_id: PipelineId,
    pub started_date_time: SystemTime,
    pub time_stamp: i64,
    pub connect_time: Duration,
    pub send_time: Duration,
    pub destination: Destination,
    pub is_xhr: bool,
    pub browsing_context_id: BrowsingContextId,
}

#[derive(Debug, PartialEq, MallocSizeOf)]
pub struct HttpResponse {
    #[ignore_malloc_size_of = "Http type"]
    pub headers: Option<HeaderMap>,
    pub status: HttpStatus,
    pub body: Option<DebugVec>,
    pub from_cache: bool,
    pub pipeline_id: PipelineId,
    pub browsing_context_id: BrowsingContextId,
}

#[derive(Debug, PartialEq)]
pub struct SecurityInfoUpdate {
    pub browsing_context_id: BrowsingContextId,
    pub security_info: Option<TlsSecurityInfo>,
}

#[derive(Debug)]
pub enum NetworkEvent {
    HttpRequest(HttpRequest),
    HttpRequestUpdate(HttpRequest),
    HttpResponse(HttpResponse),
    SecurityInfo(SecurityInfoUpdate),
}

impl NetworkEvent {
    pub fn forward_to_devtools(&self) -> bool {
        match self {
            NetworkEvent::HttpRequest(http_request) => http_request.url.scheme() != "data",
            NetworkEvent::HttpRequestUpdate(_) => true,
            NetworkEvent::HttpResponse(_) => true,
            NetworkEvent::SecurityInfo(_) => true,
        }
    }
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
impl Display for WorkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl FromStr for WorkerId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

#[derive(Debug, Deserialize, Serialize, MallocSizeOf)]
#[serde(rename_all = "camelCase")]
pub struct CssDatabaseProperty {
    pub is_inherited: bool,
    pub values: Vec<String>,
    pub supports: Vec<String>,
    pub subproperties: Vec<String>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceInfo {
    pub url: ServoUrl,
    pub introduction_type: String,
    pub inline: bool,
    pub worker_id: Option<WorkerId>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub spidermonkey_id: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedBreakpointLocation {
    pub script_id: u32,
    pub offset: u32,
    pub line_number: u32,
    pub column_number: u32,
    pub is_step_start: bool,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameInfo {
    pub display_name: String,
    pub on_stack: bool,
    pub oldest: bool,
    pub terminated: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventListenerInfo {
    pub event_type: String,
    pub capturing: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseReason {
    #[serde(rename = "type")]
    pub type_: String,
    pub on_next: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FrameOffset {
    pub actor: String,
    pub column: u32,
    pub line: u32,
}
