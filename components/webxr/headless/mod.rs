/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::SurfmanGL;
use crate::SurfmanLayerManager;
use euclid::{Point2D, RigidTransform3D};
use std::sync::{Arc, Mutex};
use std::thread;
use surfman::chains::SwapChains;
use webxr_api::util::{self, ClipPlanes, HitTestList};
use webxr_api::{
    ApiSpace, BaseSpace, ContextId, DeviceAPI, DiscoveryAPI, Error, Event, EventBuffer, Floor,
    Frame, FrameUpdateEvent, HitTestId, HitTestResult, HitTestSource, Input, InputFrame, InputId,
    InputSource, LayerGrandManager, LayerId, LayerInit, LayerManager, MockButton, MockDeviceInit,
    MockDeviceMsg, MockDiscoveryAPI, MockInputMsg, MockViewInit, MockViewsInit, MockWorld, Native,
    Quitter, Ray, Receiver, SelectEvent, SelectKind, Sender, Session, SessionBuilder, SessionInit,
    SessionMode, Space, SubImages, View, Viewer, ViewerPose, Viewports, Views,
};

pub struct HeadlessMockDiscovery {}

struct HeadlessDiscovery {
    data: Arc<Mutex<HeadlessDeviceData>>,
    supports_vr: bool,
    supports_inline: bool,
    supports_ar: bool,
}

struct InputInfo {
    source: InputSource,
    active: bool,
    pointer: Option<RigidTransform3D<f32, Input, Native>>,
    grip: Option<RigidTransform3D<f32, Input, Native>>,
    clicking: bool,
    buttons: Vec<MockButton>,
}

struct HeadlessDevice {
    data: Arc<Mutex<HeadlessDeviceData>>,
    id: u32,
    hit_tests: HitTestList,
    granted_features: Vec<String>,
    grand_manager: LayerGrandManager<SurfmanGL>,
    layer_manager: Option<LayerManager>,
}

struct PerSessionData {
    id: u32,
    mode: SessionMode,
    clip_planes: ClipPlanes,
    quitter: Option<Quitter>,
    events: EventBuffer,
    needs_vp_update: bool,
}

struct HeadlessDeviceData {
    floor_transform: Option<RigidTransform3D<f32, Native, Floor>>,
    viewer_origin: Option<RigidTransform3D<f32, Viewer, Native>>,
    supported_features: Vec<String>,
    views: MockViewsInit,
    needs_floor_update: bool,
    inputs: Vec<InputInfo>,
    sessions: Vec<PerSessionData>,
    disconnected: bool,
    world: Option<MockWorld>,
    next_id: u32,
    bounds_geometry: Vec<Point2D<f32, Floor>>,
}

impl MockDiscoveryAPI<SurfmanGL> for HeadlessMockDiscovery {
    fn simulate_device_connection(
        &mut self,
        init: MockDeviceInit,
        receiver: Receiver<MockDeviceMsg>,
    ) -> Result<Box<dyn DiscoveryAPI<SurfmanGL>>, Error> {
        let viewer_origin = init.viewer_origin.clone();
        let floor_transform = init.floor_origin.map(|f| f.inverse());
        let views = init.views.clone();
        let data = HeadlessDeviceData {
            floor_transform,
            viewer_origin,
            supported_features: init.supported_features,
            views,
            needs_floor_update: false,
            inputs: vec![],
            sessions: vec![],
            disconnected: false,
            world: init.world,
            next_id: 0,
            bounds_geometry: vec![],
        };
        let data = Arc::new(Mutex::new(data));
        let data_ = data.clone();

        thread::spawn(move || {
            run_loop(receiver, data_);
        });
        Ok(Box::new(HeadlessDiscovery {
            data,
            supports_vr: init.supports_vr,
            supports_inline: init.supports_inline,
            supports_ar: init.supports_ar,
        }))
    }
}

fn run_loop(receiver: Receiver<MockDeviceMsg>, data: Arc<Mutex<HeadlessDeviceData>>) {
    while let Ok(msg) = receiver.recv() {
        if !data.lock().expect("Mutex poisoned").handle_msg(msg) {
            break;
        }
    }
}

