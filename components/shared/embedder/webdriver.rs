/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use std::collections::HashMap;

use base::generic_channel::GenericSender;
use base::id::{BrowsingContextId, WebViewId};
use cookie::Cookie;
use euclid::default::Rect as UntypedRect;
use euclid::{Rect, Size2D};
use hyper_serde::Serde;
use ipc_channel::ipc::IpcSender;
use keyboard_types::{CompositionEvent, KeyboardEvent};
use pixels::RasterImage;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_geometry::DeviceIndependentIntRect;
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use webdriver::error::ErrorStatus;
use webrender_api::units::DevicePixel;

use crate::{JSValue, MouseButton, MouseButtonAction, TraversalId};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct WebDriverMessageId(pub usize);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WebDriverUserPrompt {
    Alert,
    BeforeUnload,
    Confirm,
    Default,
    File,
    Prompt,
    FallbackDefault,
}

impl WebDriverUserPrompt {
    pub fn new_from_str(s: &str) -> Option<Self> {
        match s {
            "alert" => Some(WebDriverUserPrompt::Alert),
            "beforeUnload" => Some(WebDriverUserPrompt::BeforeUnload),
            "confirm" => Some(WebDriverUserPrompt::Confirm),
            "default" => Some(WebDriverUserPrompt::Default),
            "file" => Some(WebDriverUserPrompt::File),
            "prompt" => Some(WebDriverUserPrompt::Prompt),
            "fallbackDefault" => Some(WebDriverUserPrompt::FallbackDefault),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebDriverUserPromptAction {
    Accept,
    Dismiss,
    Ignore,
}

impl WebDriverUserPromptAction {
    pub fn new_from_str(s: &str) -> Option<Self> {
        match s {
            "accept" => Some(WebDriverUserPromptAction::Accept),
            "dismiss" => Some(WebDriverUserPromptAction::Dismiss),
            "ignore" => Some(WebDriverUserPromptAction::Ignore),
            _ => None,
        }
    }
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
    LoadUrl(WebViewId, ServoUrl, GenericSender<WebDriverLoadStatus>),
    /// Refresh the top-level browsing context with the given ID.
    Refresh(WebViewId, GenericSender<WebDriverLoadStatus>),
    /// Navigate the webview with the given ID to the previous page in the browsing context's history.
    GoBack(WebViewId, GenericSender<WebDriverLoadStatus>),
    /// Navigate the webview with the given ID to the next page in the browsing context's history.
    GoForward(WebViewId, GenericSender<WebDriverLoadStatus>),
    /// Pass a webdriver command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverScriptCommand),
    /// Dispatch composition event from element send keys command.
    DispatchComposition(WebViewId, CompositionEvent),
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
        f64,
        f64,
        f64,
        f64,
        // None if it's not the last `perform_wheel_scroll` since we only
        // expect one response from constellation for each tick actions.
        Option<WebDriverMessageId>,
    ),
    /// Set the outer window rectangle.
    SetWindowRect(
        WebViewId,
        DeviceIndependentIntRect,
        IpcSender<DeviceIndependentIntRect>,
    ),
    /// Maximize the window. Send back result window rectangle.
    MaximizeWebView(WebViewId, IpcSender<DeviceIndependentIntRect>),
    /// Take a screenshot of the viewport.
    TakeScreenshot(
        WebViewId,
        Option<Rect<f32, CSSPixel>>,
        IpcSender<Result<Option<RasterImage>, ()>>,
    ),
    /// Create a new webview that loads about:blank. The embedder will use
    /// the provided channels to return the top level browsing context id
    /// associated with the new webview, and sets a "load status sender" if provided.
    NewWebView(
        IpcSender<WebViewId>,
        Option<GenericSender<WebDriverLoadStatus>>,
    ),
    /// Close the webview associated with the provided id.
    CloseWebView(WebViewId, IpcSender<()>),
    /// Focus the webview associated with the provided id.
    FocusWebView(WebViewId),
    /// Get focused webview. For now, this is only used when start new session.
    GetFocusedWebView(IpcSender<Option<WebViewId>>),
    /// Get webviews state
    GetAllWebViews(IpcSender<Vec<WebViewId>>),
    /// Check whether top-level browsing context is open.
    IsWebViewOpen(WebViewId, IpcSender<bool>),
    /// Check whether browsing context is open.
    IsBrowsingContextOpen(BrowsingContextId, IpcSender<bool>),
    CurrentUserPrompt(WebViewId, IpcSender<Option<WebDriverUserPrompt>>),
    HandleUserPrompt(
        WebViewId,
        WebDriverUserPromptAction,
        IpcSender<Result<Option<String>, ()>>,
    ),
    GetAlertText(WebViewId, IpcSender<Result<String, ()>>),
    SendAlertText(WebViewId, String),
    FocusBrowsingContext(BrowsingContextId),
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
    ElementClear(String, IpcSender<Result<(), ErrorStatus>>),
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
    GetKnownElement(String, IpcSender<Result<(), ErrorStatus>>),
    GetKnownShadowRoot(String, IpcSender<Result<(), ErrorStatus>>),
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
    GetElementProperty(String, String, IpcSender<Result<JSValue, ErrorStatus>>),
    GetElementCSS(String, String, IpcSender<Result<String, ErrorStatus>>),
    GetElementRect(String, IpcSender<Result<UntypedRect<f64>, ErrorStatus>>),
    GetElementTagName(String, IpcSender<Result<String, ErrorStatus>>),
    GetElementText(String, IpcSender<Result<String, ErrorStatus>>),
    GetElementInViewCenterPoint(String, IpcSender<Result<Option<(i64, i64)>, ErrorStatus>>),
    ScrollAndGetBoundingClientRect(String, IpcSender<Result<UntypedRect<f32>, ErrorStatus>>),
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
    /// Deal with the case of input element for Element Send Keys, which does not send keys.
    WillSendKeys(String, String, bool, IpcSender<Result<bool, ErrorStatus>>),
    AddLoadStatusSender(WebViewId, GenericSender<WebDriverLoadStatus>),
    RemoveLoadStatusSender(WebViewId),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverJSError {
    /// Occurs when handler received an event message for a layout channel that is not
    /// associated with the current script thread
    BrowsingContextNotFound,
    DetachedShadowRoot,
    JSException(JSValue),
    JSError,
    StaleElementReference,
    Timeout,
    UnknownType,
}

pub type WebDriverJSResult = Result<JSValue, WebDriverJSError>;

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
    NavigationStart,
    // Navigation stops for any reason
    NavigationStop,
    // Document ready state is complete
    Complete,
    // Load timeout
    Timeout,
    // Navigation is blocked by a user prompt
    Blocked,
}

/// A collection of [`IpcSender`]s that are used to asynchronously communicate
/// to a WebDriver server with information about application state.
#[derive(Clone, Default)]
pub struct WebDriverSenders {
    pub load_status_senders: FxHashMap<WebViewId, GenericSender<WebDriverLoadStatus>>,
    pub script_evaluation_interrupt_sender: Option<IpcSender<WebDriverJSResult>>,
    pub pending_traversals: HashMap<TraversalId, GenericSender<WebDriverLoadStatus>>,
}
