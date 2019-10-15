/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateInit;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateMethods;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XREnvironmentBlendMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRVisibilityState;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutDom, MutNullableDom};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::Node;
use crate::dom::node::NodeDamage;
use crate::dom::promise::Promise;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrinputsourcearray::XRInputSourceArray;
use crate::dom::xrinputsourceevent::XRInputSourceEvent;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrenderstate::XRRenderState;
use crate::dom::xrsessionevent::XRSessionEvent;
use crate::dom::xrspace::XRSpace;
use crate::dom::xrwebgllayer::XRWebGLLayer;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use euclid::RigidTransform3D;
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use std::cell::Cell;
use std::mem;
use std::rc::Rc;
use webxr_api::{
    self, EnvironmentBlendMode, Event as XREvent, Frame, SelectEvent, Session, Visibility,
};

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    base_layer: MutNullableDom<XRWebGLLayer>,
    blend_mode: XREnvironmentBlendMode,
    visibility_state: Cell<XRVisibilityState>,
    viewer_space: MutNullableDom<XRSpace>,
    #[ignore_malloc_size_of = "defined in webxr"]
    session: DomRefCell<Session>,
    frame_requested: Cell<bool>,
    pending_render_state: MutNullableDom<XRRenderState>,
    active_render_state: MutDom<XRRenderState>,

    next_raf_id: Cell<i32>,
    #[ignore_malloc_size_of = "closures are hard"]
    raf_callback_list: DomRefCell<Vec<(i32, Option<Rc<XRFrameRequestCallback>>)>>,
    #[ignore_malloc_size_of = "defined in ipc-channel"]
    raf_sender: DomRefCell<Option<IpcSender<(f64, Frame)>>>,
    input_sources: Dom<XRInputSourceArray>,
    // Any promises from calling end()
    #[ignore_malloc_size_of = "promises are hard"]
    end_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// https://immersive-web.github.io/webxr/#ended
    ended: Cell<bool>,
    /// Opaque framebuffers need to know the session is "outside of a requestAnimationFrame"
    /// https://immersive-web.github.io/webxr/#opaque-framebuffer
    outside_raf: Cell<bool>,
}

impl XRSession {
    fn new_inherited(
        session: Session,
        render_state: &XRRenderState,
        input_sources: &XRInputSourceArray,
    ) -> XRSession {
        XRSession {
            eventtarget: EventTarget::new_inherited(),
            base_layer: Default::default(),
            blend_mode: session.environment_blend_mode().into(),
            visibility_state: Cell::new(XRVisibilityState::Visible),
            viewer_space: Default::default(),
            session: DomRefCell::new(session),
            frame_requested: Cell::new(false),
            pending_render_state: MutNullableDom::new(None),
            active_render_state: MutDom::new(render_state),

            next_raf_id: Cell::new(0),
            raf_callback_list: DomRefCell::new(vec![]),
            raf_sender: DomRefCell::new(None),
            input_sources: Dom::from_ref(input_sources),
            end_promises: DomRefCell::new(vec![]),
            ended: Cell::new(false),
            outside_raf: Cell::new(true),
        }
    }

    pub fn new(global: &GlobalScope, session: Session) -> DomRoot<XRSession> {
        let render_state = XRRenderState::new(global, 0.1, 1000.0, None);
        let input_sources = XRInputSourceArray::new(global);
        let ret = reflect_dom_object(
            Box::new(XRSession::new_inherited(
                session,
                &render_state,
                &input_sources,
            )),
            global,
            XRSessionBinding::Wrap,
        );
        input_sources.set_initial_inputs(&ret);
        ret.attach_event_handler();
        ret.setup_raf_loop();
        ret
    }

    pub fn with_session<R, F: FnOnce(&Session) -> R>(&self, with: F) -> R {
        let session = self.session.borrow();
        with(&session)
    }

    pub fn is_ended(&self) -> bool {
        self.ended.get()
    }

    fn setup_raf_loop(&self) {
        assert!(
            self.raf_sender.borrow().is_none(),
            "RAF loop already set up"
        );
        let this = Trusted::new(self);
        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        *self.raf_sender.borrow_mut() = Some(sender);
        ROUTER.add_route(
            receiver.to_opaque(),
            Box::new(move |message| {
                let this = this.clone();
                let _ = task_source.queue_with_canceller(
                    task!(xr_raf_callback: move || {
                        this.root().raf_callback(message.to().unwrap());
                    }),
                    &canceller,
                );
            }),
        );

        self.request_new_xr_frame();
    }

