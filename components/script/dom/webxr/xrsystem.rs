/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use profile_traits::generic_callback::GenericCallback as ProfileGenericCallback;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::reflect_dom_object;
use servo_base::generic_channel::GenericCallback;
use servo_base::id::PipelineId;
use servo_config::pref;
use stylo_atoms::Atom;
use webxr_api::{Error as XRError, Session, SessionInit, SessionMode};

use crate::conversions::Convert;
use crate::dom::SimpleEventTask;
use crate::dom::bindings::codegen::Bindings::XRSystemBinding::{
    XRSessionInit, XRSessionMode, XRSystemMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::gamepad::Gamepad;
use crate::dom::promise::{Promise, RootedPromise};
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;
use crate::dom::xrtest::XRTest;
use crate::event_loop::script_thread::ScriptThread;

#[dom_struct]
pub(crate) struct XRSystem {
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

    pub(crate) fn new(cx: &mut JSContext, window: &Window) -> DomRoot<XRSystem> {
        let this = reflect_dom_object(
            cx,
            Box::new(XRSystem::new_inherited(window.pipeline_id())),
            window,
        );

        let task_source = window
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback = GenericCallback::new({
            let this: Trusted<EventTarget> = Trusted::new(this.upcast());
            move |_| {
                let this = this.clone();
                task_source.queue(SimpleEventTask {
                    target: this,
                    name: Atom::from("devicechange"),
                });
            }
        })
        .expect("Could not create XR devicechange callback");

        if let Some(mut registry) = window.webxr_registry() {
            registry.set_device_change_callback(window.pipeline_id(), callback);
        }

        this
    }

    pub(crate) fn pending_or_active_session(&self) -> bool {
        self.pending_immersive_session.get() || self.active_immersive_session.get().is_some()
    }

    pub(crate) fn set_pending(&self) {
        self.pending_immersive_session.set(true)
    }

    pub(crate) fn set_active_immersive_session(&self, session: &XRSession) {
        // XXXManishearth when we support non-immersive (inline) sessions we should
        // ensure they never reach these codepaths
        self.pending_immersive_session.set(false);
        self.active_immersive_session.set(Some(session))
    }

    /// <https://immersive-web.github.io/webxr/#ref-for-eventdef-xrsession-end>
    pub(crate) fn end_session(&self, session: &XRSession) {
        // Step 3
        if let Some(active) = self.active_immersive_session.get() &&
            Dom::from_ref(&*active) == Dom::from_ref(session)
        {
            self.active_immersive_session.set(None);
            // Dirty the canvas, since it has been skipping this step whilst in immersive
            // mode
            session.dirty_layers();
        }
        self.active_inline_sessions
            .borrow_mut()
            .retain(|sess| Dom::from_ref(&**sess) != Dom::from_ref(session));
    }
}

impl Convert<SessionMode> for XRSessionMode {
    fn convert(self) -> SessionMode {
        match self {
            XRSessionMode::Immersive_vr => SessionMode::ImmersiveVR,
            XRSessionMode::Immersive_ar => SessionMode::ImmersiveAR,
            XRSessionMode::Inline => SessionMode::Inline,
        }
    }
}

impl XRSystemMethods<crate::DomTypeHolder> for XRSystem {
    /// <https://immersive-web.github.io/webxr/#dom-xr-issessionsupported>
    fn IsSessionSupported(&self, cx: &mut CurrentRealm, mode: XRSessionMode) -> RootedPromise {
        // XXXManishearth this should select an XR device first
        let promise = Promise::new_in_realm(cx);

        // > 2. If mode is "inline", resolve promise with true and return it.
        if mode == XRSessionMode::Inline {
            promise.resolve_native(cx, &true);
            return promise;
        }

        let mut trusted = Some(TrustedPromise::from(&promise));
        let global = self.global();
        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();

        let callback = ProfileGenericCallback::new(move |message| {
            // router doesn't know this is only called once
            let trusted = if let Some(trusted) = trusted.take() {
                trusted
            } else {
                error!("supportsSession callback called twice!");
                return;
            };
            let message: Result<(), webxr_api::Error> = if let Ok(message) = message {
                message
            } else {
                error!("supportsSession callback given incorrect payload");
                return;
            };
            if let Ok(()) = message {
                task_source.queue(trusted.resolve_task(true));
            } else {
                task_source.queue(trusted.resolve_task(false));
            };
        })
        .expect("Could not create callback");

        if let Some(mut r) = global.as_window().webxr_registry() {
            r.supports_session(mode.convert(), callback);
        }

        promise
    }

    /// <https://immersive-web.github.io/webxr/#dom-xr-requestsession>
    fn RequestSession(
        &self,
        realm: &mut CurrentRealm,
        mode: XRSessionMode,
        init: &XRSessionInit,
    ) -> RootedPromise {
        let global = self.global();
        let window = global.as_window();
        let promise = Promise::new_in_realm(realm);

        if mode != XRSessionMode::Inline {
            if !ScriptThread::is_user_interacting() {
                if pref!(dom_webxr_unsafe_assume_user_intent) {
                    warn!(
                        "The dom.webxr.unsafe-assume-user-intent preference assumes user intent to enter WebXR."
                    );
                } else {
                    promise
                        .reject_error(realm, Error::Security(Some("Feature not supported".into())));
                    return promise;
                }
            }

            if self.pending_or_active_session() {
                promise.reject_error(realm, Error::InvalidState(None));
                return promise;
            }

            self.set_pending();
        }

        let mut required_features: Vec<_> = init
            .requiredFeatures
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|domstring| domstring.clone().into())
            .collect();
        let optional_features: Vec<_> = init
            .optionalFeatures
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|domstring| domstring.clone().into())
            .collect();

        if !required_features.contains(&"viewer".to_string()) {
            required_features.push("viewer".to_string());
        }

        if !required_features.contains(&"local".to_string()) && mode != XRSessionMode::Inline {
            required_features.push("local".to_string());
        }

        let init = SessionInit {
            required_features,
            optional_features,
            first_person_observer_view: pref!(dom_webxr_first_person_observer_view),
            painter_id: global.as_window().webview_id().into(),
        };

        let mut trusted = Some(TrustedPromise::from(&promise));
        let this = Trusted::new(self);
        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback = ProfileGenericCallback::new(move |message| {
            let Some(trusted) = trusted.take() else {
                error!("requestSession callback called more than once");
                return;
            };
            let this = this.clone();
            let message: Result<Session, webxr_api::Error> = if let Ok(message) = message {
                message
            } else {
                error!("requestSession callback given incorrect payload");
                return;
            };
            task_source.queue(task!(request_session: move |cx| {
                this.root().session_obtained(cx, message, &trusted.root(cx), mode);
            }));
        })
        .expect("Could not create callback");
        if let Some(mut r) = window.webxr_registry() {
            r.request_session(mode.convert(), init, callback);
        }
        promise
    }

    // <https://immersive-web.github.io/webxr/#dom-xrsystem-ondevicechange>
    event_handler!(devicechange, GetOndevicechange, SetOndevicechange);

    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn Test(&self, cx: &mut JSContext) -> DomRoot<XRTest> {
        self.test.or_init(|| XRTest::new(cx, &self.global()))
    }
}

