/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_traits"]
#![crate_type = "rlib"]

extern crate rustc_serialize;
use rustc_serialize::json::{Json, ToJson};
use std::sync::mpsc::Sender;

pub enum WebDriverScriptCommand {
    ExecuteScript(String, Sender<WebDriverJSResult>),
    ExecuteAsyncScript(String, Sender<WebDriverJSResult>),
    FindElementCSS(String, Sender<Result<Option<String>, ()>>),
    FindElementsCSS(String, Sender<Result<Vec<String>, ()>>),
    GetActiveElement(Sender<Option<String>>),
    GetElementTagName(String, Sender<Result<String, ()>>),
    GetElementText(String, Sender<Result<String, ()>>),
    GetTitle(Sender<String>)
}

pub enum WebDriverJSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    // TODO: ObjectValue and WebElementValue
}

pub enum WebDriverJSError {
    Timeout,
    UnknownType
}

pub type WebDriverJSResult = Result<WebDriverJSValue, WebDriverJSError>;

impl ToJson for WebDriverJSValue {
    fn to_json(&self) -> Json {
        match self {
            &WebDriverJSValue::Undefined => Json::Null,
            &WebDriverJSValue::Null => Json::Null,
            &WebDriverJSValue::Boolean(ref x) => x.to_json(),
            &WebDriverJSValue::Number(ref x) => x.to_json(),
            &WebDriverJSValue::String(ref x) => x.to_json()
        }
    }
}

pub struct LoadComplete;
