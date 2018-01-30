/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

use SendableFrameTree;
use compositor::CompositingReason;
use euclid::{Point2D, Size2D};
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{Key, KeyModifiers, KeyState, PipelineId, TopLevelBrowsingContextId};
use net_traits::image::base::Image;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{AnimationState, ConstellationMsg, EventResult, LoadData};
use servo_url::ServoUrl;
use std::fmt::{Debug, Error, Formatter};
use std::sync::mpsc::{Receiver, Sender};
use style_traits::cursor::CursorKind;
use style_traits::viewport::ViewportConstraints;
use webrender;
use webrender_api;


/// Used to wake up the event loop, provided by the servo port/embedder.
pub trait EventLoopWaker : 'static + Send {
    fn clone(&self) -> Box<EventLoopWaker + Send>;
    fn wake(&self);
}

/// Sends messages to the embedder.
pub struct EmbedderProxy {
    pub sender: Sender<EmbedderMsg>,
    pub event_loop_waker: Box<EventLoopWaker>,
}

impl EmbedderProxy {
    pub fn send(&self, msg: EmbedderMsg) {
        // Send a message and kick the OS event loop awake.
        if let Err(err) = self.sender.send(msg) {
            warn!("Failed to send response ({}).", err);
        }
        self.event_loop_waker.wake();
    }
}

impl Clone for EmbedderProxy {
    fn clone(&self) -> EmbedderProxy {
        EmbedderProxy {
            sender: self.sender.clone(),
            event_loop_waker: self.event_loop_waker.clone(),
        }
    }
}

/// The port that the embedder receives messages on.
pub struct EmbedderReceiver {
    pub receiver: Receiver<EmbedderMsg>
}

impl EmbedderReceiver {
    pub fn try_recv_embedder_msg(&mut self) -> Option<EmbedderMsg> {
        self.receiver.try_recv().ok()
    }
    pub fn recv_embedder_msg(&mut self) -> EmbedderMsg {
        self.receiver.recv().unwrap()
    }
}

/// Sends messages to the compositor.
pub struct CompositorProxy {
    pub sender: Sender<Msg>,
    pub event_loop_waker: Box<EventLoopWaker>,
}

impl CompositorProxy {
    pub fn send(&self, msg: Msg) {
        // Send a message and kick the OS event loop awake.
        if let Err(err) = self.sender.send(msg) {
            warn!("Failed to send response ({}).", err);
        }
        self.event_loop_waker.wake();
    }
}

impl Clone for CompositorProxy {
    fn clone(&self) -> CompositorProxy {
        CompositorProxy {
            sender: self.sender.clone(),
            event_loop_waker: self.event_loop_waker.clone(),
        }
    }
}

/// The port that the compositor receives messages on.
pub struct CompositorReceiver {
    pub receiver: Receiver<Msg>
}

impl CompositorReceiver {
    pub fn try_recv_compositor_msg(&mut self) -> Option<Msg> {
        self.receiver.try_recv().ok()
    }
    pub fn recv_compositor_msg(&mut self) -> Msg {
        self.receiver.recv().unwrap()
    }
}

impl CompositorProxy {
    pub fn recomposite(&self, reason: CompositingReason) {
        self.send(Msg::Recomposite(reason));
    }
}

pub enum EmbedderMsg {
    /// A status message to be displayed by the browser chrome.
    Status(TopLevelBrowsingContextId, Option<String>),
    /// Alerts the embedder that the current page has changed its title.
    ChangePageTitle(TopLevelBrowsingContextId, Option<String>),
    /// Move the window to a point
    MoveTo(TopLevelBrowsingContextId, Point2D<i32>),
    /// Resize the window to size
    ResizeTo(TopLevelBrowsingContextId, Size2D<u32>),
    /// Get Window Informations size and position
    GetClientWindow(TopLevelBrowsingContextId, IpcSender<(Size2D<u32>, Point2D<i32>)>),
    /// Get screen size (pixel)
    GetScreenSize(TopLevelBrowsingContextId, IpcSender<(Size2D<u32>)>),
    /// Get screen available size (pixel)
    GetScreenAvailSize(TopLevelBrowsingContextId, IpcSender<(Size2D<u32>)>),
    /// Wether or not to follow a link
    AllowNavigation(TopLevelBrowsingContextId, ServoUrl, IpcSender<bool>),
    /// Sends an unconsumed key event back to the embedder.
    KeyEvent(Option<TopLevelBrowsingContextId>, Option<char>, Key, KeyState, KeyModifiers),
    /// Changes the cursor.
    SetCursor(CursorKind),
    /// A favicon was detected
    NewFavicon(TopLevelBrowsingContextId, ServoUrl),
    /// <head> tag finished parsing
    HeadParsed(TopLevelBrowsingContextId),
    /// The history state has changed.
    HistoryChanged(TopLevelBrowsingContextId, Vec<LoadData>, usize),
    /// Enter or exit fullscreen
    SetFullscreenState(TopLevelBrowsingContextId, bool),
    /// The load of a page has begun
    LoadStart(TopLevelBrowsingContextId),
    /// The load of a page has completed
    LoadComplete(TopLevelBrowsingContextId),
}

