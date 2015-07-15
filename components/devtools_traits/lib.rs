/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![allow(non_snake_case)]

#[macro_use]
extern crate bitflags;

extern crate msg;
extern crate rustc_serialize;
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

use std::net::TcpStream;
use std::sync::mpsc::{Sender, Receiver};

pub type DevtoolsControlChan = Sender<DevtoolsControlMsg>;
pub type DevtoolsControlPort = Receiver<DevtoolScriptControlMsg>;

// Information would be attached to NewGlobal to be received and show in devtools.
// Extend these fields if we need more information.
pub struct DevtoolsPageInfo {
    pub title: DOMString,
    pub url: Url
}

/// Messages to the instruct the devtools server to update its known actors/state
/// according to changes in the browser.
pub enum DevtoolsControlMsg {
    AddClient(TcpStream),
    FramerateTick(String, f64),
    NewGlobal((PipelineId, Option<WorkerId>), Sender<DevtoolScriptControlMsg>, DevtoolsPageInfo),
    SendConsoleMessage(PipelineId, ConsoleMessage),
    ServerExitMsg,
    NetworkEventMessage(String, NetworkEvent),
}

/// Serialized JS return values
/// TODO: generalize this beyond the EvaluateJS message?
pub enum EvaluateJSReply {
    VoidValue,
    NullValue,
    BooleanValue(bool),
    NumberValue(f64),
    StringValue(String),
    ActorValue(String),
}

pub struct AttrInfo {
    pub namespace: String,
    pub name: String,
    pub value: String,
}

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

#[derive(PartialEq, Eq)]
pub enum TracingMetadata {
    Default,
    IntervalStart,
    IntervalEnd,
    Event,
    EventBacktrace,
}

pub struct TimelineMarker {
    pub name: String,
    pub metadata: TracingMetadata,
    pub time: time::PreciseTime,
    pub stack: Option<Vec<()>>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum TimelineMarkerType {
    Reflow,
    DOMEvent,
}

/// Messages to process in a particular script task, as instructed by a devtools client.
pub enum DevtoolScriptControlMsg {
    EvaluateJS(PipelineId, String, Sender<EvaluateJSReply>),
    GetRootNode(PipelineId, Sender<NodeInfo>),
    GetDocumentElement(PipelineId, Sender<NodeInfo>),
    GetChildren(PipelineId, String, Sender<Vec<NodeInfo>>),
    GetLayout(PipelineId, String, Sender<(f32, f32)>),
    GetCachedMessages(PipelineId, CachedConsoleMessageTypes, Sender<Vec<CachedConsoleMessage>>),
    ModifyAttribute(PipelineId, String, Vec<Modification>),
    WantsLiveNotifications(PipelineId, bool),
    SetTimelineMarkers(PipelineId, Vec<TimelineMarkerType>, Sender<TimelineMarker>),
    DropTimelineMarkers(PipelineId, Vec<TimelineMarkerType>),
    RequestAnimationFrame(PipelineId, Box<Fn(f64, ) + Send>),
}

#[derive(RustcEncodable)]
pub struct Modification{
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

#[derive(Clone)]
pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone)]
pub struct ConsoleMessage {
    pub message: String,
    pub logLevel: LogLevel,
    pub filename: String,
    pub lineNumber: u32,
    pub columnNumber: u32,
}

bitflags! {
    flags CachedConsoleMessageTypes: u8 {
        const PAGE_ERROR  = 1 << 0,
        const CONSOLE_API = 1 << 1,
    }
}

#[derive(RustcEncodable)]
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

#[derive(RustcEncodable)]
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
            time: time::PreciseTime::now(),
            stack: None,
        }
    }
}
