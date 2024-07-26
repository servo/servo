/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::f64::consts::{FRAC_PI_2, PI};
use std::mem;
use std::rc::Rc;

use dom_struct::dom_struct;
use euclid::{RigidTransform3D, Transform3D, Vector3D};
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::router::ROUTER;
use metrics::ToMs;
use profile_traits::ipc;
use webxr_api::{
    self, util, ApiSpace, ContextId as WebXRContextId, Display, EntityTypes, EnvironmentBlendMode,
    Event as XREvent, Frame, FrameUpdateEvent, HitTestId, HitTestSource, Ray, SelectEvent,
    SelectKind, Session, SessionId, View, Viewer, Visibility,
};

use super::bindings::trace::HashMapTracedValues;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::Navigator_Binding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XRHitTestSourceBinding::{
    XRHitTestOptionsInit, XRHitTestTrackableType,
};
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::{
    XRRenderStateInit, XRRenderStateMethods,
};
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::{
    XREnvironmentBlendMode, XRFrameRequestCallback, XRSessionMethods, XRVisibilityState,
};
use crate::dom::bindings::codegen::Bindings::XRSystemBinding::XRSessionMode;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutDom, MutNullableDom};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::reduce_timing_resolution;
use crate::dom::promise::Promise;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrhittestsource::XRHitTestSource;
use crate::dom::xrinputsourcearray::XRInputSourceArray;
use crate::dom::xrinputsourceevent::XRInputSourceEvent;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrenderstate::XRRenderState;
use crate::dom::xrsessionevent::XRSessionEvent;
use crate::dom::xrspace::XRSpace;
use crate::realms::InRealm;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    blend_mode: XREnvironmentBlendMode,
    mode: XRSessionMode,
    visibility_state: Cell<XRVisibilityState>,
    viewer_space: MutNullableDom<XRSpace>,
    #[ignore_malloc_size_of = "defined in webxr"]
    #[no_trace]
    session: DomRefCell<Session>,
    frame_requested: Cell<bool>,
    pending_render_state: MutNullableDom<XRRenderState>,
    active_render_state: MutDom<XRRenderState>,
    /// Cached projection matrix for inline sessions
    #[no_trace]
    inline_projection_matrix: DomRefCell<Transform3D<f32, Viewer, Display>>,

    next_raf_id: Cell<i32>,
    #[ignore_malloc_size_of = "closures are hard"]
    raf_callback_list: DomRefCell<Vec<(i32, Option<Rc<XRFrameRequestCallback>>)>>,
    #[ignore_malloc_size_of = "closures are hard"]
    current_raf_callback_list: DomRefCell<Vec<(i32, Option<Rc<XRFrameRequestCallback>>)>>,
    input_sources: Dom<XRInputSourceArray>,
    // Any promises from calling end()
    #[ignore_malloc_size_of = "promises are hard"]
    end_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// <https://immersive-web.github.io/webxr/#ended>
    ended: Cell<bool>,
    #[ignore_malloc_size_of = "defined in webxr"]
    #[no_trace]
    next_hit_test_id: Cell<HitTestId>,
    #[ignore_malloc_size_of = "defined in webxr"]
    pending_hit_test_promises: DomRefCell<HashMapTracedValues<HitTestId, Rc<Promise>>>,
    /// Opaque framebuffers need to know the session is "outside of a requestAnimationFrame"
    /// <https://immersive-web.github.io/webxr/#opaque-framebuffer>
    outside_raf: Cell<bool>,
}

