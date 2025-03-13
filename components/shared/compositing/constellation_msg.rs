/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use base::Epoch;
use base::id::{BrowsingContextId, PipelineId, WebViewId};
use embedder_traits::{
    Cursor, InputEvent, MediaSessionActionType, Theme, TraversalDirection, WebDriverCommandMsg,
};
use ipc_channel::ipc::IpcSender;
use script_traits::{AnimationTickType, LogEntry, WindowSizeData, WindowSizeType};
use servo_url::ServoUrl;
use strum_macros::IntoStaticStr;
use webrender_traits::CompositorHitTestResult;

/// Messages to the constellation.
#[derive(IntoStaticStr)]
pub enum ConstellationMsg {
    /// Exit the constellation.
    Exit,
    /// Request that the constellation send the BrowsingContextId corresponding to the document
    /// with the provided pipeline id
    GetBrowsingContext(PipelineId, IpcSender<Option<BrowsingContextId>>),
    /// Request that the constellation send the current pipeline id for the provided
    /// browsing context id, over a provided channel.
    GetPipeline(BrowsingContextId, IpcSender<Option<PipelineId>>),
    /// Request that the constellation send the current focused top-level browsing context id,
    /// over a provided channel.
    GetFocusTopLevelBrowsingContext(IpcSender<Option<WebViewId>>),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    /// Whether to allow script to navigate.
    AllowNavigationResponse(PipelineId, bool),
    /// Request to load a page.
    LoadUrl(WebViewId, ServoUrl),
    /// Clear the network cache.
    ClearCache,
    /// Request to traverse the joint session history of the provided browsing context.
    TraverseHistory(WebViewId, TraversalDirection),
    /// Inform the constellation of a window being resized.
    WindowSize(WebViewId, WindowSizeData, WindowSizeType),
    /// Inform the constellation of a theme change.
    ThemeChange(Theme),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload a top-level browsing context.
    Reload(WebViewId),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<WebViewId>, Option<String>, LogEntry),
    /// Create a new top level browsing context.
    NewWebView(ServoUrl, WebViewId),
    /// Close a top level browsing context.
    CloseWebView(WebViewId),
    /// Panic a top level browsing context.
    SendError(Option<WebViewId>, String),
    /// Make a webview focused.
    FocusWebView(WebViewId),
    /// Make none of the webviews focused.
    BlurWebView,
    /// Forward an input event to an appropriate ScriptTask.
    ForwardInputEvent(WebViewId, InputEvent, Option<CompositorHitTestResult>),
    /// Requesting a change to the onscreen cursor.
    SetCursor(WebViewId, Cursor),
    /// Enable the sampling profiler, with a given sampling rate and max total sampling duration.
    ToggleProfiler(Duration, Duration),
    /// Request to exit from fullscreen mode
    ExitFullScreen(WebViewId),
    /// Media session action.
    MediaSessionAction(MediaSessionActionType),
    /// Set whether to use less resources, by stopping animations and running timers at a heavily limited rate.
    SetWebViewThrottled(WebViewId, bool),
}

impl fmt::Debug for ConstellationMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let variant_string: &'static str = self.into();
        write!(formatter, "ConstellationMsg::{variant_string}")
    }
}