impl DiscoveryAPI<SurfmanGL> for HeadlessDiscovery {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<SurfmanGL>,
    ) -> Result<Session, Error> {
        if !self.supports_session(mode) {
            return Err(Error::NoMatchingDevice);
        }
        let data = self.data.clone();
        let mut d = data.lock().unwrap();
        let id = d.next_id;
        d.next_id += 1;
        let per_session = PerSessionData {
            id,
            mode,
            clip_planes: Default::default(),
            quitter: Default::default(),
            events: Default::default(),
            needs_vp_update: false,
        };
        d.sessions.push(per_session);

        let granted_features = init.validate(mode, &d.supported_features)?;
        let layer_manager = None;
        drop(d);
        xr.spawn(move |grand_manager| {
            Ok(HeadlessDevice {
                data,
                id,
                granted_features,
                hit_tests: HitTestList::default(),
                grand_manager,
                layer_manager,
            })
        })
    }

    fn supports_session(&self, mode: SessionMode) -> bool {
        if self.data.lock().unwrap().disconnected {
            return false;
        }
        match mode {
            SessionMode::Inline => self.supports_inline,
            SessionMode::ImmersiveVR => self.supports_vr,
            SessionMode::ImmersiveAR => self.supports_ar,
        }
    }
}

fn view<Eye>(
    init: MockViewInit<Eye>,
    viewer: RigidTransform3D<f32, Viewer, Native>,
    clip_planes: ClipPlanes,
) -> View<Eye> {
    let projection = if let Some((l, r, t, b)) = init.fov {
        util::fov_to_projection_matrix(l, r, t, b, clip_planes)
    } else {
        init.projection
    };

    View {
        transform: init.transform.inverse().then(&viewer),
        projection,
    }
}

impl HeadlessDevice {
    fn with_per_session<R>(&self, f: impl FnOnce(&mut PerSessionData) -> R) -> R {
        f(self
            .data
            .lock()
            .unwrap()
            .sessions
            .iter_mut()
            .find(|s| s.id == self.id)
            .unwrap())
    }

    fn layer_manager(&mut self) -> Result<&mut LayerManager, Error> {
        if let Some(ref mut manager) = self.layer_manager {
            return Ok(manager);
        }
        let swap_chains = SwapChains::new();
        let viewports = self.viewports();
        let layer_manager = self.grand_manager.create_layer_manager(move |_, _| {
            Ok(SurfmanLayerManager::new(viewports, swap_chains))
        })?;
        self.layer_manager = Some(layer_manager);
        Ok(self.layer_manager.as_mut().unwrap())
    }
}

impl DeviceAPI for HeadlessDevice {
    fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>> {
        self.data.lock().unwrap().floor_transform.clone()
    }

    fn viewports(&self) -> Viewports {
        let d = self.data.lock().unwrap();
        let per_session = d.sessions.iter().find(|s| s.id == self.id).unwrap();
        d.viewports(per_session.mode)
    }

    fn create_layer(&mut self, context_id: ContextId, init: LayerInit) -> Result<LayerId, Error> {
        self.layer_manager()?.create_layer(context_id, init)
    }

    fn destroy_layer(&mut self, context_id: ContextId, layer_id: LayerId) {
        self.layer_manager()
            .unwrap()
            .destroy_layer(context_id, layer_id)
    }

    fn begin_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) -> Option<Frame> {
        let sub_images = self.layer_manager().ok()?.begin_frame(layers).ok()?;
        let mut data = self.data.lock().unwrap();
        let mut frame = data.get_frame(
            data.sessions.iter().find(|s| s.id == self.id).unwrap(),
            sub_images,
        );
        let per_session = data.sessions.iter_mut().find(|s| s.id == self.id).unwrap();
        if per_session.needs_vp_update {
            per_session.needs_vp_update = false;
            let mode = per_session.mode;
            let vp = data.viewports(mode);
            frame.events.push(FrameUpdateEvent::UpdateViewports(vp));
        }
        let events = self.hit_tests.commit_tests();
        frame.events = events;

        if let Some(ref world) = data.world {
            for source in self.hit_tests.tests() {
                let ray = data.native_ray(source.ray, source.space);
                let ray = if let Some(ray) = ray { ray } else { break };
                let hits = world
                    .regions
                    .iter()
                    .filter(|region| source.types.is_type(region.ty))
                    .flat_map(|region| &region.faces)
                    .filter_map(|triangle| triangle.intersect(ray))
                    .map(|space| HitTestResult {
                        space,
                        id: source.id,
                    });
                frame.hit_test_results.extend(hits);
            }
        }

        if data.needs_floor_update {
            frame.events.push(FrameUpdateEvent::UpdateFloorTransform(
                data.floor_transform.clone(),
            ));
            data.needs_floor_update = false;
        }
        Some(frame)
    }

    fn end_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) {
        let _ = self.layer_manager().unwrap().end_frame(layers);
        thread::sleep(std::time::Duration::from_millis(20));
    }

    fn initial_inputs(&self) -> Vec<InputSource> {
        vec![]
    }

    fn set_event_dest(&mut self, dest: Sender<Event>) {
        self.with_per_session(|s| s.events.upgrade(dest))
    }

    fn quit(&mut self) {
        self.with_per_session(|s| s.events.callback(Event::SessionEnd))
    }

    fn set_quitter(&mut self, quitter: Quitter) {
        self.with_per_session(|s| s.quitter = Some(quitter))
    }

    fn update_clip_planes(&mut self, near: f32, far: f32) {
        self.with_per_session(|s| s.clip_planes.update(near, far));
    }

    fn granted_features(&self) -> &[String] {
        &self.granted_features
    }

    fn request_hit_test(&mut self, source: HitTestSource) {
        self.hit_tests.request_hit_test(source)
    }

    fn cancel_hit_test(&mut self, id: HitTestId) {
        self.hit_tests.cancel_hit_test(id)
    }

    fn reference_space_bounds(&self) -> Option<Vec<Point2D<f32, Floor>>> {
        let bounds = self.data.lock().unwrap().bounds_geometry.clone();
        Some(bounds)
    }
}

