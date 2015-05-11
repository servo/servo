/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_traits"]
#![crate_type = "rlib"]

extern crate rustc_serialize;
use rustc_serialize::json::{Json, ToJson};

use std::sync::mpsc::Sender;

pub enum WebDriverScriptCommand {
    EvaluateJS(String, Sender<Result<EvaluateJSReply, ()>>),
    FindElementCSS(String, Sender<Result<Option<String>, ()>>),
    FindElementsCSS(String, Sender<Result<Vec<String>, ()>>),
    GetActiveElement(Sender<Option<String>>),
    GetElementTagName(String, Sender<Result<String, ()>>),
    GetElementText(String, Sender<Result<String, ()>>),
    GetTitle(Sender<String>)
}

pub enum EvaluateJSReply {
    VoidValue,
    NullValue,
    BooleanValue(bool),
    NumberValue(f64),
    StringValue(String),
    // TODO: ObjectValue and WebElementValue
}

impl ToJson for EvaluateJSReply {
    fn to_json(&self) -> Json {
        match self {
            &EvaluateJSReply::VoidValue => Json::Null,
            &EvaluateJSReply::NullValue => Json::Null,
            &EvaluateJSReply::BooleanValue(ref x) => x.to_json(),
            &EvaluateJSReply::NumberValue(ref x) => x.to_json(),
            &EvaluateJSReply::StringValue(ref x) => x.to_json()
        }
    }
}
