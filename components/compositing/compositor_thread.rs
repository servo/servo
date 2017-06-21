/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

use SendableFrameTree;
use compositor::CompositingReason;
use euclid::{Point2D, Size2D};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{Key, KeyModifiers, KeyState, PipelineId};
use net_traits::image::base::Image;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{AnimationState, ConstellationMsg, EventResult, LoadData};
use servo_url::ServoUrl;
use std::fmt::{Debug, Error, Formatter};
use std::sync::mpsc::{Receiver, Sender};
use style_traits::cursor::Cursor;
use style_traits::viewport::ViewportConstraints;
use webrender;
use webrender_traits;


/// Used to wake up the event loop, provided by the servo port/embedder.
pub trait EventLoopWaker : 'static + Send {
    fn clone(&self) -> Box<EventLoopWaker + Send>;
    fn wake(&self);
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
    pub fn clone_compositor_proxy(&self) -> CompositorProxy {
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

pub trait RenderListener {
    fn recomposite(&mut self, reason: CompositingReason);
}

impl RenderListener for CompositorProxy {
    fn recomposite(&mut self, reason: CompositingReason) {
        self.send(Msg::Recomposite(reason));
    }
}

/// Messages from the painting thread and the constellation thread to the compositor thread.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit,

    /// Informs the compositor that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make
    /// (e.g. SetFrameTree) at the time that we send it an ExitMsg.
    ShutdownComplete,

    /// Scroll a page in a window
    ScrollFragmentPoint(webrender_traits::ClipId, Point2D<f32>, bool),
    /// Alerts the compositor that the current page has changed its title.
    ChangePageTitle(PipelineId, Option<String>),
    /// Alerts the compositor that the given pipeline has changed whether it is running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Replaces the current frame tree, typically called during main frame navigation.
    SetFrameTree(SendableFrameTree, IpcSender<()>),
    /// The load of a page has begun
    LoadStart,
    /// The load of a page has completed
    LoadComplete,
    /// The history state has changed.
    HistoryChanged(Vec<LoadData>, usize),
    /// Wether or not to follow a link
    AllowNavigation(ServoUrl, IpcSender<bool>),
    /// Composite.
    Recomposite(CompositingReason),
    /// Sends an unconsumed key event back to the compositor.
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// Changes the cursor.
    SetCursor(Cursor),
    /// Composite to a PNG file and return the Image over a passed channel.
    CreatePng(IpcSender<Option<Image>>),
    /// Alerts the compositor that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
    /// A reply to the compositor asking if the output image is stable.
    IsReadyToSaveImageReply(bool),
    /// A favicon was detected
    NewFavicon(ServoUrl),
    /// <head> tag finished parsing
    HeadParsed,
    /// A status message to be displayed by the browser chrome.
    Status(Option<String>),
    /// Get Window Informations size and position
    GetClientWindow(IpcSender<(Size2D<u32>, Point2D<i32>)>),
    /// Move the window to a point
    MoveTo(Point2D<i32>),
    /// Resize the window to size
    ResizeTo(Size2D<u32>),
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
    /// Enter or exit fullscreen
    SetFullscreenState(bool),
}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Msg::Exit => write!(f, "Exit"),
            Msg::ShutdownComplete => write!(f, "ShutdownComplete"),
            Msg::ScrollFragmentPoint(..) => write!(f, "ScrollFragmentPoint"),
            Msg::ChangeRunningAnimationsState(..) => write!(f, "ChangeRunningAnimationsState"),
            Msg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            Msg::SetFrameTree(..) => write!(f, "SetFrameTree"),
            Msg::LoadComplete => write!(f, "LoadComplete"),
            Msg::AllowNavigation(..) => write!(f, "AllowNavigation"),
            Msg::LoadStart => write!(f, "LoadStart"),
            Msg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            Msg::Recomposite(..) => write!(f, "Recomposite"),
            Msg::KeyEvent(..) => write!(f, "KeyEvent"),
            Msg::TouchEventProcessed(..) => write!(f, "TouchEventProcessed"),
            Msg::SetCursor(..) => write!(f, "SetCursor"),
            Msg::CreatePng(..) => write!(f, "CreatePng"),
            Msg::ViewportConstrained(..) => write!(f, "ViewportConstrained"),
            Msg::IsReadyToSaveImageReply(..) => write!(f, "IsReadyToSaveImageReply"),
            Msg::NewFavicon(..) => write!(f, "NewFavicon"),
            Msg::HeadParsed => write!(f, "HeadParsed"),
            Msg::Status(..) => write!(f, "Status"),
            Msg::GetClientWindow(..) => write!(f, "GetClientWindow"),
            Msg::MoveTo(..) => write!(f, "MoveTo"),
            Msg::ResizeTo(..) => write!(f, "ResizeTo"),
            Msg::PipelineVisibilityChanged(..) => write!(f, "PipelineVisibilityChanged"),
            Msg::PipelineExited(..) => write!(f, "PipelineExited"),
            Msg::NewScrollFrameReady(..) => write!(f, "NewScrollFrameReady"),
            Msg::Dispatch(..) => write!(f, "Dispatch"),
            Msg::SetFullscreenState(..) => write!(f, "SetFullscreenState"),
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
    pub webrender_api_sender: webrender_traits::RenderApiSender,
}