impl HeadlessMockDiscovery {
    pub fn new() -> HeadlessMockDiscovery {
        HeadlessMockDiscovery {}
    }
}

macro_rules! with_all_sessions {
    ($self:ident, |$s:ident| $e:expr) => {
        for $s in &mut $self.sessions {
            $e;
        }
    };
}

impl HeadlessDeviceData {
    fn get_frame(&self, s: &PerSessionData, sub_images: Vec<SubImages>) -> Frame {
        let views = self.views.clone();

        let pose = self.viewer_origin.map(|transform| {
            let views = if s.mode == SessionMode::Inline {
                Views::Inline
            } else {
                match views {
                    MockViewsInit::Mono(one) => Views::Mono(view(one, transform, s.clip_planes)),
                    MockViewsInit::Stereo(one, two) => Views::Stereo(
                        view(one, transform, s.clip_planes),
                        view(two, transform, s.clip_planes),
                    ),
                }
            };

            ViewerPose { transform, views }
        });
        let inputs = self
            .inputs
            .iter()
            .filter(|i| i.active)
            .map(|i| InputFrame {
                id: i.source.id,
                target_ray_origin: i.pointer,
                grip_origin: i.grip,
                pressed: false,
                squeezed: false,
                hand: None,
                button_values: vec![],
                axis_values: vec![],
                input_changed: false,
            })
            .collect();
        Frame {
            pose,
            inputs,
            events: vec![],
            sub_images,
            hit_test_results: vec![],
            predicted_display_time: 0.0,
        }
    }

    fn viewports(&self, mode: SessionMode) -> Viewports {
        let vec = if mode == SessionMode::Inline {
            vec![]
        } else {
            match &self.views {
                MockViewsInit::Mono(one) => vec![one.viewport],
                MockViewsInit::Stereo(one, two) => vec![one.viewport, two.viewport],
            }
        };
        Viewports { viewports: vec }
    }

    fn trigger_select(&mut self, id: InputId, kind: SelectKind, event: SelectEvent) {
        for i in 0..self.sessions.len() {
            let frame = self.get_frame(&self.sessions[i], Vec::new());
            self.sessions[i]
                .events
                .callback(Event::Select(id, kind, event, frame));
        }
    }

