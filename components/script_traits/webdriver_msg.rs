/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use cookie_rs::Cookie;
use euclid::rect::Rect;
use hyper_serde::Serde;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use rustc_serialize::json::{Json, ToJson};
use servo_url::ServoUrl;

#[derive(Deserialize, Serialize)]
pub enum WebDriverScriptCommand {
    AddCookie(#[serde(deserialize_with = "::hyper_serde::deserialize",
                serialize_with = "::hyper_serde::serialize")]
              Cookie,
              IpcSender<Result<(), WebDriverCookieError>>),
    ExecuteScript(String, IpcSender<WebDriverJSResult>),
    ExecuteAsyncScript(String, IpcSender<WebDriverJSResult>),
    FindElementCSS(String, IpcSender<Result<Option<String>, ()>>),
    FindElementsCSS(String, IpcSender<Result<Vec<String>, ()>>),
    FocusElement(String, IpcSender<Result<(), ()>>),
    GetActiveElement(IpcSender<Option<String>>),
    GetCookie(String, IpcSender<Vec<Serde<Cookie>>>),
    GetCookies(IpcSender<Vec<Serde<Cookie>>>),
    GetElementAttribute(String, String, IpcSender<Result<Option<String>, ()>>),
    GetElementCSS(String, String, IpcSender<Result<String, ()>>),
    GetElementRect(String, IpcSender<Result<Rect<f64>, ()>>),
    GetElementTagName(String, IpcSender<Result<String, ()>>),
    GetElementText(String, IpcSender<Result<String, ()>>),
    GetFrameId(WebDriverFrameId, IpcSender<Result<Option<PipelineId>, ()>>),
    GetUrl(IpcSender<ServoUrl>),
    IsEnabled(String, IpcSender<Result<bool, ()>>),
    IsSelected(String, IpcSender<Result<bool, ()>>),
    GetTitle(IpcSender<String>),
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverCookieError {
    InvalidDomain,
    UnableToSetCookie,
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverJSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String), // TODO: Object and WebElement
}

#[derive(Deserialize, Serialize)]
pub enum WebDriverJSError {
    Timeout,
    UnknownType,
    /// Occurs when handler received an event message for a layout channel that is not
    /// associated with the current script thread
    BrowsingContextNotFound,
}

pub type WebDriverJSResult = Result<WebDriverJSValue, WebDriverJSError>;

#[derive(Deserialize, Serialize)]
pub enum WebDriverFrameId {
    Short(u16),
    Element(String),
    Parent,
}

impl ToJson for WebDriverJSValue {
    fn to_json(&self) -> Json {
        match *self {
            WebDriverJSValue::Undefined => Json::Null,
            WebDriverJSValue::Null => Json::Null,
            WebDriverJSValue::Boolean(ref x) => x.to_json(),
            WebDriverJSValue::Number(ref x) => x.to_json(),
            WebDriverJSValue::String(ref x) => x.to_json(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum LoadStatus {
    LoadComplete,
    LoadTimeout,
}
