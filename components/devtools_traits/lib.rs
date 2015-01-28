/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains shared types and messages for use by devtools/script.
//! The traits are here instead of in script so that the devtools crate can be
//! modified independently of the rest of Servo.

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![feature(int_uint)]

#![allow(non_snake_case)]
#![allow(missing_copy_implementations)]
#![allow(unstable)]

extern crate "msg" as servo_msg;
extern crate serialize;
extern crate url;
extern crate "util" as servo_util;

pub use self::DevtoolsControlMsg::*;
pub use self::DevtoolScriptControlMsg::*;
pub use self::EvaluateJSReply::*;

use serialize::{Decodable, Decoder};
use servo_msg::constellation_msg::PipelineId;
use servo_util::str::DOMString;
use url::Url;

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
    NewGlobal(PipelineId, Sender<DevtoolScriptControlMsg>, DevtoolsPageInfo),
    ServerExitMsg
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
    pub nodeType: uint,
    pub namespaceURI: String,
    pub nodeName: String,
    pub numChildren: uint,

    pub name: String,
    pub publicId: String,
    pub systemId: String,

    pub attrs: Vec<AttrInfo>,

    pub isDocumentElement: bool,

    pub shortValue: String,
    pub incompleteValue: bool,
}

/// Messages to process in a particular script task, as instructed by a devtools client.
pub enum DevtoolScriptControlMsg {
    EvaluateJS(PipelineId, String, Sender<EvaluateJSReply>),
    GetRootNode(PipelineId, Sender<NodeInfo>),
    GetDocumentElement(PipelineId, Sender<NodeInfo>),
    GetChildren(PipelineId, String, Sender<Vec<NodeInfo>>),
    GetLayout(PipelineId, String, Sender<(f32, f32)>),
    ModifyAttribute(PipelineId, String, Vec<Modification>),
}

/// Messages to instruct devtools server to update its state relating to a particular
/// tab.
pub enum ScriptDevtoolControlMsg {
    /// Report a new JS error message
    ReportConsoleMsg(String),
}

#[derive(Encodable)]
pub struct Modification{
    pub attributeName: String,
    pub newValue: Option<String>,
}

impl Decodable for Modification {
    fn decode<D: Decoder>(d: &mut D) -> Result<Modification, D::Error> {
        d.read_struct("Modification", 2u, |d|
            Ok(Modification {
                attributeName: try!(d.read_struct_field("attributeName", 0u, |d| Decodable::decode(d))),
                newValue: match d.read_struct_field("newValue", 1u, |d| Decodable::decode(d)) {
                    Ok(opt) => opt,
                    Err(_) => None
                }
            })
        )
    }
}
