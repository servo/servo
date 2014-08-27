/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "devtools_traits"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

pub type DevtoolsControlChan = Sender<DevtoolsControlMsg>;
pub type DevtoolsControlPort = Receiver<DevtoolScriptControlMsg>;

pub enum DevtoolsControlMsg {
    NewGlobal(Sender<DevtoolScriptControlMsg>),
    ServerExitMsg
}

pub enum EvaluateJSReply {
    VoidValue,
    NullValue,
    NumberValue(f64),
    StringValue(String),
    ActorValue(String),
}

pub enum DevtoolScriptControlMsg {
    EvaluateJS(String, Sender<EvaluateJSReply>),
}

pub enum ScriptDevtoolControlMsg {
    ReportConsoleMsg(String),
}
