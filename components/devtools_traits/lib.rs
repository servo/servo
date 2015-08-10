/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![allow(non_snake_case)]
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

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

/// Messages to the instruct the devtools server to update its known actors/state
/// according to changes in the browser.
pub enum DevtoolsControlMsg {
    FromChrome(ChromeToDevtoolsControlMsg),
    FromScript(ScriptToDevtoolsControlMsg),
}

pub enum ChromeToDevtoolsControlMsg {
    AddClient(TcpStream),
    FramerateTick(String, f64),
    ServerExitMsg,
    NetworkEventMessage(String, NetworkEvent),
}

#[derive(Deserialize, Serialize)]
pub enum ScriptToDevtoolsControlMsg {
    NewGlobal((PipelineId, Option<WorkerId>),
              IpcSender<DevtoolScriptControlMsg>,
              DevtoolsPageInfo),
    ConsoleAPI(PipelineId, ConsoleMessage, Option<WorkerId>),
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

/// Messages to process in a particular script task, as instructed by a devtools client.
#[derive(Deserialize, Serialize)]
pub enum DevtoolScriptControlMsg {
    EvaluateJS(PipelineId, String, IpcSender<EvaluateJSReply>),
    GetRootNode(PipelineId, IpcSender<NodeInfo>),
    GetDocumentElement(PipelineId, IpcSender<NodeInfo>),
    GetChildren(PipelineId, String, IpcSender<Vec<NodeInfo>>),
    GetLayout(PipelineId, String, IpcSender<(f32, f32)>),
    GetCachedMessages(PipelineId, CachedConsoleMessageTypes, IpcSender<Vec<CachedConsoleMessage>>),
    ModifyAttribute(PipelineId, String, Vec<Modification>),
    WantsLiveNotifications(PipelineId, bool),
    SetTimelineMarkers(PipelineId, Vec<TimelineMarkerType>, IpcSender<TimelineMarker>),
    DropTimelineMarkers(PipelineId, Vec<TimelineMarkerType>),
    RequestAnimationFrame(PipelineId, IpcSender<f64>),
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
                attributeName: try!(d.read_struct_field("attributeName", 0, |d| Decodable::decode(d))),
                newValue: match d.read_struct_field("newValue", 1, |d| Decodable::decode(d)) {
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
