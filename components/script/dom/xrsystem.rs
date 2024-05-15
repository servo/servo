/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use base::id::PipelineId;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self as ipc_crate, IpcReceiver};
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use servo_config::pref;
use webxr_api::{Error as XRError, Frame, Session, SessionInit, SessionMode};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::XRSystemBinding::{
    XRSessionInit, XRSessionMode, XRSystemMethods,
};
use crate::dom::bindings::conversions::{ConversionResult, FromJSValConvertible};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::eventtarget::EventTarget;
use crate::dom::gamepad::Gamepad;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;
use crate::dom::xrtest::XRTest;
use crate::realms::InRealm;
use crate::script_thread::ScriptThread;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct XRSystem {
    eventtarget: EventTarget,
    gamepads: DomRefCell<Vec<Dom<Gamepad>>>,
    pending_immersive_session: Cell<bool>,
    active_immersive_session: MutNullableDom<XRSession>,
    active_inline_sessions: DomRefCell<Vec<Dom<XRSession>>>,
    test: MutNullableDom<XRTest>,
    #[no_trace]
    pipeline: PipelineId,
}

impl XRSystem {
    fn new_inherited(pipeline: PipelineId) -> XRSystem {
        XRSystem {
            eventtarget: EventTarget::new_inherited(),
            gamepads: DomRefCell::new(Vec::new()),
            pending_immersive_session: Cell::new(false),
            active_immersive_session: Default::default(),
            active_inline_sessions: DomRefCell::new(Vec::new()),
            test: Default::default(),
            pipeline,
        }
    }

    pub fn new(window: &Window) -> DomRoot<XRSystem> {
        reflect_dom_object(
            Box::new(XRSystem::new_inherited(window.pipeline_id())),
            window,
        )
    }

    pub fn pending_or_active_session(&self) -> bool {
        self.pending_immersive_session.get() || self.active_immersive_session.get().is_some()
    }

    pub fn set_pending(&self) {
        self.pending_immersive_session.set(true)
    }

    pub fn set_active_immersive_session(&self, session: &XRSession) {
        // XXXManishearth when we support non-immersive (inline) sessions we should
        // ensure they never reach these codepaths
        self.pending_immersive_session.set(false);
        self.active_immersive_session.set(Some(session))
    }

    /// <https://immersive-web.github.io/webxr/#ref-for-eventdef-xrsession-end>
    pub fn end_session(&self, session: &XRSession) {
        // Step 3
        if let Some(active) = self.active_immersive_session.get() {
            if Dom::from_ref(&*active) == Dom::from_ref(session) {
                self.active_immersive_session.set(None);
                // Dirty the canvas, since it has been skipping this step whilst in immersive
                // mode
                session.dirty_layers();
            }
        }
        self.active_inline_sessions
            .borrow_mut()
            .retain(|sess| Dom::from_ref(&**sess) != Dom::from_ref(session));
    }
}

impl From<XRSessionMode> for SessionMode {
    fn from(mode: XRSessionMode) -> SessionMode {
        match mode {
            XRSessionMode::Immersive_vr => SessionMode::ImmersiveVR,
            XRSessionMode::Immersive_ar => SessionMode::ImmersiveAR,
            XRSessionMode::Inline => SessionMode::Inline,
        }
    }
}

impl XRSystemMethods for XRSystem {
    /// <https://immersive-web.github.io/webxr/#dom-xr-issessionsupported>
    fn IsSessionSupported(&self, mode: XRSessionMode) -> Rc<Promise> {
        // XXXManishearth this should select an XR device first
        let promise = Promise::new(&self.global());
        let mut trusted = Some(TrustedPromise::new(promise.clone()));
        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        ROUTER.add_route(
            receiver.to_opaque(),
            Box::new(move |message| {
                // router doesn't know this is only called once
                let trusted = if let Some(trusted) = trusted.take() {
                    trusted
                } else {
                    error!("supportsSession callback called twice!");
                    return;
                };
                let message: Result<(), webxr_api::Error> = if let Ok(message) = message.to() {
                    message
                } else {
                    error!("supportsSession callback given incorrect payload");
                    return;
                };
                if let Ok(()) = message {
                    let _ =
                        task_source.queue_with_canceller(trusted.resolve_task(true), &canceller);
                } else {
                    let _ =
                        task_source.queue_with_canceller(trusted.resolve_task(false), &canceller);
                };
            }),
        );
        window
            .webxr_registry()
            .supports_session(mode.into(), sender);

        promise
    }