impl XRSession {
    fn new_inherited(
        session: Session,
        render_state: &XRRenderState,
        input_sources: &XRInputSourceArray,
        mode: XRSessionMode,
    ) -> XRSession {
        XRSession {
            eventtarget: EventTarget::new_inherited(),
            blend_mode: session.environment_blend_mode().into(),
            mode,
            visibility_state: Cell::new(XRVisibilityState::Visible),
            viewer_space: Default::default(),
            session: DomRefCell::new(session),
            frame_requested: Cell::new(false),
            pending_render_state: MutNullableDom::new(None),
            active_render_state: MutDom::new(render_state),
            inline_projection_matrix: Default::default(),

            next_raf_id: Cell::new(0),
            raf_callback_list: DomRefCell::new(vec![]),
            current_raf_callback_list: DomRefCell::new(vec![]),
            input_sources: Dom::from_ref(input_sources),
            end_promises: DomRefCell::new(vec![]),
            ended: Cell::new(false),
            next_hit_test_id: Cell::new(HitTestId(0)),
            pending_hit_test_promises: DomRefCell::new(HashMapTracedValues::new()),
            outside_raf: Cell::new(true),
        }
    }

    pub fn new(
        global: &GlobalScope,
        session: Session,
        mode: XRSessionMode,
        frame_receiver: IpcReceiver<Frame>,
    ) -> DomRoot<XRSession> {
        let ivfov = if mode == XRSessionMode::Inline {
            Some(FRAC_PI_2)
        } else {
            None
        };
        let render_state = XRRenderState::new(global, 0.1, 1000.0, ivfov, None, Vec::new());
        let input_sources = XRInputSourceArray::new(global);
        let ret = reflect_dom_object(
            Box::new(XRSession::new_inherited(
                session,
                &render_state,
                &input_sources,
                mode,
            )),
            global,
        );
        ret.attach_event_handler();
        ret.setup_raf_loop(frame_receiver);
        ret
    }

    pub fn with_session<R, F: FnOnce(&Session) -> R>(&self, with: F) -> R {
        let session = self.session.borrow();
        with(&session)
    }

    pub fn is_ended(&self) -> bool {
        self.ended.get()
    }

    pub fn is_immersive(&self) -> bool {
        self.mode != XRSessionMode::Inline
    }

    // https://immersive-web.github.io/layers/#feature-descriptor-layers
    pub fn has_layers_feature(&self) -> bool {
        // We do not support creating layers other than projection layers
        // https://github.com/servo/servo/issues/27493
        false
    }

    fn setup_raf_loop(&self, frame_receiver: IpcReceiver<Frame>) {
        let this = Trusted::new(self);
        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        ROUTER.add_route(
            frame_receiver.to_opaque(),
            Box::new(move |message| {
                let frame: Frame = message.to().unwrap();
                let time = time::precise_time_ns();
                let this = this.clone();
                let _ = task_source.queue_with_canceller(
                    task!(xr_raf_callback: move || {
                        this.root().raf_callback(frame, time);
                    }),
                    &canceller,
                );
            }),
        );

        self.session.borrow_mut().start_render_loop();
    }

    pub fn is_outside_raf(&self) -> bool {
        self.outside_raf.get()
    }

    fn attach_event_handler(&self) {
        let this = Trusted::new(self);
        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();

        ROUTER.add_route(
            receiver.to_opaque(),
            Box::new(move |message| {
                let this = this.clone();
                let _ = task_source.queue_with_canceller(
                    task!(xr_event_callback: move || {
                        this.root().event_callback(message.to().unwrap());
                    }),
                    &canceller,
                );
            }),
        );

        // request animation frame
        self.session.borrow_mut().set_event_dest(sender);
    }

    // Must be called after the promise for session creation is resolved
    // https://github.com/immersive-web/webxr/issues/961
    //
    // This enables content that assumes all input sources are accompanied
    // by an inputsourceschange event to work properly. Without
    pub fn setup_initial_inputs(&self) {
        let initial_inputs = self.session.borrow().initial_inputs().to_owned();

        if initial_inputs.is_empty() {
            // do not fire an empty event
            return;
        }

        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let this = Trusted::new(self);
        // Queue a task so that it runs after resolve()'s microtasks complete
        // so that content has a chance to attach a listener for inputsourceschange
        let _ = task_source.queue_with_canceller(
            task!(session_initial_inputs: move || {
                let this = this.root();
                this.input_sources.add_input_sources(&this, &initial_inputs);
            }),
            &canceller,
        );
    }

