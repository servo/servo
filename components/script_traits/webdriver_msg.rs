/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use cookie::Cookie;
use euclid::Rect;
use hyper_serde::Serde;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::BrowsingContextId;
use servo_url::ServoUrl;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverScriptCommand {
    AddCookie(
        #[serde(
            deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize"
        )]
        Cookie<'static>,
        IpcSender<Result<(), WebDriverCookieError>>,
    ),
    DeleteCookies(IpcSender<Result<(), ()>>),
    ExecuteScript(String, IpcSender<WebDriverJSResult>),
    ExecuteAsyncScript(String, IpcSender<WebDriverJSResult>),
    FindElementCSS(String, IpcSender<Result<Option<String>, ()>>),
    FindElementLinkText(String, bool, IpcSender<Result<Option<String>, ()>>),
    FindElementTagName(String, IpcSender<Result<Option<String>, ()>>),
    FindElementsCSS(String, IpcSender<Result<Vec<String>, ()>>),
    FindElementsLinkText(String, bool, IpcSender<Result<Vec<String>, ()>>),
    FindElementsTagName(String, IpcSender<Result<Vec<String>, ()>>),
    FindElementElementCSS(String, String, IpcSender<Result<Option<String>, ()>>),
    FindElementElementLinkText(String, String, bool, IpcSender<Result<Option<String>, ()>>),
    FindElementElementTagName(String, String, IpcSender<Result<Option<String>, ()>>),
    FindElementElementsCSS(String, String, IpcSender<Result<Vec<String>, ()>>),
    FindElementElementsLinkText(String, String, bool, IpcSender<Result<Vec<String>, ()>>),
    FindElementElementsTagName(String, String, IpcSender<Result<Vec<String>, ()>>),
    FocusElement(String, IpcSender<Result<(), ()>>),
    GetActiveElement(IpcSender<Option<String>>),
    GetCookie(String, IpcSender<Vec<Serde<Cookie<'static>>>>),
    GetCookies(IpcSender<Vec<Serde<Cookie<'static>>>>),
    GetElementAttribute(String, String, IpcSender<Result<Option<String>, ()>>),
    GetElementCSS(String, String, IpcSender<Result<String, ()>>),
    GetElementRect(String, IpcSender<Result<Rect<f64>, ()>>),
    GetElementTagName(String, IpcSender<Result<String, ()>>),
    GetElementText(String, IpcSender<Result<String, ()>>),
    GetBrowsingContextId(WebDriverFrameId, IpcSender<Result<BrowsingContextId, ()>>),
    GetUrl(IpcSender<ServoUrl>),
    GetPageSource(IpcSender<Result<String, ()>>),
    IsEnabled(String, IpcSender<Result<bool, ()>>),
    IsSelected(String, IpcSender<Result<bool, ()>>),
    GetTitle(IpcSender<String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverCookieError {
    InvalidDomain,
    UnableToSetCookie,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverJSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String), // TODO: Object and WebElement
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverJSError {
    Timeout,
    UnknownType,
    /// Occurs when handler received an event message for a layout channel that is not
    /// associated with the current script thread
    BrowsingContextNotFound,
}

pub type WebDriverJSResult = Result<WebDriverJSValue, WebDriverJSError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverFrameId {
    Short(u16),
    Element(String),
    Parent,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LoadStatus {
    LoadComplete,
    LoadTimeout,
}
