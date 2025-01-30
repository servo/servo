/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::channel;
use crate::ContextId;
use crate::DeviceAPI;
use crate::Error;
use crate::Event;
use crate::Floor;
use crate::Frame;
use crate::FrameUpdateEvent;
use crate::HitTestId;
use crate::HitTestSource;
use crate::InputSource;
use crate::LayerGrandManager;
use crate::LayerId;
use crate::LayerInit;
use crate::Native;
use crate::Receiver;
use crate::Sender;
use crate::Viewport;
use crate::Viewports;

use euclid::Point2D;
use euclid::Rect;
use euclid::RigidTransform3D;
use euclid::Size2D;

use log::warn;

use std::thread;
use std::time::Duration;

#[cfg(feature = "ipc")]
use serde::{Deserialize, Serialize};

// How long to wait for an rAF.
static TIMEOUT: Duration = Duration::from_millis(5);

/// https://www.w3.org/TR/webxr/#xrsessionmode-enum
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum SessionMode {
    Inline,
    ImmersiveVR,
    ImmersiveAR,
}

/// https://immersive-web.github.io/webxr/#dictdef-xrsessioninit
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct SessionInit {
    pub required_features: Vec<String>,
    pub optional_features: Vec<String>,
    /// Secondary views are enabled with the `secondary-view` feature
    /// but for performance reasons we also ask users to enable this pref
    /// for now.
    pub first_person_observer_view: bool,
}

impl SessionInit {
    /// Helper function for validating a list of requested features against
    /// a list of supported features for a given mode
    pub fn validate(&self, mode: SessionMode, supported: &[String]) -> Result<Vec<String>, Error> {
        for f in &self.required_features {
            // viewer and local in immersive are granted by default
            // https://immersive-web.github.io/webxr/#default-features
            if f == "viewer" || (f == "local" && mode != SessionMode::Inline) {
                continue;
            }

            if !supported.contains(f) {
                return Err(Error::UnsupportedFeature(f.into()));
            }
        }
        let mut granted = self.required_features.clone();
        for f in &self.optional_features {
            if f == "viewer"
                || (f == "local" && mode != SessionMode::Inline)
                || supported.contains(f)
            {
                granted.push(f.clone());
            }
        }

        Ok(granted)
    }

    pub fn feature_requested(&self, f: &str) -> bool {
        self.required_features
            .iter()
            .chain(self.optional_features.iter())
            .find(|x| *x == f)
            .is_some()
    }
}

/// https://immersive-web.github.io/webxr-ar-module/#xrenvironmentblendmode-enum
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub enum EnvironmentBlendMode {
    Opaque,
    AlphaBlend,
    Additive,
}

// The messages that are sent from the content thread to the session thread.
#[derive(Debug)]
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
enum SessionMsg {
    CreateLayer(ContextId, LayerInit, Sender<Result<LayerId, Error>>),
    DestroyLayer(ContextId, LayerId),
    SetLayers(Vec<(ContextId, LayerId)>),
    SetEventDest(Sender<Event>),
    UpdateClipPlanes(/* near */ f32, /* far */ f32),
    StartRenderLoop,
    RenderAnimationFrame,
    RequestHitTest(HitTestSource),
    CancelHitTest(HitTestId),
    UpdateFrameRate(f32, Sender<f32>),
    Quit,
    GetBoundsGeometry(Sender<Option<Vec<Point2D<f32, Floor>>>>),
}

#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct Quitter {
    sender: Sender<SessionMsg>,
}

impl Quitter {
    pub fn quit(&self) {
        let _ = self.sender.send(SessionMsg::Quit);
    }
}

/// An object that represents an XR session.
/// This is owned by the content thread.
/// https://www.w3.org/TR/webxr/#xrsession-interface
#[cfg_attr(feature = "ipc", derive(Serialize, Deserialize))]
pub struct Session {
    floor_transform: Option<RigidTransform3D<f32, Native, Floor>>,
    viewports: Viewports,
    sender: Sender<SessionMsg>,
    environment_blend_mode: EnvironmentBlendMode,
    initial_inputs: Vec<InputSource>,
    granted_features: Vec<String>,
    id: SessionId,
    supported_frame_rates: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "ipc", derive(Deserialize, Serialize))]