impl XRSystem {
    fn session_obtained(
        &self,
        cx: &mut JSContext,
        response: Result<Session, XRError>,
        promise: &RootedPromise,
        mode: XRSessionMode,
    ) {
        let session = match response {
            Ok(session) => session,
            Err(e) => {
                warn!("Error requesting XR session: {:?}", e);
                if mode != XRSessionMode::Inline {
                    self.pending_immersive_session.set(false);
                }
                promise.reject_error(cx, Error::NotSupported(None));
                return;
            },
        };
        let session = XRSession::new(cx, self.global().as_window(), session, mode);
        if mode == XRSessionMode::Inline {
            self.active_inline_sessions
                .borrow_mut()
                .push(Dom::from_ref(&*session));
        } else {
            self.set_active_immersive_session(&session);
        }
        promise.resolve_native(cx, &session);
        // https://github.com/immersive-web/webxr/issues/961
        // This must be called _after_ the promise is resolved
        session.setup_initial_inputs();
    }

    // https://github.com/immersive-web/navigation/issues/10
    pub(crate) fn dispatch_sessionavailable(&self) {
        let xr = Trusted::new(self);
        self.global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_sessionavailable_event: move |cx| {
                // The sessionavailable event indicates user intent to enter an XR session
                let xr = xr.root();
                    let _guard = ScriptThread::user_interacting_guard();
                    xr.upcast::<EventTarget>().fire_bubbling_event(cx, atom!("sessionavailable"));
            }));
    }
}
