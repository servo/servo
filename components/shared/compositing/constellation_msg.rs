/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use base::id::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId, WebViewId};
use base::Epoch;
use embedder_traits::Cursor;
use ipc_channel::ipc::IpcSender;
use keyboard_types::KeyboardEvent;
use script_traits::{
    AnimationTickType, CompositorEvent, GamepadEvent, LogEntry, MediaSessionActionType,
    TraversalDirection, WebDriverCommandMsg, WindowSizeData, WindowSizeType,
};
use servo_url::ServoUrl;

/// Messages to the constellation.
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
    GetFocusTopLevelBrowsingContext(IpcSender<Option<TopLevelBrowsingContextId>>),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    /// Inform the constellation of a key event.
    Keyboard(KeyboardEvent),
    /// Whether to allow script to navigate.
    AllowNavigationResponse(PipelineId, bool),
    /// Request to load a page.
    LoadUrl(TopLevelBrowsingContextId, ServoUrl),
    /// Clear the network cache.
    ClearCache,
    /// Request to traverse the joint session history of the provided browsing context.
    TraverseHistory(TopLevelBrowsingContextId, TraversalDirection),
    /// Inform the constellation of a window being resized.
    WindowSize(TopLevelBrowsingContextId, WindowSizeData, WindowSizeType),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload a top-level browsing context.
    Reload(TopLevelBrowsingContextId),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<TopLevelBrowsingContextId>, Option<String>, LogEntry),
    /// Create a new top level browsing context.
    NewWebView(ServoUrl, TopLevelBrowsingContextId),
    /// A top level browsing context is created in both constellation and compositor.
    WebViewOpened(TopLevelBrowsingContextId),
    /// Close a top level browsing context.
    CloseWebView(TopLevelBrowsingContextId),
    /// Panic a top level browsing context.
    SendError(Option<TopLevelBrowsingContextId>, String),
    /// Make a webview focused.
    FocusWebView(TopLevelBrowsingContextId),
    /// Make none of the webviews focused.
    BlurWebView,
    /// Forward an event to the script task of the given pipeline.
    ForwardEvent(PipelineId, CompositorEvent),
    /// Requesting a change to the onscreen cursor.
    SetCursor(Cursor),
    /// Enable the sampling profiler, with a given sampling rate and max total sampling duration.
    EnableProfiler(Duration, Duration),
    /// Disable the sampling profiler.
    DisableProfiler,
    /// Request to exit from fullscreen mode
    ExitFullScreen(TopLevelBrowsingContextId),
    /// Media session action.
    MediaSessionAction(MediaSessionActionType),
    /// Set whether to use less resources, by stopping animations and running timers at a heavily limited rate.
    SetWebViewThrottled(TopLevelBrowsingContextId, bool),
    /// Virtual keyboard was dismissed
    IMEDismissed,
    /// Notify the embedder that it needs to present a new frame.
    ReadyToPresent(Vec<WebViewId>),
    /// Gamepad state has changed
    Gamepad(GamepadEvent),
}

impl fmt::Debug for ConstellationMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "ConstellationMsg::{}", self.variant_name())
    }
}

impl ConstellationMsg {
    /// Return the variant name, for error logging that happens after the message is consumed.
    pub fn variant_name(&self) -> &'static str {
        use self::ConstellationMsg::*;
        match *self {
            Exit => "Exit",
            GetBrowsingContext(..) => "GetBrowsingContext",
            GetPipeline(..) => "GetPipeline",
            GetFocusTopLevelBrowsingContext(..) => "GetFocusTopLevelBrowsingContext",
            IsReadyToSaveImage(..) => "IsReadyToSaveImage",
            Keyboard(..) => "Keyboard",
            AllowNavigationResponse(..) => "AllowNavigationResponse",
            LoadUrl(..) => "LoadUrl",
            TraverseHistory(..) => "TraverseHistory",
            WindowSize(..) => "WindowSize",
            TickAnimation(..) => "TickAnimation",
            WebDriverCommand(..) => "WebDriverCommand",
            Reload(..) => "Reload",
            LogEntry(..) => "LogEntry",
            NewWebView(..) => "NewWebView",
            WebViewOpened(..) => "WebViewOpened",
            CloseWebView(..) => "CloseWebView",
            FocusWebView(..) => "FocusWebView",
            BlurWebView => "BlurWebView",
            SendError(..) => "SendError",
            ForwardEvent(..) => "ForwardEvent",
            SetCursor(..) => "SetCursor",
            EnableProfiler(..) => "EnableProfiler",
            DisableProfiler => "DisableProfiler",
            ExitFullScreen(..) => "ExitFullScreen",
            MediaSessionAction(..) => "MediaSessionAction",
            SetWebViewThrottled(..) => "SetWebViewThrottled",
            IMEDismissed => "IMEDismissed",
            ClearCache => "ClearCache",
            ReadyToPresent(..) => "ReadyToPresent",
            Gamepad(..) => "Gamepad",
        }
    }
}
