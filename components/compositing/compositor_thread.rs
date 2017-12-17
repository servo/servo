/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

use SendableFrameTree;
use compositor::CompositingReason;
use crossbeam_channel::{Receiver, Sender};
use embedder_traits::EventLoopWaker;
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{PipelineId, TopLevelBrowsingContextId};
use net_traits::image::base::Image;
use profile_traits::mem;
use profile_traits::time;
use script_traits::{AnimationState, ConstellationMsg, EventResult};
use std::fmt::{Debug, Error, Formatter};
use style_traits::viewport::ViewportConstraints;
use webrender;
use webrender_api::{self, DeviceIntPoint, DeviceUintSize};


/// Sends messages to the compositor.
pub struct CompositorProxy {
    pub sender: Sender<Msg>,
    pub event_loop_waker: Box<EventLoopWaker>,
}

impl CompositorProxy {
    pub fn send(&self, msg: Msg) {
        self.sender.send(msg);
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
        self.receiver.try_recv()
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

/// Messages from the painting thread and the constellation thread to the compositor thread.
pub enum Msg {
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

    /// Get Window Informations size and position.
    GetClientWindow(IpcSender<(DeviceUintSize, DeviceIntPoint)>),
    /// Get screen size.
    GetScreenSize(IpcSender<DeviceUintSize>),
    /// Get screen available size.
    GetScreenAvailSize(IpcSender<DeviceUintSize>),
}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
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
            Msg::GetClientWindow(..) => write!(f, "GetClientWindow"),
            Msg::GetScreenSize(..) => write!(f, "GetScreenSize"),
            Msg::GetScreenAvailSize(..) => write!(f, "GetScreenAvailSize"),
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