    fn event_callback(&self, event: XREvent) {
        match event {
            XREvent::SessionEnd => {
                // https://immersive-web.github.io/webxr/#shut-down-the-session
                // Step 2
                self.ended.set(true);
                // Step 3-4
                self.global().as_window().Navigator().Xr().end_session(self);
                // Step 5: We currently do not have any such promises
                // Step 6 is happening n the XR session
                // https://immersive-web.github.io/webxr/#dom-xrsession-end step 3
                for promise in self.end_promises.borrow_mut().drain(..) {
                    promise.resolve_native(&());
                }
                // Step 7
                let event = XRSessionEvent::new(&self.global(), atom!("end"), false, false, self);
                event.upcast::<Event>().fire(self.upcast());
            },
            XREvent::Select(input, kind, ty, frame) => {
                use servo_atoms::Atom;
                const START_ATOMS: [Atom; 2] = [atom!("selectstart"), atom!("squeezestart")];
                const EVENT_ATOMS: [Atom; 2] = [atom!("select"), atom!("squeeze")];
                const END_ATOMS: [Atom; 2] = [atom!("selectend"), atom!("squeezeend")];

                // https://immersive-web.github.io/webxr/#primary-action
                let source = self.input_sources.find(input);
                let atom_index = if kind == SelectKind::Squeeze { 1 } else { 0 };
                if let Some(source) = source {
                    let frame = XRFrame::new(&self.global(), self, frame);
                    frame.set_active(true);
                    if ty == SelectEvent::Start {
                        let event = XRInputSourceEvent::new(
                            &self.global(),
                            START_ATOMS[atom_index].clone(),
                            false,
                            false,
                            &frame,
                            &source,
                        );
                        event.upcast::<Event>().fire(self.upcast());
                    } else {
                        if ty == SelectEvent::Select {
                            let event = XRInputSourceEvent::new(
                                &self.global(),
                                EVENT_ATOMS[atom_index].clone(),
                                false,
                                false,
                                &frame,
                                &source,
                            );
                            event.upcast::<Event>().fire(self.upcast());
                        }
                        let event = XRInputSourceEvent::new(
                            &self.global(),
                            END_ATOMS[atom_index].clone(),
                            false,
                            false,
                            &frame,
                            &source,
                        );
                        event.upcast::<Event>().fire(self.upcast());
                    }
                    frame.set_active(false);
                }
            },
            XREvent::VisibilityChange(v) => {
                let v = match v {
                    Visibility::Visible => XRVisibilityState::Visible,
                    Visibility::VisibleBlurred => XRVisibilityState::Visible_blurred,
                    Visibility::Hidden => XRVisibilityState::Hidden,
                };
                self.visibility_state.set(v);
                let event = XRSessionEvent::new(
                    &self.global(),
                    atom!("visibilitychange"),
                    false,
                    false,
                    self,
                );
                event.upcast::<Event>().fire(self.upcast());
                // The page may be visible again, dirty the layers
                // This also wakes up the event loop if necessary
                self.dirty_layers();
            },
            XREvent::AddInput(info) => {
                self.input_sources.add_input_sources(self, &[info]);
            },
            XREvent::RemoveInput(id) => {
                self.input_sources.remove_input_source(self, id);
            },
            XREvent::UpdateInput(id, source) => {
                self.input_sources.add_remove_input_source(self, id, source);
            },
        }
    }

