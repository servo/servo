/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use std::collections::HashMap;

use base::id::{BrowsingContextId, WebViewId};
use cookie::Cookie;
use euclid::default::Rect as UntypedRect;
use euclid::{Rect, Size2D};
use hyper_serde::Serde;
use ipc_channel::ipc::IpcSender;
use keyboard_types::KeyboardEvent;
use keyboard_types::webdriver::Event as WebDriverInputEvent;
use pixels::Image;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use webdriver::common::{WebElement, WebFrame, WebWindow};
use webdriver::error::ErrorStatus;
use webrender_api::units::DeviceIntSize;

use crate::{MouseButton, MouseButtonAction};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct WebDriverMessageId(pub usize);

/// Messages to the constellation originating from the WebDriver server.
#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    /// Get the window size.
    GetWindowSize(WebViewId, IpcSender<Size2D<f32, CSSPixel>>),
    /// Load a URL in the top-level browsing context with the given ID.
    LoadUrl(WebViewId, ServoUrl, IpcSender<WebDriverLoadStatus>),
    /// Refresh the top-level browsing context with the given ID.
    Refresh(WebViewId, IpcSender<WebDriverLoadStatus>),
    /// Pass a webdriver command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverScriptCommand),
    /// Act as if keys were pressed in the browsing context with the given ID.
    SendKeys(BrowsingContextId, Vec<WebDriverInputEvent>),
    /// Act as if keys were pressed or release in the browsing context with the given ID.
    KeyboardAction(BrowsingContextId, KeyboardEvent),
    /// Act as if the mouse was clicked in the browsing context with the given ID.
    MouseButtonAction(
        WebViewId,
        MouseButtonAction,
        MouseButton,
        f32,
        f32,
        WebDriverMessageId,
        IpcSender<WebDriverCommandResponse>,
    ),
    /// Act as if the mouse was moved in the browsing context with the given ID.
    MouseMoveAction(
        WebViewId,
        f32,
        f32,
        WebDriverMessageId,
        IpcSender<WebDriverCommandResponse>,
    ),
    /// Act as if the mouse wheel is scrolled in the browsing context given the given ID.
    WheelScrollAction(WebViewId, f32, f32, f64, f64),
    /// Set the window size.
    SetWindowSize(WebViewId, DeviceIntSize, IpcSender<Size2D<f32, CSSPixel>>),
    /// Take a screenshot of the window.
    TakeScreenshot(
        WebViewId,
        Option<Rect<f32, CSSPixel>>,
        IpcSender<Option<Image>>,
    ),
    /// Create a new webview that loads about:blank. The constellation will use
    /// the provided channels to return the top level browsing context id
    /// associated with the new webview, and a notification when the initial
    /// load is complete.
    NewWebView(
        WebViewId,
        IpcSender<WebViewId>,
        IpcSender<WebDriverLoadStatus>,
    ),
    /// Close the webview associated with the provided id.
    CloseWebView(WebViewId),
    /// Focus the webview associated with the provided id.
    FocusWebView(WebViewId),
}

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
    DeleteCookie(String, IpcSender<Result<(), ErrorStatus>>),
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
    GetComputedRole(String, IpcSender<Result<Option<String>, ErrorStatus>>),
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
    GetElementRect(String, IpcSender<Result<UntypedRect<f64>, ErrorStatus>>),
    GetElementTagName(String, IpcSender<Result<String, ErrorStatus>>),
    GetElementText(String, IpcSender<Result<String, ErrorStatus>>),
    GetElementInViewCenterPoint(String, IpcSender<Result<Option<(i64, i64)>, ErrorStatus>>),
    GetBoundingClientRect(String, IpcSender<Result<UntypedRect<f32>, ErrorStatus>>),
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
    Int(i32),
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
    JSException(WebDriverJSValue),
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
pub enum WebDriverCommandResponse {
    WebDriverInputComplete(WebDriverMessageId),
    WebDriverLoadStatus(WebDriverLoadStatus),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverLoadStatus {
    Complete,
    Timeout,
    Canceled,
}
