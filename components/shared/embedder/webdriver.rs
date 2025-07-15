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
use pixels::RasterImage;
use serde::{Deserialize, Serialize};
use servo_geometry::{DeviceIndependentIntRect, DeviceIndependentIntSize, DeviceIndependentPixel};
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use webdriver::common::{WebElement, WebFrame, WebWindow};
use webdriver::error::ErrorStatus;
use webrender_api::units::DevicePixel;

use crate::{MouseButton, MouseButtonAction};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct WebDriverMessageId(pub usize);

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverUserPromptAction {
    Accept,
    Dismiss,
}

/// Messages to the constellation originating from the WebDriver server.
#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    /// Used in the initialization of the WebDriver server to set the sender for sending responses
    /// back to the WebDriver client. It is set to constellation for now
    SetWebDriverResponseSender(IpcSender<WebDriverCommandResponse>),
    /// Get the window rectangle.
    GetWindowRect(WebViewId, IpcSender<DeviceIndependentIntRect>),
    /// Get the viewport size.
    GetViewportSize(WebViewId, IpcSender<Size2D<u32, DevicePixel>>),
    /// Load a URL in the top-level browsing context with the given ID.
    LoadUrl(WebViewId, ServoUrl, IpcSender<WebDriverLoadStatus>),
    /// Refresh the top-level browsing context with the given ID.
    Refresh(WebViewId, IpcSender<WebDriverLoadStatus>),
    /// Navigate the webview with the given ID to the previous page in the browsing context's history.
    GoBack(WebViewId, IpcSender<WebDriverLoadStatus>),
    /// Navigate the webview with the given ID to the next page in the browsing context's history.
    GoForward(WebViewId, IpcSender<WebDriverLoadStatus>),
    /// Pass a webdriver command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverScriptCommand),
    /// Act as if keys were pressed in the browsing context with the given ID.
    SendKeys(WebViewId, Vec<WebDriverInputEvent>),
    /// Act as if keys were pressed or release in the browsing context with the given ID.
    KeyboardAction(
        WebViewId,
        KeyboardEvent,
        // Should never be None.
        Option<WebDriverMessageId>,
    ),
    /// Act as if the mouse was clicked in the browsing context with the given ID.
    MouseButtonAction(
        WebViewId,
        MouseButtonAction,
        MouseButton,
        f32,
        f32,
        // Should never be None.
        Option<WebDriverMessageId>,
    ),
    /// Act as if the mouse was moved in the browsing context with the given ID.
    MouseMoveAction(
        WebViewId,
        f32,
        f32,
        // None if it's not the last `perform_pointer_move` since we only
        // expect one response from constellation for each tick actions.
        Option<WebDriverMessageId>,
    ),
    /// Act as if the mouse wheel is scrolled in the browsing context given the given ID.
    WheelScrollAction(
        WebViewId,
        f32,
        f32,
        f64,
        f64,
        // None if it's not the last `perform_wheel_scroll` since we only
        // expect one response from constellation for each tick actions.
        Option<WebDriverMessageId>,
    ),
    /// Set the window size.
    SetWindowSize(
        WebViewId,
        DeviceIndependentIntSize,
        IpcSender<Size2D<i32, DeviceIndependentPixel>>,
    ),
    /// Take a screenshot of the window.
    TakeScreenshot(
        WebViewId,
        Option<Rect<f32, CSSPixel>>,
        IpcSender<Option<RasterImage>>,
    ),
    /// Create a new webview that loads about:blank. The constellation will use
    /// the provided channels to return the top level browsing context id
    /// associated with the new webview, and a notification when the initial
    /// load is complete.
    NewWebView(IpcSender<WebViewId>, IpcSender<WebDriverLoadStatus>),
    /// Close the webview associated with the provided id.
    CloseWebView(WebViewId),
    /// Focus the webview associated with the provided id.
    FocusWebView(WebViewId),
    /// Get focused webview.
    GetFocusedWebView(IpcSender<Option<WebViewId>>),
    /// Check whether top-level browsing context is open.
    IsWebViewOpen(WebViewId, IpcSender<bool>),
    /// Check whether browsing context is open.
    IsBrowsingContextOpen(BrowsingContextId, IpcSender<bool>),
    HandleUserPrompt(
        WebViewId,
        WebDriverUserPromptAction,
        IpcSender<Result<(), ()>>,
    ),
    GetAlertText(WebViewId, IpcSender<Result<String, ()>>),
    AddLoadStatusSender(WebViewId, IpcSender<WebDriverLoadStatus>),
    RemoveLoadStatusSender(WebViewId),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverScriptCommand {
    AddCookie(
        #[serde(
            deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize"
        )]
        Cookie<'static>,
        IpcSender<Result<(), ErrorStatus>>,
    ),
    DeleteCookies(IpcSender<Result<(), ErrorStatus>>),
    DeleteCookie(String, IpcSender<Result<(), ErrorStatus>>),
    ExecuteScript(String, IpcSender<WebDriverJSResult>),
    ExecuteAsyncScript(String, IpcSender<WebDriverJSResult>),
    FindElementsCSSSelector(String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsLinkText(String, bool, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsTagName(String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsXpathSelector(String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementElementsCSSSelector(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementElementsLinkText(
        String,
        String,
        bool,
        IpcSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindElementElementsTagName(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindElementElementsXPathSelector(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindShadowElementsCSSSelector(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindShadowElementsLinkText(
        String,
        String,
        bool,
        IpcSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindShadowElementsTagName(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    FindShadowElementsXPathSelector(String, String, IpcSender<Result<Vec<String>, ErrorStatus>>),
    GetElementShadowRoot(String, IpcSender<Result<Option<String>, ErrorStatus>>),
    ElementClick(String, IpcSender<Result<Option<String>, ErrorStatus>>),
    GetActiveElement(IpcSender<Option<String>>),
    GetComputedRole(String, IpcSender<Result<Option<String>, ErrorStatus>>),
    GetCookie(
        String,
        IpcSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>,
    ),
    GetCookies(IpcSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>),
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
    GetParentFrameId(IpcSender<Result<BrowsingContextId, ErrorStatus>>),
    GetUrl(IpcSender<ServoUrl>),
    GetPageSource(IpcSender<Result<String, ErrorStatus>>),
    IsEnabled(String, IpcSender<Result<bool, ErrorStatus>>),
    IsSelected(String, IpcSender<Result<bool, ErrorStatus>>),
    GetTitle(IpcSender<String>),
    /// Match the element type before sending the event for webdriver `element send keys`.
    WillSendKeys(String, String, bool, IpcSender<Result<bool, ErrorStatus>>),
    IsDocumentReadyStateComplete(IpcSender<bool>),
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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebDriverCommandResponse {
    pub id: WebDriverMessageId,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverLoadStatus {
    Complete,
    Timeout,
    Canceled,
    Blocked,
}