    /// <https://immersive-web.github.io/webxr/#xr-animation-frame>
    fn raf_callback(&self, mut frame: Frame, time: u64) {
        debug!("WebXR RAF callback {:?}", frame);

        // Step 1-2 happen in the xebxr device thread

        // Step 3
        if let Some(pending) = self.pending_render_state.take() {
            // https://immersive-web.github.io/webxr/#apply-the-pending-render-state
            // (Steps 1-4 are implicit)
            // Step 5
            self.active_render_state.set(&pending);
            // Step 6-7: XXXManishearth handle inlineVerticalFieldOfView

            if !self.is_immersive() {
                self.update_inline_projection_matrix()
            }
        }

        // TODO: how does this fit the webxr spec?
        for event in frame.events.drain(..) {
            self.handle_frame_event(event);
        }

        // Step 4
        // TODO: what should this check be?
        // This is checking that the new render state has the same
        // layers as the frame.
        // Related to https://github.com/immersive-web/webxr/issues/1051
        if !self
            .active_render_state
            .get()
            .has_sub_images(&frame.sub_images[..])
        {
            // If the frame has different layers than the render state,
            // we just return early, drawing a blank frame.
            // This can result in flickering when the render state is changed.
            // TODO: it would be better to not render anything until the next frame.
            warn!("Rendering blank XR frame");
            self.session.borrow_mut().render_animation_frame();
            return;
        }

        // Step 5: XXXManishearth handle inline session

        // Step 6-7
        {
            let mut current = self.current_raf_callback_list.borrow_mut();
            assert!(current.is_empty());
            mem::swap(&mut *self.raf_callback_list.borrow_mut(), &mut current);
        }
        let start = self.global().as_window().get_navigation_start();
        let time = reduce_timing_resolution((time - start).to_ms());

        let frame = XRFrame::new(&self.global(), self, frame);
        // Step 8-9
        frame.set_active(true);
        frame.set_animation_frame(true);

        // Step 10
        self.apply_frame_updates(&frame);

        // TODO: how does this fit with the webxr and xr layers specs?
        self.layers_begin_frame(&frame);

        // Step 11-12
        self.outside_raf.set(false);
        let len = self.current_raf_callback_list.borrow().len();
        for i in 0..len {
            let callback = self.current_raf_callback_list.borrow()[i].1.clone();
            if let Some(callback) = callback {
                let _ = callback.Call__(time, &frame, ExceptionHandling::Report);
            }
        }
        self.outside_raf.set(true);
        *self.current_raf_callback_list.borrow_mut() = vec![];

        // TODO: how does this fit with the webxr and xr layers specs?
        self.layers_end_frame(&frame);

        // Step 13
        frame.set_active(false);

        // TODO: how does this fit the webxr spec?
        self.session.borrow_mut().render_animation_frame();
    }

    fn update_inline_projection_matrix(&self) {
        debug_assert!(!self.is_immersive());
        let render_state = self.active_render_state.get();
        let size = if let Some(base) = render_state.GetBaseLayer() {
            base.size()
        } else {
            return;
        };
        let mut clip_planes = util::ClipPlanes::default();
        let near = *render_state.DepthNear() as f32;
        let far = *render_state.DepthFar() as f32;
        clip_planes.update(near, far);
        let top = *render_state
            .GetInlineVerticalFieldOfView()
            .expect("IVFOV should be non null for inline sessions") /
            2.;
        let top = near * top.tan() as f32;
        let bottom = top;
        let left = top * size.width as f32 / size.height as f32;
        let right = left;
        let matrix = util::frustum_to_projection_matrix(left, right, top, bottom, clip_planes);
        *self.inline_projection_matrix.borrow_mut() = matrix;
    }

    /// Constructs a View suitable for inline sessions using the inlineVerticalFieldOfView and canvas size
    pub fn inline_view(&self) -> View<Viewer> {
        debug_assert!(!self.is_immersive());
        View {
            // Inline views have no offset
            transform: RigidTransform3D::identity(),
            projection: *self.inline_projection_matrix.borrow(),
        }
    }

    pub fn session_id(&self) -> SessionId {
        self.session.borrow().id()
    }

    pub fn dirty_layers(&self) {
        if let Some(layer) = self.RenderState().GetBaseLayer() {
            layer.context().mark_as_dirty();
        }
    }

    // TODO: how does this align with the layers spec?
    fn layers_begin_frame(&self, frame: &XRFrame) {
        if let Some(layer) = self.active_render_state.get().GetBaseLayer() {
            layer.begin_frame(frame);
        }
        self.active_render_state.get().with_layers(|layers| {
            for layer in layers {
                layer.begin_frame(frame);
            }
        });
    }

