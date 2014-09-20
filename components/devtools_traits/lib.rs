/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

extern crate "msg" as servo_msg;

/// This module contains shared types and messages for use by devtools/script.
/// The traits are here instead of in script so that the devtools crate can be
/// modified independently of the rest of Servo.

use servo_msg::constellation_msg::PipelineId;

pub type DevtoolsControlChan = Sender<DevtoolsControlMsg>;
pub type DevtoolsControlPort = Receiver<DevtoolScriptControlMsg>;

/// Messages to the instruct the devtools server to update its known actors/state
/// according to changes in the browser.
pub enum DevtoolsControlMsg {
    NewGlobal(PipelineId, Sender<DevtoolScriptControlMsg>),
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
}

/// Messages to instruct devtools server to update its state relating to a particular
/// tab.
pub enum ScriptDevtoolControlMsg {
    /// Report a new JS error message
    ReportConsoleMsg(String),
}
