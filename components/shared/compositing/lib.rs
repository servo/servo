/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

mod constellation_msg;

use std::fmt::{Debug, Error, Formatter};

use base::id::{PipelineId, TopLevelBrowsingContextId};
use base::Epoch;
pub use constellation_msg::ConstellationMsg;
use crossbeam_channel::{Receiver, Sender};
use embedder_traits::{EventLoopWaker, MouseButton, MouseButtonAction};
use euclid::Rect;
use ipc_channel::ipc::IpcSender;
use log::warn;
use pixels::Image;
use script_traits::{AnimationState, EventResult, ScriptThreadMessage};
use style_traits::CSSPixel;
use webrender_api::DocumentId;
use webrender_traits::{CrossProcessCompositorApi, CrossProcessCompositorMessage};

/// Sends messages to the compositor.
#[derive(Clone)]
pub struct CompositorProxy {
    pub sender: Sender<CompositorMsg>,
    /// Access to [`Self::sender`] that is possible to send across an IPC
    /// channel. These messages are routed via the router thread to
    /// [`Self::sender`].
    pub cross_process_compositor_api: CrossProcessCompositorApi,
    pub event_loop_waker: Box<dyn EventLoopWaker>,
}

impl CompositorProxy {
    pub fn send(&self, msg: CompositorMsg) {
        if let Err(err) = self.sender.send(msg) {
            warn!("Failed to send response ({:?}).", err);
        }
        self.event_loop_waker.wake();
    }
}

/// The port that the compositor receives messages on.
pub struct CompositorReceiver {
    pub receiver: Receiver<CompositorMsg>,
}

impl CompositorReceiver {
    pub fn try_recv_compositor_msg(&mut self) -> Option<CompositorMsg> {
        self.receiver.try_recv().ok()
    }
    pub fn recv_compositor_msg(&mut self) -> CompositorMsg {
        self.receiver.recv().unwrap()
    }
}

/// Messages from (or via) the constellation thread to the compositor.
pub enum CompositorMsg {
    /// Alerts the compositor that the given pipeline has changed whether it is running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Create or update a webview, given its frame tree.
    CreateOrUpdateWebView(SendableFrameTree),
    /// Remove a webview.
    RemoveWebView(TopLevelBrowsingContextId),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// Composite to a PNG file and return the Image over a passed channel.
    CreatePng(Option<Rect<f32, CSSPixel>>, IpcSender<Option<Image>>),
    /// A reply to the compositor asking if the output image is stable.
    IsReadyToSaveImageReply(bool),
    /// Set whether to use less resources by stopping animations.
    SetThrottled(PipelineId, bool),
    /// WebRender has produced a new frame. This message informs the compositor that
    /// the frame is ready. It contains a bool to indicate if it needs to composite and the
    /// `DocumentId` of the new frame.
    NewWebRenderFrameReady(DocumentId, bool),
    /// A pipeline was shut down.
    // This message acts as a synchronization point between the constellation,
    // when it shuts down a pipeline, to the compositor; when the compositor
    // sends a reply on the IpcSender, the constellation knows it's safe to
    // tear down the other threads associated with this pipeline.
    PipelineExited(PipelineId, IpcSender<()>),
    /// Indicates to the compositor that it needs to record the time when the frame with
    /// the given ID (epoch) is painted and report it to the layout of the given
    /// pipeline ID.
    PendingPaintMetric(PipelineId, Epoch),
    /// The load of a page has completed
    LoadComplete(TopLevelBrowsingContextId),
    /// WebDriver mouse button event
    WebDriverMouseButtonEvent(MouseButtonAction, MouseButton, f32, f32),
    /// WebDriver mouse move event
    WebDriverMouseMoveEvent(f32, f32),

    /// Messages forwarded to the compositor by the constellation from other crates. These
    /// messages are mainly passed on from the compositor to WebRender.
    CrossProcess(CrossProcessCompositorMessage),
}

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub children: Vec<SendableFrameTree>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    pub script_chan: IpcSender<ScriptThreadMessage>,
}

impl Debug for CompositorMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            CompositorMsg::ChangeRunningAnimationsState(_, state) => {
                write!(f, "ChangeRunningAnimationsState({:?})", state)
            },
            CompositorMsg::CreateOrUpdateWebView(..) => write!(f, "CreateOrUpdateWebView"),
            CompositorMsg::RemoveWebView(..) => write!(f, "RemoveWebView"),
            CompositorMsg::TouchEventProcessed(..) => write!(f, "TouchEventProcessed"),
            CompositorMsg::CreatePng(..) => write!(f, "CreatePng"),
            CompositorMsg::IsReadyToSaveImageReply(..) => write!(f, "IsReadyToSaveImageReply"),
            CompositorMsg::SetThrottled(..) => write!(f, "SetThrottled"),
            CompositorMsg::PipelineExited(..) => write!(f, "PipelineExited"),
            CompositorMsg::NewWebRenderFrameReady(..) => write!(f, "NewWebRenderFrameReady"),
            CompositorMsg::PendingPaintMetric(..) => write!(f, "PendingPaintMetric"),
            CompositorMsg::LoadComplete(..) => write!(f, "LoadComplete"),
            CompositorMsg::WebDriverMouseButtonEvent(..) => write!(f, "WebDriverMouseButtonEvent"),
            CompositorMsg::WebDriverMouseMoveEvent(..) => write!(f, "WebDriverMouseMoveEvent"),
            CompositorMsg::CrossProcess(..) => write!(f, "CrossProcess"),
        }
    }
}
