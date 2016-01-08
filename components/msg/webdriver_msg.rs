/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation_msg::{PipelineId, WindowSizeData};
use ipc_channel::ipc::IpcSender;
use rustc_serialize::json::{Json, ToJson};
use url::Url;

#[derive(Deserialize, Serialize)]
pub enum WebDriverScriptCommand {
    ExecuteScript(String, IpcSender<WebDriverJSResult>),
    ExecuteAsyncScript(String, IpcSender<WebDriverJSResult>),
    FindElementCSS(String, IpcSender<Result<Option<String>, ()>>),
    FindElementsCSS(String, IpcSender<Result<Vec<String>, ()>>),
    FocusElement(String, IpcSender<Result<(), ()>>),
    GetActiveElement(IpcSender<Option<String>>),
    GetElementAttribute(String, String, IpcSender<Result<Option<String>, ()>>),
    GetElementCSS(String, String, IpcSender<Result<String, ()>>),
    GetElementTagName(String, IpcSender<Result<String, ()>>),
    GetElementText(String, IpcSender<Result<String, ()>>),
    GetFrameId(WebDriverFrameId, IpcSender<Result<Option<PipelineId>, ()>>),
    GetUrl(IpcSender<Url>),
    GetWindowSize(IpcSender<Option<WindowSizeData>>),
    SetWindowSize(i32, i32, IpcSender<WebDriverJSResult>),
    Close(IpcSender<Result<(), ()>>),
    IsEnabled(String, IpcSender<Result<bool, ()>>),
    IsSelected(String, IpcSender<Result<bool, ()>>),
    GetTitle(IpcSender<String>)
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverJSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    // TODO: Object and WebElement
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverJSError {
    Timeout,
    UnknownType
}

pub type WebDriverJSResult = Result<WebDriverJSValue, WebDriverJSError>;

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub enum LoadStatus {
    LoadComplete,
    LoadTimeout
}