pub struct SessionId(pub(crate) u32);

impl Session {
    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>> {
        self.floor_transform.clone()
    }

    pub fn reference_space_bounds(&self) -> Option<Vec<Point2D<f32, Floor>>> {
        let (sender, receiver) = channel().ok()?;
        let _ = self.sender.send(SessionMsg::GetBoundsGeometry(sender));
        receiver.recv().ok()?
    }

    pub fn initial_inputs(&self) -> &[InputSource] {
        &self.initial_inputs
    }

    pub fn environment_blend_mode(&self) -> EnvironmentBlendMode {
        self.environment_blend_mode
    }

    pub fn viewports(&self) -> &[Rect<i32, Viewport>] {
        &self.viewports.viewports
    }

    /// A resolution large enough to contain all the viewports.
    /// https://immersive-web.github.io/webxr/#recommended-webgl-framebuffer-resolution
    ///
    /// Returns None if the session is inline
    pub fn recommended_framebuffer_resolution(&self) -> Option<Size2D<i32, Viewport>> {
        self.viewports()
            .iter()
            .fold(None::<Rect<_, _>>, |acc, vp| {
                Some(acc.map(|a| a.union(vp)).unwrap_or(*vp))
            })
            .map(|rect| Size2D::new(rect.max_x(), rect.max_y()))
    }

    pub fn create_layer(&self, context_id: ContextId, init: LayerInit) -> Result<LayerId, Error> {
        let (sender, receiver) = channel().map_err(|_| Error::CommunicationError)?;
        let _ = self
            .sender
            .send(SessionMsg::CreateLayer(context_id, init, sender));
        receiver.recv().map_err(|_| Error::CommunicationError)?
    }

    /// Destroy a layer
    pub fn destroy_layer(&self, context_id: ContextId, layer_id: LayerId) {
        let _ = self
            .sender
            .send(SessionMsg::DestroyLayer(context_id, layer_id));
    }

    pub fn set_layers(&self, layers: Vec<(ContextId, LayerId)>) {
        let _ = self.sender.send(SessionMsg::SetLayers(layers));
    }

    pub fn start_render_loop(&mut self) {
        let _ = self.sender.send(SessionMsg::StartRenderLoop);
    }

    pub fn update_clip_planes(&mut self, near: f32, far: f32) {
        let _ = self.sender.send(SessionMsg::UpdateClipPlanes(near, far));
    }

    pub fn set_event_dest(&mut self, dest: Sender<Event>) {
        let _ = self.sender.send(SessionMsg::SetEventDest(dest));
    }

    pub fn render_animation_frame(&mut self) {
        let _ = self.sender.send(SessionMsg::RenderAnimationFrame);
    }

    pub fn end_session(&mut self) {
        let _ = self.sender.send(SessionMsg::Quit);
    }

    pub fn apply_event(&mut self, event: FrameUpdateEvent) {
        match event {
            FrameUpdateEvent::UpdateFloorTransform(floor) => self.floor_transform = floor,
            FrameUpdateEvent::UpdateViewports(vp) => self.viewports = vp,
            FrameUpdateEvent::HitTestSourceAdded(_) => (),
        }
    }

    pub fn granted_features(&self) -> &[String] {
        &self.granted_features
    }

    pub fn request_hit_test(&self, source: HitTestSource) {
        let _ = self.sender.send(SessionMsg::RequestHitTest(source));
    }

    pub fn cancel_hit_test(&self, id: HitTestId) {
        let _ = self.sender.send(SessionMsg::CancelHitTest(id));
    }

    pub fn update_frame_rate(&mut self, rate: f32, sender: Sender<f32>) {
        let _ = self.sender.send(SessionMsg::UpdateFrameRate(rate, sender));
    }

