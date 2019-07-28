/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateInit;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateMethods;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XREnvironmentBlendMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
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
use crate::dom::xrinputsource::XRInputSource;
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
use webxr_api::{self, Event as XREvent, Frame, Session};

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    base_layer: MutNullableDom<XRWebGLLayer>,
    blend_mode: XREnvironmentBlendMode,
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
    input_sources: DomRefCell<Vec<Dom<XRInputSource>>>,
    // Any promises from calling end()
    #[ignore_malloc_size_of = "promises are hard"]
    end_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// https://immersive-web.github.io/webxr/#ended
    ended: Cell<bool>,
}

impl XRSession {
    fn new_inherited(session: Session, render_state: &XRRenderState) -> XRSession {
        XRSession {
            eventtarget: EventTarget::new_inherited(),
            base_layer: Default::default(),
            // we don't yet support any AR devices
            blend_mode: XREnvironmentBlendMode::Opaque,
            viewer_space: Default::default(),
            session: DomRefCell::new(session),
            frame_requested: Cell::new(false),
            pending_render_state: MutNullableDom::new(None),
            active_render_state: MutDom::new(render_state),

            next_raf_id: Cell::new(0),
            raf_callback_list: DomRefCell::new(vec![]),
            raf_sender: DomRefCell::new(None),
            input_sources: DomRefCell::new(vec![]),
            end_promises: DomRefCell::new(vec![]),
            ended: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope, session: Session) -> DomRoot<XRSession> {
        let render_state = XRRenderState::new(global, 0.1, 1000.0, None);
        let ret = reflect_dom_object(
            Box::new(XRSession::new_inherited(session, &render_state)),
            global,
            XRSessionBinding::Wrap,
        );
        {
            let mut input_sources = ret.input_sources.borrow_mut();
            for info in ret.session.borrow().initial_inputs() {
                // XXXManishearth we should be able to listen for updates
                // to the input sources
                let input = XRInputSource::new(global, &ret, *info);
                input_sources.push(Dom::from_ref(&input));
            }
        }
        ret.attach_event_handler();
        ret
    }

    pub fn with_session<R, F: FnOnce(&Session) -> R>(&self, with: F) -> R {
        let session = self.session.borrow();
        with(&session)
    }

    pub fn is_ended(&self) -> bool {
        self.ended.get()
    }

    fn attach_event_handler(&self) {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct EventCallback {
            sender: IpcSender<XREvent>,
        }

        #[typetag::serde]
        impl webxr_api::EventCallback for EventCallback {
            fn callback(&mut self, event: XREvent) {
                let _ = self.sender.send(event);
            }
        }

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
        self.session
            .borrow_mut()
            .set_event_callback(EventCallback { sender });
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
            _ => (), // XXXManishearth TBD
        }
    }

    /// https://immersive-web.github.io/webxr/#xr-animation-frame
    fn raf_callback(&self, (time, frame): (f64, Frame)) {
        // Step 1
        if let Some(pending) = self.pending_render_state.take() {
            // https://immersive-web.github.io/webxr/#apply-the-pending-render-state
            // (Steps 1-4 are implicit)
            // Step 5
            self.active_render_state.set(&pending);
            // Step 6-7: XXXManishearth handle inlineVerticalFieldOfView

            // XXXManishearth handle inline sessions and composition disabled flag
            let layer = pending.GetBaseLayer();
            if let Some(layer) = layer {
                let mut session = self.session.borrow_mut();
                session.update_webgl_external_image_api(
                    layer.Context().webgl_sender().webxr_external_image_api(),
                );
            }
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
        for (_, callback) in callbacks.drain(..) {
            if let Some(callback) = callback {
                let _ = callback.Call__(Finite::wrap(time), &frame, ExceptionHandling::Report);
            }
        }

        frame.set_active(false);
        self.session.borrow_mut().render_animation_frame();

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

    /// https://immersive-web.github.io/webxr/#dom-xrsession-mode
    fn Mode(&self) -> XRSessionMode {
        XRSessionMode::Immersive_vr
    }

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
        // XXXManishearth handle inlineVerticalFieldOfView
        Ok(())
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<XRFrameRequestCallback>) -> i32 {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct FrameCallback {
            sender: IpcSender<(f64, Frame)>,
        }

        #[typetag::serde]
        impl webxr_api::FrameRequestCallback for FrameCallback {
            fn callback(&mut self, time: f64, frame: Frame) {
                let _ = self.sender.send((time, frame));
            }
        }

        // queue up RAF callback, obtain ID
        let raf_id = self.next_raf_id.get();
        self.next_raf_id.set(raf_id + 1);
        self.raf_callback_list
            .borrow_mut()
            .push((raf_id, Some(callback)));

        // set up listener for response, if necessary
        if self.raf_sender.borrow().is_none() {
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
        }
        let sender = self.raf_sender.borrow().clone().unwrap();

        // request animation frame
        self.session
            .borrow_mut()
            .request_animation_frame(FrameCallback { sender });

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

    /// https://immersive-web.github.io/webxr/#dom-xrsession-getinputsources
    fn GetInputSources(&self) -> Vec<DomRoot<XRInputSource>> {
        self.input_sources
            .borrow()
            .iter()
            .map(|x| DomRoot::from_ref(&**x))
            .collect()
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
