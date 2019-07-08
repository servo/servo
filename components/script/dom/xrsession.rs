/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceBinding::XRReferenceSpaceType;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateInit;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XREnvironmentBlendMode;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRSessionMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutDom, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrlayer::XRLayer;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrenderstate::XRRenderState;
use crate::dom::xrspace::XRSpace;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use euclid::Vector3D;
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use std::cell::Cell;
use std::rc::Rc;
use webxr_api::{self, Frame, Session};

#[dom_struct]
pub struct XRSession {
    eventtarget: EventTarget,
    base_layer: MutNullableDom<XRLayer>,
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
        }
    }

    pub fn new(global: &GlobalScope, session: Session) -> DomRoot<XRSession> {
        let render_state = XRRenderState::new(global, 0.1, 1000.0, None);
        reflect_dom_object(
            Box::new(XRSession::new_inherited(session, &render_state)),
            global,
            XRSessionBinding::Wrap,
        )
    }

    pub fn set_layer(&self, layer: &XRLayer) {
        self.base_layer.set(Some(layer))
    }

    pub fn left_eye_params_offset(&self) -> Vector3D<f64> {
        unimplemented!()
    }

    pub fn right_eye_params_offset(&self) -> Vector3D<f64> {
        unimplemented!()
    }

    fn raf_callback(&self, (time, frame): (f64, Frame)) {
        unimplemented!()
    }
}

impl XRSessionMethods for XRSession {
    /// https://immersive-web.github.io/webxr/#dom-xrsession-mode
    fn Mode(&self) -> XRSessionMode {
        XRSessionMode::Immersive_vr
    }

    // https://immersive-web.github.io/webxr/#dom-xrsession-renderstate
    fn RenderState(&self) -> DomRoot<XRRenderState> {
        self.active_render_state.get()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrsession-updaterenderstate
    fn UpdateRenderState(&self, init: &XRRenderStateInit, _: InCompartment) {
        // XXXManishearth various checks:
        // If session’s ended value is true, throw an InvalidStateError and abort these steps
        // If newState’s baseLayer's was created with an XRSession other than session,
        // throw an InvalidStateError and abort these steps
        // If newState’s inlineVerticalFieldOfView is set and session is an
        // immersive session, throw an InvalidStateError and abort these steps.

        let pending = self
            .pending_render_state
            .or_init(|| self.active_render_state.get().copy());
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
        self.session.borrow_mut()
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
        unimplemented!()
    }
}
