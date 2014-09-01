/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

extern crate servo_msg = "msg";

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

/// Messages to process in a particular script task, as instructed by a devtools client.
pub enum DevtoolScriptControlMsg {
    EvaluateJS(PipelineId, String, Sender<EvaluateJSReply>),
}

/// Messages to instruct devtools server to update its state relating to a particular
/// tab.
pub enum ScriptDevtoolControlMsg {
    /// Report a new JS error message
    ReportConsoleMsg(String),
}
