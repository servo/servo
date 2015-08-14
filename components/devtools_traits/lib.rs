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

#![feature(custom_derive, plugin)]
#![plugin(serde_macros, plugins)]

#[macro_use]
extern crate bitflags;

extern crate ipc_channel;
extern crate msg;
extern crate rustc_serialize;
extern crate serde;
extern crate url;
extern crate hyper;
extern crate util;
extern crate time;

use rustc_serialize::{Decodable, Decoder};
use msg::constellation_msg::{PipelineId, WorkerId};
use util::str::DOMString;
use url::Url;

use hyper::header::Headers;
use hyper::http::RawStatus;
use hyper::method::Method;
use ipc_channel::ipc::IpcSender;
use time::Duration;

use std::net::TcpStream;

// Information would be attached to NewGlobal to be received and show in devtools.
// Extend these fields if we need more information.
#[derive(Deserialize, Serialize)]
pub struct DevtoolsPageInfo {
    pub title: DOMString,
    pub url: Url
}

/// Messages to instruct the devtools server to update its known actors/state
/// according to changes in the browser.
pub enum DevtoolsControlMsg {
    /// Messages from tasks in the chrome process (resource/constellation/devtools)
    FromChrome(ChromeToDevtoolsControlMsg),
    /// Messages from script tasks
    FromScript(ScriptToDevtoolsControlMsg),
}

/// Events that the devtools server must act upon.
pub enum ChromeToDevtoolsControlMsg {
    /// A new client has connected to the server.
    AddClient(TcpStream),
    /// The browser is shutting down.
    ServerExitMsg,
    /// A network event occurred (request, reply, etc.). The actor with the
    /// provided name should be notified.
    NetworkEvent(String, NetworkEvent),
}

#[derive(Deserialize, Serialize)]
/// Events that the devtools server must act upon.
pub enum ScriptToDevtoolsControlMsg {
    /// A new global object was created, associated with a particular pipeline.
    /// The means of communicating directly with it are provided.
    NewGlobal((PipelineId, Option<WorkerId>),
              IpcSender<DevtoolScriptControlMsg>,
              DevtoolsPageInfo),
    /// A particular page has invoked the console API.
    ConsoleAPI(PipelineId, ConsoleMessage, Option<WorkerId>),
    /// An animation frame with the given timestamp was processed in a script task.
    /// The actor with the provided name should be notified.
    FramerateTick(String, f64),
}

/// Serialized JS return values
/// TODO: generalize this beyond the EvaluateJS message?
#[derive(Deserialize, Serialize)]
pub enum EvaluateJSReply {
    VoidValue,
    NullValue,
    BooleanValue(bool),
    NumberValue(f64),
    StringValue(String),
    ActorValue { class: String, uuid: String },
}

#[derive(Deserialize, Serialize)]
pub struct AttrInfo {
    pub namespace: String,
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Serialize)]
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

#[derive(PartialEq, Eq, Deserialize, Serialize)]
pub enum TracingMetadata {
    Default,
    IntervalStart,
    IntervalEnd,
    Event,
    EventBacktrace,
}