    /// Requests a new https://immersive-web.github.io/webxr/#xr-animation-frame
    ///
    /// This happens regardless of the presense of rAF callbacks
    fn request_new_xr_frame(&self) {
        let sender = self.raf_sender.borrow().clone().unwrap();
        self.session.borrow_mut().request_animation_frame(sender);
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
            XREvent::Select(input, kind, frame) => {
                // https://immersive-web.github.io/webxr/#primary-action
                let source = self.input_sources.find(input);
                if let Some(source) = source {
                    let frame = XRFrame::new(&self.global(), self, frame);
                    frame.set_active(true);
                    if kind == SelectEvent::Start {
                        let event = XRInputSourceEvent::new(
                            &self.global(),
                            atom!("selectstart"),
                            false,
                            false,
                            &frame,
                            &source,
                        );
                        event.upcast::<Event>().fire(self.upcast());
                    } else {
                        if kind == SelectEvent::Select {
                            let event = XRInputSourceEvent::new(
                                &self.global(),
                                atom!("select"),
                                false,
                                false,
                                &frame,
                                &source,
                            );
                            event.upcast::<Event>().fire(self.upcast());
                        }
                        let event = XRInputSourceEvent::new(
                            &self.global(),
                            atom!("selectend"),
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
            },
            _ => (), // XXXManishearth TBD
        }
    }

    /// https://immersive-web.github.io/webxr/#xr-animation-frame
    fn raf_callback(&self, (time, mut frame): (f64, Frame)) {
        debug!("WebXR RAF callback");

        // Step 1
        if let Some(pending) = self.pending_render_state.take() {
            // https://immersive-web.github.io/webxr/#apply-the-pending-render-state
            // (Steps 1-4 are implicit)
            // Step 5
            self.active_render_state.set(&pending);
            // Step 6-7: XXXManishearth handle inlineVerticalFieldOfView

            // XXXManishearth handle inline sessions and composition disabled flag
            let swap_chain_id = pending.GetBaseLayer().map(|layer| layer.swap_chain_id());
            self.session.borrow_mut().set_swap_chain(swap_chain_id);
        }

        for event in frame.events.drain(..) {
            self.session.borrow_mut().apply_event(event)
        }

        // Step 2
        let base_layer = match self.active_render_state.get().GetBaseLayer() {
            Some(layer) => layer,
            None => return,
        };

        // Step 3: XXXManishearth handle inline session

        // Step 4-5
        let mut callbacks = mem::replace(&mut *self.raf_callback_list.borrow_mut(), vec![]);

        let frame = XRFrame::new(&self.global(), self, frame);
        // Step 6,7
        frame.set_active(true);
        frame.set_animation_frame(true);

        // Step 8
        self.outside_raf.set(false);
        for (_, callback) in callbacks.drain(..) {
            if let Some(callback) = callback {
                let _ = callback.Call__(Finite::wrap(time), &frame, ExceptionHandling::Report);
            }
        }
        self.outside_raf.set(true);

        frame.set_active(false);
        base_layer.swap_buffers();
        self.session.borrow_mut().render_animation_frame();
        self.request_new_xr_frame();

        // If the canvas element is attached to the DOM, it is now dirty,
        // and we need to trigger a reflow.
        base_layer
            .Context()
            .Canvas()
            .upcast::<Node>()
            .dirty(NodeDamage::OtherNodeDamage);
    }
}

impl XRSessionMethods for XRSession {
    /// https://immersive-web.github.io/webxr/#eventdef-xrsession-end
    event_handler!(end, GetOnend, SetOnend);

    /// https://immersive-web.github.io/webxr/#eventdef-xrsession-select
    event_handler!(select, GetOnselect, SetOnselect);

    /// https://immersive-web.github.io/webxr/#eventdef-xrsession-selectstart
    event_handler!(selectstart, GetOnselectstart, SetOnselectstart);

    /// https://immersive-web.github.io/webxr/#eventdef-xrsession-selectend
    event_handler!(selectend, GetOnselectend, SetOnselectend);

    /// https://immersive-web.github.io/webxr/#eventdef-xrsession-visibilitychange
    event_handler!(
        visibilitychange,
        GetOnvisibilitychange,
        SetOnvisibilitychange
    );

    // https://immersive-web.github.io/webxr/#dom-xrsession-renderstate
    fn RenderState(&self) -> DomRoot<XRRenderState> {
        self.active_render_state.get()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-updaterenderstate
    fn UpdateRenderState(&self, init: &XRRenderStateInit, _: InCompartment) -> ErrorResult {
        // Step 2
        if self.ended.get() {
            return Err(Error::InvalidState);
        }
        // Step 3:
        if let Some(ref layer) = init.baseLayer {
            if Dom::from_ref(layer.session()) != Dom::from_ref(self) {
                return Err(Error::InvalidState);
            }
        }

        // XXXManishearth step 4:
        // If newState’s inlineVerticalFieldOfView is set and session is an
        // immersive session, throw an InvalidStateError and abort these steps.

        let pending = self
            .pending_render_state
            .or_init(|| self.active_render_state.get().clone_object());
        if let Some(near) = init.depthNear {
            pending.set_depth_near(*near);
        }
        if let Some(far) = init.depthFar {
            pending.set_depth_far(*far);
        }
        if let Some(ref layer) = init.baseLayer {
            pending.set_layer(Some(&layer))
        }

        if init.depthFar.is_some() || init.depthNear.is_some() {
            self.session
                .borrow_mut()
                .update_clip_planes(*pending.DepthNear() as f32, *pending.DepthFar() as f32);
        }
        // XXXManishearth handle inlineVerticalFieldOfView
        Ok(())
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<XRFrameRequestCallback>) -> i32 {
        // queue up RAF callback, obtain ID
        let raf_id = self.next_raf_id.get();
        self.next_raf_id.set(raf_id + 1);
        self.raf_callback_list
            .borrow_mut()
            .push((raf_id, Some(callback)));

        raf_id
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-cancelanimationframe
    fn CancelAnimationFrame(&self, frame: i32) {
        let mut list = self.raf_callback_list.borrow_mut();
        if let Some(pair) = list.iter_mut().find(|pair| pair.0 == frame) {
            pair.1 = None;
        }
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-environmentblendmode
    fn EnvironmentBlendMode(&self) -> XREnvironmentBlendMode {
        self.blend_mode
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-visibilitystate
    fn VisibilityState(&self) -> XRVisibilityState {
        self.visibility_state.get()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestreferencespace
    fn RequestReferenceSpace(&self, ty: XRReferenceSpaceType, comp: InCompartment) -> Rc<Promise> {
        let p = Promise::new_in_current_compartment(&self.global(), comp);

        // https://immersive-web.github.io/webxr/#create-a-reference-space

        // XXXManishearth reject based on session type
        // https://github.com/immersive-web/webxr/blob/master/spatial-tracking-explainer.md#practical-usage-guidelines

        match ty {
            XRReferenceSpaceType::Bounded_floor | XRReferenceSpaceType::Unbounded => {
                // XXXManishearth eventually support these
                p.reject_error(Error::NotSupported)
            },
            ty => {
                p.resolve_native(&XRReferenceSpace::new(&self.global(), self, ty));
            },
        }

        p
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-inputsources
    fn InputSources(&self) -> DomRoot<XRInputSourceArray> {
        DomRoot::from_ref(&*self.input_sources)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-end
    fn End(&self) -> Rc<Promise> {
        let global = self.global();
        let p = Promise::new(&global);
        self.end_promises.borrow_mut().push(p.clone());
        // This is duplicated in event_callback since this should
        // happen ASAP for end() but can happen later if the device
        // shuts itself down
        self.ended.set(true);
        global.as_window().Navigator().Xr().end_session(self);
        self.session.borrow_mut().end_session();
        p
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ApiSpace;
// The pose of an object in native-space. Should never be exposed.
pub type ApiPose = RigidTransform3D<f32, ApiSpace, webxr_api::Native>;
// The pose of the viewer in some api-space.
pub type ApiViewerPose = RigidTransform3D<f32, webxr_api::Viewer, ApiSpace>;
// A transform between objects in some API-space
pub type ApiRigidTransform = RigidTransform3D<f32, ApiSpace, ApiSpace>;

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
