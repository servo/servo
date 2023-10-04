/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

mod constellation_msg;

use std::fmt::{Debug, Error, Formatter};

use canvas::canvas_paint_thread::ImageUpdate;
pub use constellation_msg::ConstellationMsg;
use crossbeam_channel::{Receiver, Sender};
use embedder_traits::EventLoopWaker;
use euclid::Rect;
use gfx_traits::Epoch;
use ipc_channel::ipc::IpcSender;
use log::warn;
use msg::constellation_msg::{PipelineId, TopLevelBrowsingContextId};
use net_traits::image::base::Image;
use net_traits::NetToCompositorMsg;
use script_traits::{
    AnimationState, ConstellationControlMsg, EventResult, LayoutControlMsg, MouseButton,
    MouseEventType, ScriptToCompositorMsg,
};
use style_traits::CSSPixel;
use webrender_api::units::{DeviceIntPoint, DeviceIntSize};
use webrender_api::{self, FontInstanceKey, FontKey, ImageKey};

/// Why we performed a composite. This is used for debugging.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompositingReason {
    /// We hit the delayed composition timeout. (See `delayed_composition.rs`.)
    DelayedCompositeTimeout,
    /// The window has been scrolled and we're starting the first recomposite.
    Scroll,
    /// A scroll has continued and we need to recomposite again.
    ContinueScroll,
    /// We're performing the single composite in headless mode.
    Headless,
    /// We're performing a composite to run an animation.
    Animation,
    /// A new frame tree has been loaded.
    NewFrameTree,
    /// New painted buffers have been received.
    NewPaintedBuffers,
    /// The window has been zoomed.
    Zoom,
    /// A new WebRender frame has arrived.
    NewWebRenderFrame,
    /// WebRender has processed a scroll event and has generated a new frame.
    NewWebRenderScrollFrame,
    /// The window has been resized and will need to be synchronously repainted.
    Resize,
}

/// Sends messages to the compositor.
pub struct CompositorProxy {
    pub sender: Sender<CompositorMsg>,
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

impl CompositorProxy {
    pub fn recomposite(&self, reason: CompositingReason) {
        self.send(CompositorMsg::Recomposite(reason));
    }
}

/// Messages from (or via) the constellation thread to the compositor.
pub enum CompositorMsg {
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
    CreatePng(Option<Rect<f32, CSSPixel>>, IpcSender<Option<Image>>),
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
    Dispatch(Box<dyn Fn() + Send>),
    /// Indicates to the compositor that it needs to record the time when the frame with
    /// the given ID (epoch) is painted and report it to the layout thread of the given
    /// pipeline ID.
    PendingPaintMetric(PipelineId, Epoch),
    /// The load of a page has completed
    LoadComplete(TopLevelBrowsingContextId),
    /// WebDriver mouse button event
    WebDriverMouseButtonEvent(MouseEventType, MouseButton, f32, f32),
    /// WebDriver mouse move event
    WebDriverMouseMoveEvent(f32, f32),

    /// Get Window Informations size and position.
    GetClientWindow(IpcSender<(DeviceIntSize, DeviceIntPoint)>),
    /// Get screen size.
    GetScreenSize(IpcSender<DeviceIntSize>),
    /// Get screen available size.
    GetScreenAvailSize(IpcSender<DeviceIntSize>),

    /// Messages forwarded to the compositor by the constellation from other crates. These
    /// messages are mainly passed on from the compositor to WebRender.
    Forwarded(ForwardedToCompositorMsg),
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
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub layout_chan: IpcSender<LayoutControlMsg>,
}

pub enum FontToCompositorMsg {
    AddFontInstance(FontKey, f32, Sender<FontInstanceKey>),
    AddFont(gfx_traits::FontData, Sender<FontKey>),
}

pub enum CanvasToCompositorMsg {
    GenerateKey(Sender<ImageKey>),
    UpdateImages(Vec<ImageUpdate>),
}

/// Messages forwarded by the Constellation to the Compositor.
pub enum ForwardedToCompositorMsg {
    Layout(ScriptToCompositorMsg),
    Net(NetToCompositorMsg),
    Font(FontToCompositorMsg),
    Canvas(CanvasToCompositorMsg),
}

impl Debug for CompositorMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            CompositorMsg::ShutdownComplete => write!(f, "ShutdownComplete"),
            CompositorMsg::ChangeRunningAnimationsState(_, state) => {
                write!(f, "ChangeRunningAnimationsState({:?})", state)
            },
            CompositorMsg::SetFrameTree(..) => write!(f, "SetFrameTree"),
            CompositorMsg::Recomposite(..) => write!(f, "Recomposite"),
            CompositorMsg::TouchEventProcessed(..) => write!(f, "TouchEventProcessed"),
            CompositorMsg::CreatePng(..) => write!(f, "CreatePng"),
            CompositorMsg::IsReadyToSaveImageReply(..) => write!(f, "IsReadyToSaveImageReply"),
            CompositorMsg::PipelineVisibilityChanged(..) => write!(f, "PipelineVisibilityChanged"),
            CompositorMsg::PipelineExited(..) => write!(f, "PipelineExited"),
            CompositorMsg::NewScrollFrameReady(..) => write!(f, "NewScrollFrameReady"),
            CompositorMsg::Dispatch(..) => write!(f, "Dispatch"),
            CompositorMsg::PendingPaintMetric(..) => write!(f, "PendingPaintMetric"),
            CompositorMsg::LoadComplete(..) => write!(f, "LoadComplete"),
            CompositorMsg::WebDriverMouseButtonEvent(..) => write!(f, "WebDriverMouseButtonEvent"),
            CompositorMsg::WebDriverMouseMoveEvent(..) => write!(f, "WebDriverMouseMoveEvent"),
            CompositorMsg::GetClientWindow(..) => write!(f, "GetClientWindow"),
            CompositorMsg::GetScreenSize(..) => write!(f, "GetScreenSize"),
            CompositorMsg::GetScreenAvailSize(..) => write!(f, "GetScreenAvailSize"),
            CompositorMsg::Forwarded(..) => write!(f, "Webrender"),
        }
    }
}