/// Messages from the painting thread and the constellation thread to the compositor thread.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit,

    /// Informs the compositor that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make
    /// (e.g. SetFrameTree) at the time that we send it an ExitMsg.
    ShutdownComplete,

    /// Alerts the compositor that the given pipeline has changed whether it is running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Replaces the current frame tree, typically called during main frame navigation.
    SetFrameTree(SendableFrameTree),
    /// Composite.
    Recomposite(CompositingReason),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// Composite to a PNG file and return the Image over a passed channel.
    CreatePng(IpcSender<Option<Image>>),
    /// Alerts the compositor that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
    /// A reply to the compositor asking if the output image is stable.
    IsReadyToSaveImageReply(bool),
    /// Pipeline visibility changed
    PipelineVisibilityChanged(PipelineId, bool),
    /// WebRender has successfully processed a scroll. The boolean specifies whether a composite is
    /// needed.
    NewScrollFrameReady(bool),
    /// A pipeline was shut down.
    // This message acts as a synchronization point between the constellation,
    // when it shuts down a pipeline, to the compositor; when the compositor
    // sends a reply on the IpcSender, the constellation knows it's safe to
    // tear down the other threads associated with this pipeline.
    PipelineExited(PipelineId, IpcSender<()>),
    /// Runs a closure in the compositor thread.
    /// It's used to dispatch functions from webrender to the main thread's event loop.
    /// Required to allow WGL GLContext sharing in Windows.
    Dispatch(Box<Fn() + Send>),
    /// Indicates to the compositor that it needs to record the time when the frame with
    /// the given ID (epoch) is painted and report it to the layout thread of the given
    /// pipeline ID.
    PendingPaintMetric(PipelineId, Epoch),
    /// The load of a page has completed
    LoadComplete(TopLevelBrowsingContextId),

}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Msg::Exit => write!(f, "Exit"),
            Msg::ShutdownComplete => write!(f, "ShutdownComplete"),
            Msg::ChangeRunningAnimationsState(..) => write!(f, "ChangeRunningAnimationsState"),
            Msg::SetFrameTree(..) => write!(f, "SetFrameTree"),
            Msg::Recomposite(..) => write!(f, "Recomposite"),
            Msg::TouchEventProcessed(..) => write!(f, "TouchEventProcessed"),
            Msg::CreatePng(..) => write!(f, "CreatePng"),
            Msg::ViewportConstrained(..) => write!(f, "ViewportConstrained"),
            Msg::IsReadyToSaveImageReply(..) => write!(f, "IsReadyToSaveImageReply"),
            Msg::PipelineVisibilityChanged(..) => write!(f, "PipelineVisibilityChanged"),
            Msg::PipelineExited(..) => write!(f, "PipelineExited"),
            Msg::NewScrollFrameReady(..) => write!(f, "NewScrollFrameReady"),
            Msg::Dispatch(..) => write!(f, "Dispatch"),
            Msg::PendingPaintMetric(..) => write!(f, "PendingPaintMetric"),
            Msg::LoadComplete(..) => write!(f, "LoadComplete"),
        }
    }
}

impl Debug for EmbedderMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            EmbedderMsg::Status(..) => write!(f, "Status"),
            EmbedderMsg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            EmbedderMsg::MoveTo(..) => write!(f, "MoveTo"),
            EmbedderMsg::ResizeTo(..) => write!(f, "ResizeTo"),
            EmbedderMsg::GetClientWindow(..) => write!(f, "GetClientWindow"),
            EmbedderMsg::GetScreenSize(..) => write!(f, "GetScreenSize"),
            EmbedderMsg::GetScreenAvailSize(..) => write!(f, "GetScreenAvailSize"),
            EmbedderMsg::AllowNavigation(..) => write!(f, "AllowNavigation"),
            EmbedderMsg::KeyEvent(..) => write!(f, "KeyEvent"),
            EmbedderMsg::SetCursor(..) => write!(f, "SetCursor"),
            EmbedderMsg::NewFavicon(..) => write!(f, "NewFavicon"),
            EmbedderMsg::HeadParsed(..) => write!(f, "HeadParsed"),
            EmbedderMsg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            EmbedderMsg::SetFullscreenState(..) => write!(f, "SetFullscreenState"),
            EmbedderMsg::LoadStart(..) => write!(f, "LoadStart"),
            EmbedderMsg::LoadComplete(..) => write!(f, "LoadComplete"),
        }
    }
}

/// Data used to construct a compositor.
pub struct InitialCompositorState {
    /// A channel to the compositor.
    pub sender: CompositorProxy,
    /// A port on which messages inbound to the compositor can be received.
    pub receiver: CompositorReceiver,
    /// A channel to the constellation.
    pub constellation_chan: Sender<ConstellationMsg>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Instance of webrender API
    pub webrender: webrender::Renderer,
    pub webrender_document: webrender_api::DocumentId,
    pub webrender_api: webrender_api::RenderApi,
}
