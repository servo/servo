/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(missing_docs)]

use std::collections::HashMap;

use base::generic_channel::{GenericOneshotSender, GenericSender};
use base::id::{BrowsingContextId, WebViewId};
use cookie::Cookie;
use crossbeam_channel::Sender;
use euclid::default::Rect as UntypedRect;
use euclid::{Rect, Size2D};
use hyper_serde::Serde;
use image::RgbaImage;
use malloc_size_of_derive::MallocSizeOf;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use servo_geometry::DeviceIndependentIntRect;
use style_traits::CSSPixel;
use url::Url;
use webdriver::error::ErrorStatus;
use webrender_api::units::DevicePixel;

use crate::{InputEvent, JSValue, JavaScriptEvaluationError, ScreenshotCaptureError, TraversalId};

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

/// <https://html.spec.whatwg.org/multipage/#registerprotocolhandler()-automation-mode>
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum CustomHandlersAutomationMode {
    AutoAccept,
    AutoReject,
    #[default]
    None,
}

/// <https://w3c.github.io/webdriver/#new-window>
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum NewWindowTypeHint {
    Auto,
    Tab,
    Window,
}

/// Messages to the constellation originating from the WebDriver server.
#[derive(Debug)]
pub enum WebDriverCommandMsg {
    /// Get the window rectangle.
    GetWindowRect(WebViewId, GenericOneshotSender<DeviceIndependentIntRect>),
    /// Get the viewport size.
    GetViewportSize(WebViewId, GenericOneshotSender<Size2D<u32, DevicePixel>>),
    /// Load a URL in the top-level browsing context with the given ID.
    LoadUrl(WebViewId, Url, GenericSender<WebDriverLoadStatus>),
    /// Refresh the top-level browsing context with the given ID.
    Refresh(WebViewId, GenericSender<WebDriverLoadStatus>),
    /// Navigate the webview with the given ID to the previous page in the browsing context's history.
    GoBack(WebViewId, GenericSender<WebDriverLoadStatus>),
    /// Navigate the webview with the given ID to the next page in the browsing context's history.
    GoForward(WebViewId, GenericSender<WebDriverLoadStatus>),
    /// Pass a webdriver command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverScriptCommand),
    /// Dispatch an input event to the given [`WebView`]. Once the event has been handled in the
    /// page DOM a single message should be sent through the [`Sender`], if provided, informing the
    /// WebDriver server that the inpute event has been handled.
    InputEvent(WebViewId, InputEvent, Option<Sender<()>>),
    /// Set the outer window rectangle.
    SetWindowRect(
        WebViewId,
        DeviceIndependentIntRect,
        GenericOneshotSender<DeviceIndependentIntRect>,
    ),
    /// Maximize the window. Send back result window rectangle.
    MaximizeWebView(WebViewId, GenericOneshotSender<DeviceIndependentIntRect>),
    /// Take a screenshot of the viewport.
    TakeScreenshot(
        WebViewId,
        Option<Rect<f32, CSSPixel>>,
        Sender<Result<RgbaImage, ScreenshotCaptureError>>,
    ),
    /// Create a new webview that loads about:blank. The embedder will use
    /// the provided channels to return the top level browsing context id
    /// associated with the new webview, and sets a "load status sender" if provided.
    NewWindow(
        NewWindowTypeHint,
        GenericOneshotSender<WebViewId>,
        Option<GenericSender<WebDriverLoadStatus>>,
    ),
    /// Close the webview associated with the provided id.
    CloseWebView(WebViewId, GenericOneshotSender<()>),
    /// Focus the webview associated with the provided id.
    FocusWebView(WebViewId),
    /// Get focused webview. For now, this is only used when start new session.
    GetFocusedWebView(GenericOneshotSender<Option<WebViewId>>),
    /// Get webviews state
    GetAllWebViews(GenericOneshotSender<Vec<WebViewId>>),
    /// Check whether top-level browsing context is open.
    IsWebViewOpen(WebViewId, GenericOneshotSender<bool>),
    /// Check whether browsing context is open.
    IsBrowsingContextOpen(BrowsingContextId, GenericOneshotSender<bool>),
    CurrentUserPrompt(WebViewId, GenericOneshotSender<Option<WebDriverUserPrompt>>),
    HandleUserPrompt(
        WebViewId,
        WebDriverUserPromptAction,
        GenericOneshotSender<Result<String, ()>>,
    ),
    GetAlertText(WebViewId, GenericOneshotSender<Result<String, ()>>),
    SendAlertText(WebViewId, String),
    FocusBrowsingContext(BrowsingContextId),
    Shutdown,
    ResetAllCookies(Sender<()>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverScriptCommand {
    AddCookie(
        #[serde(
            deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize"
        )]
        Cookie<'static>,
        GenericSender<Result<(), ErrorStatus>>,
    ),
    DeleteCookies(GenericSender<Result<(), ErrorStatus>>),
    DeleteCookie(String, GenericSender<Result<(), ErrorStatus>>),
    ElementClear(String, GenericSender<Result<(), ErrorStatus>>),
    ExecuteScriptWithCallback(String, GenericSender<WebDriverJSResult>),
    FindElementsCSSSelector(String, GenericSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsLinkText(
        String,
        bool,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindElementsTagName(String, GenericSender<Result<Vec<String>, ErrorStatus>>),
    FindElementsXpathSelector(String, GenericSender<Result<Vec<String>, ErrorStatus>>),
    FindElementElementsCSSSelector(
        String,
        String,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindElementElementsLinkText(
        String,
        String,
        bool,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindElementElementsTagName(
        String,
        String,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindElementElementsXPathSelector(
        String,
        String,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindShadowElementsCSSSelector(
        String,
        String,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindShadowElementsLinkText(
        String,
        String,
        bool,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindShadowElementsTagName(
        String,
        String,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    FindShadowElementsXPathSelector(
        String,
        String,
        GenericSender<Result<Vec<String>, ErrorStatus>>,
    ),
    GetElementShadowRoot(String, GenericSender<Result<Option<String>, ErrorStatus>>),
    ElementClick(String, GenericSender<Result<Option<String>, ErrorStatus>>),
    GetKnownElement(String, GenericSender<Result<(), ErrorStatus>>),
    GetKnownShadowRoot(String, GenericSender<Result<(), ErrorStatus>>),
    GetKnownWindow(String, GenericSender<Result<(), ErrorStatus>>),
    GetActiveElement(GenericSender<Option<String>>),
    GetComputedRole(String, GenericSender<Result<Option<String>, ErrorStatus>>),
    GetCookie(
        String,
        GenericSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>,
    ),
    GetCookies(GenericSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>),
    GetElementAttribute(
        String,
        String,
        GenericSender<Result<Option<String>, ErrorStatus>>,
    ),
    GetElementProperty(String, String, GenericSender<Result<JSValue, ErrorStatus>>),
    GetElementCSS(String, String, GenericSender<Result<String, ErrorStatus>>),
    GetElementRect(String, GenericSender<Result<UntypedRect<f64>, ErrorStatus>>),
    GetElementTagName(String, GenericSender<Result<String, ErrorStatus>>),
    GetElementText(String, GenericSender<Result<String, ErrorStatus>>),
    GetElementInViewCenterPoint(
        String,
        GenericOneshotSender<Result<Option<(i64, i64)>, ErrorStatus>>,
    ),
    ScrollAndGetBoundingClientRect(String, GenericSender<Result<UntypedRect<f32>, ErrorStatus>>),
    GetBrowsingContextId(
        WebDriverFrameId,
        GenericSender<Result<BrowsingContextId, ErrorStatus>>,
    ),
    GetParentFrameId(GenericSender<Result<BrowsingContextId, ErrorStatus>>),
    GetUrl(GenericSender<String>),
    GetPageSource(GenericSender<Result<String, ErrorStatus>>),
    IsEnabled(String, GenericSender<Result<bool, ErrorStatus>>),
    IsSelected(String, GenericSender<Result<bool, ErrorStatus>>),
    GetTitle(GenericSender<String>),
    /// Deal with the case of input element for Element Send Keys, which does not send keys.
    WillSendKeys(
        String,
        String,
        bool,
        GenericSender<Result<bool, ErrorStatus>>,
    ),
    AddLoadStatusSender(WebViewId, GenericSender<WebDriverLoadStatus>),
    RemoveLoadStatusSender(WebViewId),
    SetProtocolHandlerAutomationMode(CustomHandlersAutomationMode),
}

pub type WebDriverJSResult = Result<JSValue, JavaScriptEvaluationError>;

#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverFrameId {
    Short(u16),
    Element(String),
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

/// A collection of [`GenericSender`]s that are used to asynchronously communicate
/// to a WebDriver server with information about application state.
#[derive(Clone, Default)]
pub struct WebDriverSenders {
    pub load_status_senders: FxHashMap<WebViewId, GenericSender<WebDriverLoadStatus>>,
    pub script_evaluation_interrupt_sender: Option<GenericSender<WebDriverJSResult>>,
    pub pending_traversals: HashMap<TraversalId, GenericSender<WebDriverLoadStatus>>,
}