#[derive(Deserialize, Serialize)]
pub struct TimelineMarker {
    pub name: String,
    pub metadata: TracingMetadata,
    pub time: PreciseTime,
    pub stack: Option<Vec<()>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub enum TimelineMarkerType {
    Reflow,
    DOMEvent,
}

/// The properties of a DOM node as computed by layout.
#[derive(Deserialize, Serialize)]
pub struct ComputedNodeLayout {
    pub width: f32,
    pub height: f32,
}

/// Messages to process in a particular script task, as instructed by a devtools client.
#[derive(Deserialize, Serialize)]
pub enum DevtoolScriptControlMsg {
    /// Evaluate a JS snippet in the context of the global for the given pipeline.
    EvaluateJS(PipelineId, String, IpcSender<EvaluateJSReply>),
    /// Retrieve the details of the root node (ie. the document) for the given pipeline.
    GetRootNode(PipelineId, IpcSender<NodeInfo>),
    /// Retrieve the details of the document element for the given pipeline.
    GetDocumentElement(PipelineId, IpcSender<NodeInfo>),
    /// Retrieve the details of the child nodes of the given node in the given pipeline.
    GetChildren(PipelineId, String, IpcSender<Vec<NodeInfo>>),
    /// Retrieve the computed layout properties of the given node in the given pipeline.
    GetLayout(PipelineId, String, IpcSender<ComputedNodeLayout>),
    /// Retrieve all stored console messages for the given pipeline.
    GetCachedMessages(PipelineId, CachedConsoleMessageTypes, IpcSender<Vec<CachedConsoleMessage>>),
    /// Update a given node's attributes with a list of modifications.
    ModifyAttribute(PipelineId, String, Vec<Modification>),
    /// Request live console messages for a given pipeline (true if desired, false otherwise).
    WantsLiveNotifications(PipelineId, bool),
    /// Request live notifications for a given set of timeline events for a given pipeline.
    SetTimelineMarkers(PipelineId, Vec<TimelineMarkerType>, IpcSender<TimelineMarker>),
    /// Withdraw request for live timeline notifications for a given pipeline.
    DropTimelineMarkers(PipelineId, Vec<TimelineMarkerType>),
    /// Request a callback directed at the given actor name from the next animation frame
    /// executed in the given pipeline.
    RequestAnimationFrame(PipelineId, String),
}

#[derive(RustcEncodable, Deserialize, Serialize)]
pub struct Modification {
    pub attributeName: String,
    pub newValue: Option<String>,
}

impl Decodable for Modification {
    fn decode<D: Decoder>(d: &mut D) -> Result<Modification, D::Error> {
        d.read_struct("Modification", 2, |d|
            Ok(Modification {
                attributeName: try!(d.read_struct_field("attributeName", 0, Decodable::decode)),
                newValue: match d.read_struct_field("newValue", 1, Decodable::decode) {
                    Ok(opt) => opt,
                    Err(_) => None
                }
            })
        )
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ConsoleMessage {
    pub message: String,
    pub logLevel: LogLevel,
    pub filename: String,
    pub lineNumber: u32,
    pub columnNumber: u32,
}

bitflags! {
    #[derive(Deserialize, Serialize)]
    flags CachedConsoleMessageTypes: u8 {
        const PAGE_ERROR  = 1 << 0,
        const CONSOLE_API = 1 << 1,
    }
}

#[derive(RustcEncodable, Deserialize, Serialize)]
pub struct PageError {
    pub _type: String,
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

#[derive(RustcEncodable, Deserialize, Serialize)]
pub struct ConsoleAPI {
    pub _type: String,
    pub level: String,
    pub filename: String,
    pub lineNumber: u32,
    pub functionName: String,
    pub timeStamp: u64,
    pub private: bool,
    pub arguments: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub enum CachedConsoleMessage {
    PageError(PageError),
    ConsoleAPI(ConsoleAPI),
}

#[derive(Clone)]
pub enum NetworkEvent {
    HttpRequest(Url, Method, Headers, Option<Vec<u8>>),
    HttpResponse(Option<Headers>, Option<RawStatus>, Option<Vec<u8>>)
}

impl TimelineMarker {
    pub fn new(name: String, metadata: TracingMetadata) -> TimelineMarker {
        TimelineMarker {
            name: name,
            metadata: metadata,
            time: PreciseTime::now(),
            stack: None,
        }
    }
}

/// A replacement for `time::PreciseTime` that isn't opaque, so we can serialize it.
///
/// The reason why this doesn't go upstream is that `time` is slated to be part of Rust's standard
/// library, which definitely can't have any dependencies on `serde`. But `serde` can't implement
/// `Deserialize` and `Serialize` itself, because `time::PreciseTime` is opaque! A Catch-22. So I'm
/// duplicating the definition here.
#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct PreciseTime(u64);

impl PreciseTime {
    pub fn now() -> PreciseTime {
        PreciseTime(time::precise_time_ns())
    }

    pub fn to(&self, later: PreciseTime) -> Duration {
        Duration::nanoseconds((later.0 - self.0) as i64)
    }
}