    pub fn supported_frame_rates(&self) -> &[f32] {
        &self.supported_frame_rates
    }
}

#[derive(PartialEq)]
enum RenderState {
    NotInRenderLoop,
    InRenderLoop,
    PendingQuit,
}

/// For devices that want to do their own thread management, the `SessionThread` type is exposed.
pub struct SessionThread<Device> {
    receiver: Receiver<SessionMsg>,
    sender: Sender<SessionMsg>,
    layers: Vec<(ContextId, LayerId)>,
    pending_layers: Option<Vec<(ContextId, LayerId)>>,
    frame_count: u64,
    frame_sender: Sender<Frame>,
    running: bool,
    device: Device,
    id: SessionId,
    render_state: RenderState,
}

impl<Device> SessionThread<Device>
where
    Device: DeviceAPI,
{
    pub fn new(
        mut device: Device,
        frame_sender: Sender<Frame>,
        id: SessionId,
    ) -> Result<Self, Error> {
        let (sender, receiver) = crate::channel().or(Err(Error::CommunicationError))?;
        device.set_quitter(Quitter {
            sender: sender.clone(),
        });
        let frame_count = 0;
        let running = true;
        let layers = Vec::new();
        let pending_layers = None;
        Ok(SessionThread {
            sender,
            receiver,
            device,
            layers,
            pending_layers,
            frame_count,
            frame_sender,
            running,
            id,
            render_state: RenderState::NotInRenderLoop,
        })
    }

    pub fn new_session(&mut self) -> Session {
        let floor_transform = self.device.floor_transform();
        let viewports = self.device.viewports();
        let sender = self.sender.clone();
        let initial_inputs = self.device.initial_inputs();
        let environment_blend_mode = self.device.environment_blend_mode();
        let granted_features = self.device.granted_features().into();
        let supported_frame_rates = self.device.supported_frame_rates();
        Session {
            floor_transform,
            viewports,
            sender,
            initial_inputs,
            environment_blend_mode,
            granted_features,
            id: self.id,
            supported_frame_rates,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Ok(msg) = self.receiver.recv() {
                if !self.handle_msg(msg) {
                    self.running = false;
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn handle_msg(&mut self, msg: SessionMsg) -> bool {
        log::debug!("processing {:?}", msg);
        match msg {
            SessionMsg::SetEventDest(dest) => {
                self.device.set_event_dest(dest);
            }
            SessionMsg::RequestHitTest(source) => {
                self.device.request_hit_test(source);
            }
            SessionMsg::CancelHitTest(id) => {
                self.device.cancel_hit_test(id);
            }
            SessionMsg::CreateLayer(context_id, layer_init, sender) => {
                let result = self.device.create_layer(context_id, layer_init);
                let _ = sender.send(result);
            }
            SessionMsg::DestroyLayer(context_id, layer_id) => {
                self.layers.retain(|&(_, other_id)| layer_id != other_id);
                self.device.destroy_layer(context_id, layer_id);
            }
            SessionMsg::SetLayers(layers) => {
                self.pending_layers = Some(layers);
            }
            SessionMsg::StartRenderLoop => {
                if let Some(layers) = self.pending_layers.take() {
                    self.layers = layers;
                }
                let frame = match self.device.begin_animation_frame(&self.layers[..]) {
                    Some(frame) => frame,
                    None => {
                        warn!("Device stopped providing frames, exiting");
                        return false;
                    }
                };
                self.render_state = RenderState::InRenderLoop;
                let _ = self.frame_sender.send(frame);
            }
            SessionMsg::UpdateClipPlanes(near, far) => self.device.update_clip_planes(near, far),
            SessionMsg::RenderAnimationFrame => {
                self.frame_count += 1;

                self.device.end_animation_frame(&self.layers[..]);

                if self.render_state == RenderState::PendingQuit {
                    self.quit();
                    return false;
                }

                if let Some(layers) = self.pending_layers.take() {
                    self.layers = layers;
                }
                #[allow(unused_mut)]
                let mut frame = match self.device.begin_animation_frame(&self.layers[..]) {
                    Some(frame) => frame,
                    None => {
                        warn!("Device stopped providing frames, exiting");
                        return false;
                    }
                };

                let _ = self.frame_sender.send(frame);
            }
            SessionMsg::UpdateFrameRate(rate, sender) => {
                let new_framerate = self.device.update_frame_rate(rate);
                let _ = sender.send(new_framerate);
            }
            SessionMsg::Quit => {
                if self.render_state == RenderState::NotInRenderLoop {
                    self.quit();
                    return false;
                } else {
                    self.render_state = RenderState::PendingQuit;
                }
            }
            SessionMsg::GetBoundsGeometry(sender) => {
                let bounds = self.device.reference_space_bounds();
                let _ = sender.send(bounds);
            }
        }
        true
    }

    fn quit(&mut self) {
        self.render_state = RenderState::NotInRenderLoop;
        self.device.quit();
    }
}

/// Devices that need to can run sessions on the main thread.
pub trait MainThreadSession: 'static {
    fn run_one_frame(&mut self);
    fn running(&self) -> bool;
}

impl<Device> MainThreadSession for SessionThread<Device>
where
    Device: DeviceAPI,
{
    fn run_one_frame(&mut self) {
        let frame_count = self.frame_count;
        while frame_count == self.frame_count && self.running {
            if let Ok(msg) = crate::recv_timeout(&self.receiver, TIMEOUT) {
                self.running = self.handle_msg(msg);
            } else {
                break;
            }
        }
    }

    fn running(&self) -> bool {
        self.running
    }
}

/// A type for building XR sessions
pub struct SessionBuilder<'a, GL> {
    sessions: &'a mut Vec<Box<dyn MainThreadSession>>,
    frame_sender: Sender<Frame>,
    layer_grand_manager: LayerGrandManager<GL>,
    id: SessionId,
}

impl<'a, GL: 'static> SessionBuilder<'a, GL> {
    pub fn id(&self) -> SessionId {
        self.id
    }

    pub(crate) fn new(
        sessions: &'a mut Vec<Box<dyn MainThreadSession>>,
        frame_sender: Sender<Frame>,
        layer_grand_manager: LayerGrandManager<GL>,
        id: SessionId,
    ) -> Self {
        SessionBuilder {
            sessions,
            frame_sender,
            layer_grand_manager,
            id,
        }
    }

    /// For devices which are happy to hand over thread management to webxr.
    pub fn spawn<Device, Factory>(self, factory: Factory) -> Result<Session, Error>
    where
        Factory: 'static + FnOnce(LayerGrandManager<GL>) -> Result<Device, Error> + Send,
        Device: DeviceAPI,
    {
        let (acks, ackr) = crate::channel().or(Err(Error::CommunicationError))?;
        let frame_sender = self.frame_sender;
        let layer_grand_manager = self.layer_grand_manager;
        let id = self.id;
        thread::spawn(move || {
            match factory(layer_grand_manager)
                .and_then(|device| SessionThread::new(device, frame_sender, id))
            {
                Ok(mut thread) => {
                    let session = thread.new_session();
                    let _ = acks.send(Ok(session));
                    thread.run();
                }
                Err(err) => {
                    let _ = acks.send(Err(err));
                }
            }
        });
        ackr.recv().unwrap_or(Err(Error::CommunicationError))
    }

    /// For devices that need to run on the main thread.
    pub fn run_on_main_thread<Device, Factory>(self, factory: Factory) -> Result<Session, Error>
    where
        Factory: 'static + FnOnce(LayerGrandManager<GL>) -> Result<Device, Error>,
        Device: DeviceAPI,
    {
        let device = factory(self.layer_grand_manager)?;
        let frame_sender = self.frame_sender;
        let mut session_thread = SessionThread::new(device, frame_sender, self.id)?;
        let session = session_thread.new_session();
        self.sessions.push(Box::new(session_thread));
        Ok(session)
    }
}
