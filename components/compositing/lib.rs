/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate log;

pub use crate::compositor::CompositingReason;
pub use crate::compositor::IOCompositor;
pub use crate::compositor::ShutdownState;
pub use crate::compositor_thread::CompositorProxy;
use embedder_traits::Cursor;
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use keyboard_types::KeyboardEvent;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
use msg::constellation_msg::{BrowsingContextId, TraversalDirection};
use script_traits::{
    AnimationTickType, LogEntry, WebDriverCommandMsg, WindowSizeData, WindowSizeType,
};
use script_traits::{
    CompositorEvent, ConstellationControlMsg, LayoutControlMsg, MediaSessionActionType,
};
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

mod compositor;
pub mod compositor_thread;
#[cfg(feature = "gl")]
mod gl;
mod touch;
pub mod windowing;

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub children: Vec<SendableFrameTree>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub layout_chan: IpcSender<LayoutControlMsg>,
}

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
    WindowSize(
        Option<TopLevelBrowsingContextId>,
        WindowSizeData,
        WindowSizeType,
    ),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload a top-level browsing context.
    Reload(TopLevelBrowsingContextId),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<TopLevelBrowsingContextId>, Option<String>, LogEntry),
    /// Create a new top level browsing context.
    NewBrowser(ServoUrl, TopLevelBrowsingContextId),
    /// Close a top level browsing context.
    CloseBrowser(TopLevelBrowsingContextId),
    /// Panic a top level browsing context.
    SendError(Option<TopLevelBrowsingContextId>, String),
    /// Make browser visible.
    SelectBrowser(TopLevelBrowsingContextId),
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
    /// Toggle browser visibility.
    ChangeBrowserVisibility(TopLevelBrowsingContextId, bool),
    /// Virtual keyboard was dismissed
    IMEDismissed,
}

impl fmt::Debug for ConstellationMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::ConstellationMsg::*;
        let variant = match *self {
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
            NewBrowser(..) => "NewBrowser",
            CloseBrowser(..) => "CloseBrowser",
            SendError(..) => "SendError",
            SelectBrowser(..) => "SelectBrowser",
            ForwardEvent(..) => "ForwardEvent",
            SetCursor(..) => "SetCursor",
            EnableProfiler(..) => "EnableProfiler",
            DisableProfiler => "DisableProfiler",
            ExitFullScreen(..) => "ExitFullScreen",
            MediaSessionAction(..) => "MediaSessionAction",
            ChangeBrowserVisibility(..) => "ChangeBrowserVisibility",
            IMEDismissed => "IMEDismissed",
            ClearCache => "ClearCache",
        };
        write!(formatter, "ConstellationMsg::{}", variant)
    }
}
