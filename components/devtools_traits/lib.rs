/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

extern crate servo_msg = "msg";

use servo_msg::constellation_msg::PipelineId;

pub type DevtoolsControlChan = Sender<DevtoolsControlMsg>;
pub type DevtoolsControlPort = Receiver<DevtoolScriptControlMsg>;

pub enum DevtoolsControlMsg {
    NewGlobal(PipelineId, Sender<DevtoolScriptControlMsg>),
    ServerExitMsg
}

pub enum EvaluateJSReply {
    VoidValue,
    NullValue,
    BooleanValue(bool),
    NumberValue(f64),
    StringValue(String),
    ActorValue(String),
}

pub enum DevtoolScriptControlMsg {
    EvaluateJS(PipelineId, String, Sender<EvaluateJSReply>),
}

pub enum ScriptDevtoolControlMsg {
    ReportConsoleMsg(String),
}