    // TODO: how does this align with the layers spec?
    fn layers_end_frame(&self, frame: &XRFrame) {
        if let Some(layer) = self.active_render_state.get().GetBaseLayer() {
            layer.end_frame(frame);
        }
        self.active_render_state.get().with_layers(|layers| {
            for layer in layers {
                layer.end_frame(frame);
            }
        });
    }

    /// <https://immersive-web.github.io/webxr/#xrframe-apply-frame-updates>
    fn apply_frame_updates(&self, _frame: &XRFrame) {
        // TODO: add a comment about why this is empty right now!
    }

    fn handle_frame_event(&self, event: FrameUpdateEvent) {
        match event {
            FrameUpdateEvent::HitTestSourceAdded(id) => {
                if let Some(promise) = self.pending_hit_test_promises.borrow_mut().remove(&id) {
                    promise.resolve_native(&XRHitTestSource::new(&self.global(), id, self));
                } else {
                    warn!(
                        "received hit test add request for unknown hit test {:?}",
                        id
                    )
                }
            },
            _ => self.session.borrow_mut().apply_event(event),
        }
    }
}

impl XRSessionMethods for XRSession {
    // https://immersive-web.github.io/webxr/#eventdef-xrsession-end
    event_handler!(end, GetOnend, SetOnend);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-select
    event_handler!(select, GetOnselect, SetOnselect);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-selectstart
    event_handler!(selectstart, GetOnselectstart, SetOnselectstart);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-selectend
    event_handler!(selectend, GetOnselectend, SetOnselectend);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-squeeze
    event_handler!(squeeze, GetOnsqueeze, SetOnsqueeze);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-squeezestart
    event_handler!(squeezestart, GetOnsqueezestart, SetOnsqueezestart);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-squeezeend
    event_handler!(squeezeend, GetOnsqueezeend, SetOnsqueezeend);

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-visibilitychange
    event_handler!(
        visibilitychange,
        GetOnvisibilitychange,
        SetOnvisibilitychange
    );

    // https://immersive-web.github.io/webxr/#eventdef-xrsession-inputsourceschange
    event_handler!(
        inputsourceschange,
        GetOninputsourceschange,
        SetOninputsourceschange
    );

