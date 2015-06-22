/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc_serialize::json::{Json, ToJson};
use constellation_msg::{PipelineId, SubpageId};

use std::sync::mpsc::Sender;

pub enum WebDriverScriptCommand {
    ExecuteScript(String, Sender<WebDriverJSResult>),
    ExecuteAsyncScript(String, Sender<WebDriverJSResult>),
    FindElementCSS(String, Sender<Result<Option<String>, ()>>),
    FindElementsCSS(String, Sender<Result<Vec<String>, ()>>),
    GetActiveElement(Sender<Option<String>>),
    GetElementTagName(String, Sender<Result<String, ()>>),
    GetElementText(String, Sender<Result<String, ()>>),
    GetFrameId(WebDriverFrameId, Sender<Result<Option<(PipelineId, SubpageId)>, ()>>),
    GetTitle(Sender<String>)
}

pub enum WebDriverJSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    // TODO: Object and WebElement
}

pub enum WebDriverJSError {
    Timeout,
    UnknownType
}

pub type WebDriverJSResult = Result<WebDriverJSValue, WebDriverJSError>;

pub enum WebDriverFrameId {
    Short(u16),
    Element(String),
    Parent
}

impl ToJson for WebDriverJSValue {
    fn to_json(&self) -> Json {
        match *self {
            WebDriverJSValue::Undefined => Json::Null,
            WebDriverJSValue::Null => Json::Null,
            WebDriverJSValue::Boolean(ref x) => x.to_json(),
            WebDriverJSValue::Number(ref x) => x.to_json(),
            WebDriverJSValue::String(ref x) => x.to_json()
        }
    }
}

pub enum LoadStatus {
    LoadComplete,
    LoadTimeout
}
