/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use std::collections::HashMap;

use base::id::BrowsingContextId;
use cookie::Cookie;
use euclid::default::Rect;
use hyper_serde::Serde;
use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use webdriver::common::{WebElement, WebFrame, WebWindow};
use webdriver::error::ErrorStatus;

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
    DeleteCookies(IpcSender<Result<(), ErrorStatus>>),
    ExecuteScript(String, IpcSender<WebDriverJSResult>),
    ExecuteAsyncScript(String, IpcSender<WebDriverJSResult>),
    FindElementCSS(String, IpcSender<Result<Option<String>, ErrorStatus>>),
    FindElementLinkText(String, bool, IpcSender<Result<Option<String>, ErrorStatus>>),
    FindElementTagName(String, IpcSender<Result<Option<String>, ErrorStatus>>),
    FindElementsCSS(String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsLinkText(String, bool, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsTagName(String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementElementCSS(
        String,
        String,
        IpcSender<Result<Option<String>, ErrorStatus>>,
    ),
    FindElementElementLinkText(
        String,
        String,
        bool,
        IpcSender<Result<Option<String>, ErrorStatus>>,
    ),
    FindElementElementTagName(
        String,
        String,
        IpcSender<Result<Option<String>, ErrorStatus>>,
    ),
    FindElementElementsCSS(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementElementsLinkText(
        String,
        String,
        bool,
        IpcSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindElementElementsTagName(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FocusElement(String, IpcSender<Result<(), ErrorStatus>>),
    ElementClick(String, IpcSender<Result<Option<String>, ErrorStatus>>),
    GetActiveElement(IpcSender<Option<String>>),
    GetCookie(String, IpcSender<Vec<Serde<Cookie<'static>>>>),
    GetCookies(IpcSender<Vec<Serde<Cookie<'static>>>>),
    GetElementAttribute(
        String,
        String,
        IpcSender<Result<Option<String>, ErrorStatus>>,
    ),
    GetElementProperty(
        String,
        String,
        IpcSender<Result<WebDriverJSValue, ErrorStatus>>,
    ),
    GetElementCSS(String, String, IpcSender<Result<String, ErrorStatus>>),
    GetElementRect(String, IpcSender<Result<Rect<f64>, ErrorStatus>>),
    GetElementTagName(String, IpcSender<Result<String, ErrorStatus>>),
    GetElementText(String, IpcSender<Result<String, ErrorStatus>>),
    GetElementInViewCenterPoint(String, IpcSender<Result<Option<(i64, i64)>, ErrorStatus>>),
    GetBoundingClientRect(String, IpcSender<Result<Rect<f32>, ErrorStatus>>),
    GetBrowsingContextId(
        WebDriverFrameId,
        IpcSender<Result<BrowsingContextId, ErrorStatus>>,
    ),
    GetUrl(IpcSender<ServoUrl>),
    GetPageSource(IpcSender<Result<String, ErrorStatus>>),
    IsEnabled(String, IpcSender<Result<bool, ErrorStatus>>),
    IsSelected(String, IpcSender<Result<bool, ErrorStatus>>),
    GetTitle(IpcSender<String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverCookieError {
    InvalidDomain,
    UnableToSetCookie,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverJSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Element(WebElement),
    Frame(WebFrame),
    Window(WebWindow),
    ArrayLike(Vec<WebDriverJSValue>),
    Object(HashMap<String, WebDriverJSValue>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverJSError {
    /// Occurs when handler received an event message for a layout channel that is not
    /// associated with the current script thread
    BrowsingContextNotFound,
    JSError,
    StaleElementReference,
    Timeout,
    UnknownType,
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