    // https://immersive-web.github.io/webxr/#dom-xrsession-renderstate
    fn RenderState(&self) -> DomRoot<XRRenderState> {
        self.active_render_state.get()
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-updaterenderstate>
    fn UpdateRenderState(&self, init: &XRRenderStateInit, _: InRealm) -> ErrorResult {
        // Step 2
        if self.ended.get() {
            return Err(Error::InvalidState);
        }
        // Step 3:
        if let Some(Some(ref layer)) = init.baseLayer {
            if Dom::from_ref(layer.session()) != Dom::from_ref(self) {
                return Err(Error::InvalidState);
            }
        }

        // Step 4:
        if init.inlineVerticalFieldOfView.is_some() && self.is_immersive() {
            return Err(Error::InvalidState);
        }

        // https://immersive-web.github.io/layers/#updaterenderstatechanges
        // Step 1.
        if init.baseLayer.is_some() {
            if self.has_layers_feature() {
                return Err(Error::NotSupported);
            }
            // https://github.com/immersive-web/layers/issues/189
            if init.layers.is_some() {
                return Err(Error::Type(String::from(
                    "Cannot set WebXR layers and baseLayer",
                )));
            }
        }

        if let Some(Some(ref layers)) = init.layers {
            // Step 2
            for layer in layers {
                let count = layers
                    .iter()
                    .filter(|other| other.layer_id() == layer.layer_id())
                    .count();
                if count > 1 {
                    return Err(Error::Type(String::from("Duplicate entry in WebXR layers")));
                }
            }

            // Step 3
            for layer in layers {
                if layer.session() != self {
                    return Err(Error::Type(String::from(
                        "Layer from different session in WebXR layers",
                    )));
                }
            }
        }

        // Step 4-5
        let pending = self
            .pending_render_state
            .or_init(|| self.active_render_state.get().clone_object());

        // Step 6
        if let Some(ref layers) = init.layers {
            let layers = layers.as_deref().unwrap_or_default();
            pending.set_base_layer(None);
            pending.set_layers(layers.iter().map(|x| &**x).collect());
            let layers = layers
                .iter()
                .filter_map(|layer| {
                    let context_id = WebXRContextId::from(layer.context_id());
                    let layer_id = layer.layer_id()?;
                    Some((context_id, layer_id))
                })
                .collect();
            self.session.borrow_mut().set_layers(layers);
        }

        // End of https://immersive-web.github.io/layers/#updaterenderstatechanges

        if let Some(near) = init.depthNear {
            let mut near = *near;
            // Step 8 from #apply-the-pending-render-state
            // this may need to be changed if backends wish to impose
            // further constraints
            if near < 0. {
                near = 0.;
            }
            pending.set_depth_near(near);
        }
        if let Some(far) = init.depthFar {
            // Step 9 from #apply-the-pending-render-state
            // this may need to be changed if backends wish to impose
            // further constraints
            // currently the maximum is infinity, so we do nothing
            pending.set_depth_far(*far);
        }
        if let Some(fov) = init.inlineVerticalFieldOfView {
            let mut fov = *fov;
            // Step 10 from #apply-the-pending-render-state
            // this may need to be changed if backends wish to impose
            // further constraints
            if fov < 0. {
                fov = 0.0001;
            } else if fov > PI {
                fov = PI - 0.0001;
            }
            pending.set_inline_vertical_fov(fov);
        }
        if let Some(ref layer) = init.baseLayer {
            pending.set_base_layer(layer.as_deref());
            pending.set_layers(Vec::new());
            let layers = layer
                .iter()
                .filter_map(|layer| {
                    let context_id = WebXRContextId::from(layer.context_id());
                    let layer_id = layer.layer_id()?;
                    Some((context_id, layer_id))
                })
                .collect();
            self.session.borrow_mut().set_layers(layers);
        }

        if init.depthFar.is_some() || init.depthNear.is_some() {
            self.session
                .borrow_mut()
                .update_clip_planes(*pending.DepthNear() as f32, *pending.DepthFar() as f32);
        }

        Ok(())
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe>
    fn RequestAnimationFrame(&self, callback: Rc<XRFrameRequestCallback>) -> i32 {
        // queue up RAF callback, obtain ID
        let raf_id = self.next_raf_id.get();
        self.next_raf_id.set(raf_id + 1);
        self.raf_callback_list
            .borrow_mut()
            .push((raf_id, Some(callback)));

        raf_id
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-cancelanimationframe>
    fn CancelAnimationFrame(&self, frame: i32) {
        let mut list = self.raf_callback_list.borrow_mut();
        if let Some(pair) = list.iter_mut().find(|pair| pair.0 == frame) {
            pair.1 = None;
        }

        let mut list = self.current_raf_callback_list.borrow_mut();
        if let Some(pair) = list.iter_mut().find(|pair| pair.0 == frame) {
            pair.1 = None;
        }
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-environmentblendmode>
    fn EnvironmentBlendMode(&self) -> XREnvironmentBlendMode {
        self.blend_mode
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-visibilitystate>
    fn VisibilityState(&self) -> XRVisibilityState {
        self.visibility_state.get()
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-requestreferencespace>
    fn RequestReferenceSpace(&self, ty: XRReferenceSpaceType, comp: InRealm) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp);

        // https://immersive-web.github.io/webxr/#create-a-reference-space

        // XXXManishearth reject based on session type
        // https://github.com/immersive-web/webxr/blob/master/spatial-tracking-explainer.md#practical-usage-guidelines

        match ty {
            XRReferenceSpaceType::Bounded_floor | XRReferenceSpaceType::Unbounded => {
                // XXXManishearth eventually support these
                p.reject_error(Error::NotSupported)
            },
            ty => {
                if ty != XRReferenceSpaceType::Viewer &&
                    (!self.is_immersive() || ty != XRReferenceSpaceType::Local)
                {
                    let s = ty.as_str();
                    if !self
                        .session
                        .borrow()
                        .granted_features()
                        .iter()
                        .any(|f| *f == s)
                    {
                        p.reject_error(Error::NotSupported);
                        return p;
                    }
                }
                p.resolve_native(&XRReferenceSpace::new(&self.global(), self, ty));
            },
        }
        p
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-inputsources>
    fn InputSources(&self) -> DomRoot<XRInputSourceArray> {
        DomRoot::from_ref(&*self.input_sources)
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrsession-end>
    fn End(&self) -> Rc<Promise> {
        let global = self.global();
        let p = Promise::new(&global);
        if self.ended.get() && self.end_promises.borrow().is_empty() {
            // If the session has completely ended and all end promises have been resolved,
            // don't queue up more end promises
            //
            // We need to check for end_promises being empty because `ended` is set
            // before everything has been completely shut down, and we do not want to
            // prematurely resolve the promise then
            //
            // However, if end_promises is empty, then all end() promises have already resolved,
            // so the session has completely shut down and we should not queue up more promises
            p.resolve_native(&());
            return p;
        }
        self.end_promises.borrow_mut().push(p.clone());
        // This is duplicated in event_callback since this should
        // happen ASAP for end() but can happen later if the device
        // shuts itself down
        self.ended.set(true);
        global.as_window().Navigator().Xr().end_session(self);
        self.session.borrow_mut().end_session();
        p
    }

    // https://immersive-web.github.io/hit-test/#dom-xrsession-requesthittestsource
    fn RequestHitTestSource(&self, options: &XRHitTestOptionsInit) -> Rc<Promise> {
        let p = Promise::new(&self.global());

        if !self
            .session
            .borrow()
            .granted_features()
            .iter()
            .any(|f| f == "hit-test")
        {
            p.reject_error(Error::NotSupported);
            return p;
        }

        let id = self.next_hit_test_id.get();
        self.next_hit_test_id.set(HitTestId(id.0 + 1));

        let space = options.space.space();
        let ray = if let Some(ref ray) = options.offsetRay {
            ray.ray()
        } else {
            Ray {
                origin: Vector3D::new(0., 0., 0.),
                direction: Vector3D::new(0., 0., -1.),
            }
        };

        let mut types = EntityTypes::default();

        if let Some(ref tys) = options.entityTypes {
            for ty in tys {
                match ty {
                    XRHitTestTrackableType::Point => types.point = true,
                    XRHitTestTrackableType::Plane => types.plane = true,
                    XRHitTestTrackableType::Mesh => types.mesh = true,
                }
            }
        } else {
            types.plane = true;
        }

        let source = HitTestSource {
            id,
            space,
            ray,
            types,
        };
        self.pending_hit_test_promises
            .borrow_mut()
            .insert(id, p.clone());

        self.session.borrow().request_hit_test(source);

        p
    }
}

// The pose of an object in native-space. Should never be exposed.
pub type ApiPose = RigidTransform3D<f32, ApiSpace, webxr_api::Native>;
// A transform between objects in some API-space
pub type ApiRigidTransform = RigidTransform3D<f32, ApiSpace, ApiSpace>;

#[derive(Clone, Copy)]
pub struct BaseSpace;

pub type BaseTransform = RigidTransform3D<f32, webxr_api::Native, BaseSpace>;

#[allow(unsafe_code)]
pub fn cast_transform<T, U, V, W>(
    transform: RigidTransform3D<f32, T, U>,
) -> RigidTransform3D<f32, V, W> {
    unsafe { mem::transmute(transform) }
}

impl From<EnvironmentBlendMode> for XREnvironmentBlendMode {
    fn from(x: EnvironmentBlendMode) -> Self {
        match x {
            EnvironmentBlendMode::Opaque => XREnvironmentBlendMode::Opaque,
            EnvironmentBlendMode::AlphaBlend => XREnvironmentBlendMode::Alpha_blend,
            EnvironmentBlendMode::Additive => XREnvironmentBlendMode::Additive,
        }
    }
}