    fn handle_msg(&mut self, msg: MockDeviceMsg) -> bool {
        match msg {
            MockDeviceMsg::SetWorld(w) => self.world = Some(w),
            MockDeviceMsg::ClearWorld => self.world = None,
            MockDeviceMsg::SetViewerOrigin(viewer_origin) => {
                self.viewer_origin = viewer_origin;
            }
            MockDeviceMsg::SetFloorOrigin(floor_origin) => {
                self.floor_transform = floor_origin.map(|f| f.inverse());
                self.needs_floor_update = true;
            }
            MockDeviceMsg::SetViews(views) => {
                self.views = views;
                with_all_sessions!(self, |s| {
                    s.needs_vp_update = true;
                })
            }
            MockDeviceMsg::VisibilityChange(v) => {
                with_all_sessions!(self, |s| s.events.callback(Event::VisibilityChange(v)))
            }
            MockDeviceMsg::AddInputSource(init) => {
                self.inputs.push(InputInfo {
                    source: init.source.clone(),
                    pointer: init.pointer_origin,
                    grip: init.grip_origin,
                    active: true,
                    clicking: false,
                    buttons: init.supported_buttons,
                });
                with_all_sessions!(self, |s| s
                    .events
                    .callback(Event::AddInput(init.source.clone())))
            }
            MockDeviceMsg::MessageInputSource(id, msg) => {
                if let Some(ref mut input) = self.inputs.iter_mut().find(|i| i.source.id == id) {
                    match msg {
                        MockInputMsg::SetHandedness(h) => {
                            input.source.handedness = h;
                            with_all_sessions!(self, |s| {
                                s.events
                                    .callback(Event::UpdateInput(id, input.source.clone()))
                            });
                        }
                        MockInputMsg::SetProfiles(p) => {
                            input.source.profiles = p;
                            with_all_sessions!(self, |s| {
                                s.events
                                    .callback(Event::UpdateInput(id, input.source.clone()))
                            });
                        }
                        MockInputMsg::SetTargetRayMode(t) => {
                            input.source.target_ray_mode = t;
                            with_all_sessions!(self, |s| {
                                s.events
                                    .callback(Event::UpdateInput(id, input.source.clone()))
                            });
                        }
                        MockInputMsg::SetPointerOrigin(p) => input.pointer = p,
                        MockInputMsg::SetGripOrigin(p) => input.grip = p,
                        MockInputMsg::TriggerSelect(kind, event) => {
                            if !input.active {
                                return true;
                            }
                            let clicking = input.clicking;
                            input.clicking = event == SelectEvent::Start;
                            match event {
                                SelectEvent::Start => {
                                    self.trigger_select(id, kind, event);
                                }
                                SelectEvent::End => {
                                    if clicking {
                                        self.trigger_select(id, kind, SelectEvent::Select);
                                    } else {
                                        self.trigger_select(id, kind, SelectEvent::End);
                                    }
                                }
                                SelectEvent::Select => {
                                    self.trigger_select(id, kind, SelectEvent::Start);
                                    self.trigger_select(id, kind, SelectEvent::Select);
                                }
                            }
                        }
                        MockInputMsg::Disconnect => {
                            if input.active {
                                with_all_sessions!(self, |s| s
                                    .events
                                    .callback(Event::RemoveInput(input.source.id)));
                                input.active = false;
                                input.clicking = false;
                            }
                        }
                        MockInputMsg::Reconnect => {
                            if !input.active {
                                with_all_sessions!(self, |s| s
                                    .events
                                    .callback(Event::AddInput(input.source.clone())));
                                input.active = true;
                            }
                        }
                        MockInputMsg::SetSupportedButtons(buttons) => {
                            input.buttons = buttons;
                            with_all_sessions!(self, |s| s.events.callback(Event::UpdateInput(
                                input.source.id,
                                input.source.clone()
                            )));
                        }
                        MockInputMsg::UpdateButtonState(state) => {
                            if let Some(button) = input
                                .buttons
                                .iter_mut()
                                .find(|b| b.button_type == state.button_type)
                            {
                                *button = state;
                            }
                        }
                    }
                }
            }
            MockDeviceMsg::Disconnect(s) => {
                self.disconnected = true;
                with_all_sessions!(self, |s| s.quitter.as_ref().map(|q| q.quit()));
                // notify the client that we're done disconnecting
                let _ = s.send(());
                return false;
            }
            MockDeviceMsg::SetBoundsGeometry(g) => {
                self.bounds_geometry = g;
            }
            MockDeviceMsg::SimulateResetPose => {
                with_all_sessions!(self, |s| s.events.callback(Event::ReferenceSpaceChanged(
                    BaseSpace::Local,
                    RigidTransform3D::identity()
                )));
            }
        }
        true
    }

    fn native_ray(&self, ray: Ray<ApiSpace>, space: Space) -> Option<Ray<Native>> {
        let origin: RigidTransform3D<f32, ApiSpace, Native> = match space.base {
            BaseSpace::Local => RigidTransform3D::identity(),
            BaseSpace::Floor => self.floor_transform?.inverse().cast_unit(),
            BaseSpace::Viewer => self.viewer_origin?.cast_unit(),
            BaseSpace::BoundedFloor => self.floor_transform?.inverse().cast_unit(),
            BaseSpace::TargetRay(id) => self
                .inputs
                .iter()
                .find(|i| i.source.id == id)?
                .pointer?
                .cast_unit(),
            BaseSpace::Grip(id) => self
                .inputs
                .iter()
                .find(|i| i.source.id == id)?
                .grip?
                .cast_unit(),
            BaseSpace::Joint(..) => panic!("Cannot request mocking backend with hands"),
        };
        let space_origin = space.offset.then(&origin);

        let origin_rigid: RigidTransform3D<f32, ApiSpace, ApiSpace> = ray.origin.into();
        Some(Ray {
            origin: origin_rigid.then(&space_origin).translation,
            direction: space_origin.rotation.transform_vector3d(ray.direction),
        })
    }
}