    /// <https://immersive-web.github.io/webxr/#dom-xr-requestsession>
    #[allow(unsafe_code)]
    fn RequestSession(
        &self,
        mode: XRSessionMode,
        init: RootedTraceableBox<XRSessionInit>,
        comp: InRealm,
    ) -> Rc<Promise> {
        let global = self.global();
        let window = global.as_window();
        let promise = Promise::new_in_current_realm(comp);

        if mode != XRSessionMode::Inline {
            if !ScriptThread::is_user_interacting() {
                if pref!(dom.webxr.unsafe_assume_user_intent) {
                    warn!("The dom.webxr.unsafe-assume-user-intent preference assumes user intent to enter WebXR.");
                } else {
                    promise.reject_error(Error::Security);
                    return promise;
                }
            }

            if self.pending_or_active_session() {
                promise.reject_error(Error::InvalidState);
                return promise;
            }

            self.set_pending();
        }

        let mut required_features = vec![];
        let mut optional_features = vec![];
        let cx = GlobalScope::get_cx();

        // We are supposed to include "viewer" and on immersive devices "local"
        // by default here, but this is handled directly in requestReferenceSpace()
        if let Some(ref r) = init.requiredFeatures {
            for feature in r {
                unsafe {
                    if let Ok(ConversionResult::Success(s)) =
                        String::from_jsval(*cx, feature.handle(), ())
                    {
                        required_features.push(s)
                    } else {
                        warn!("Unable to convert required feature to string");
                        if mode != XRSessionMode::Inline {
                            self.pending_immersive_session.set(false);
                        }
                        promise.reject_error(Error::NotSupported);
                        return promise;
                    }
                }
            }
        }

        if let Some(ref o) = init.optionalFeatures {
            for feature in o {
                unsafe {
                    if let Ok(ConversionResult::Success(s)) =
                        String::from_jsval(*cx, feature.handle(), ())
                    {
                        optional_features.push(s)
                    } else {
                        warn!("Unable to convert optional feature to string");
                    }
                }
            }
        }

        let init = SessionInit {
            required_features,
            optional_features,
            first_person_observer_view: pref!(dom.webxr.first_person_observer_view),
        };

        let mut trusted = Some(TrustedPromise::new(promise.clone()));
        let this = Trusted::new(self);
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        let (frame_sender, frame_receiver) = ipc_crate::channel().unwrap();
        let mut frame_receiver = Some(frame_receiver);
        ROUTER.add_route(
            receiver.to_opaque(),
            Box::new(move |message| {
                // router doesn't know this is only called once
                let trusted = trusted.take().unwrap();
                let this = this.clone();
                let frame_receiver = frame_receiver.take().unwrap();
                let message: Result<Session, webxr_api::Error> = if let Ok(message) = message.to() {
                    message
                } else {
                    error!("requestSession callback given incorrect payload");
                    return;
                };
                let _ = task_source.queue_with_canceller(
                    task!(request_session: move || {
                        this.root().session_obtained(message, trusted.root(), mode, frame_receiver);
                    }),
                    &canceller,
                );
            }),
        );
        window
            .webxr_registry()
            .request_session(mode.into(), init, sender, frame_sender);
        promise
    }

    // https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn Test(&self) -> DomRoot<XRTest> {
        self.test.or_init(|| XRTest::new(&self.global()))
    }
}

impl XRSystem {
    fn session_obtained(
        &self,
        response: Result<Session, XRError>,
        promise: Rc<Promise>,
        mode: XRSessionMode,
        frame_receiver: IpcReceiver<Frame>,
    ) {
        let session = match response {
            Ok(session) => session,
            Err(e) => {
                warn!("Error requesting XR session: {:?}", e);
                if mode != XRSessionMode::Inline {
                    self.pending_immersive_session.set(false);
                }
                promise.reject_error(Error::NotSupported);
                return;
            },
        };
        let session = XRSession::new(&self.global(), session, mode, frame_receiver);
        if mode == XRSessionMode::Inline {
            self.active_inline_sessions
                .borrow_mut()
                .push(Dom::from_ref(&*session));
        } else {
            self.set_active_immersive_session(&session);
        }
        promise.resolve_native(&session);
        // https://github.com/immersive-web/webxr/issues/961
        // This must be called _after_ the promise is resolved
        session.setup_initial_inputs();
    }

    // https://github.com/immersive-web/navigation/issues/10
    pub fn dispatch_sessionavailable(&self) {
        let xr = Trusted::new(self);
        let global = self.global();
        let window = global.as_window();
        window
            .task_manager()
            .dom_manipulation_task_source()
            .queue(
                task!(fire_sessionavailable_event: move || {
                    // The sessionavailable event indicates user intent to enter an XR session
                    let xr = xr.root();
                    let interacting = ScriptThread::is_user_interacting();
                    ScriptThread::set_user_interacting(true);
                    xr.upcast::<EventTarget>().fire_bubbling_event(atom!("sessionavailable"));
                    ScriptThread::set_user_interacting(interacting);
                }),
                window.upcast(),
            )
            .unwrap();
    }
}
