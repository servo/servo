/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::JSVal;
use js::realm::CurrentRealm;
use profile_traits::generic_callback::GenericCallback as ProfileGenericCallback;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::generic_channel::GenericSender;
use webxr_api::{self, Error as XRError, MockDeviceInit, MockDeviceMsg};

use crate::ScriptThread;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::XRSystemBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRTestBinding::{FakeXRDeviceInit, XRTestMethods};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::fakexrdevice::{FakeXRDevice, get_origin, get_views, get_world};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;

#[dom_struct]
pub(crate) struct XRTest {
    reflector: Reflector,
    devices_connected: DomRefCell<Vec<Dom<FakeXRDevice>>>,
}

impl XRTest {
    pub(crate) fn new_inherited() -> XRTest {
        XRTest {
            reflector: Reflector::new(),
            devices_connected: DomRefCell::new(vec![]),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<XRTest> {
        reflect_dom_object_with_cx(Box::new(XRTest::new_inherited()), global, cx)
    }

    fn device_obtained(
        &self,
        cx: &mut JSContext,
        response: Result<GenericSender<MockDeviceMsg>, XRError>,
        trusted: TrustedPromise,
    ) {
        let promise = trusted.root();
        if let Ok(sender) = response {
            let device = FakeXRDevice::new(cx, &self.global(), sender);
            self.devices_connected
                .borrow_mut()
                .push(Dom::from_ref(&device));
            promise.resolve_native_with_cx(cx, &device);
        } else {
            promise.reject_native_with_cx(cx, &());
        }
    }
}

impl XRTestMethods<crate::DomTypeHolder> for XRTest {
    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn SimulateDeviceConnection(
        &self,
        cx: &mut CurrentRealm,
        init: &FakeXRDeviceInit,
    ) -> Rc<Promise> {
        let p = Promise::new_in_realm(cx);

        let origin = if let Some(ref o) = init.viewerOrigin {
            match get_origin(o) {
                Ok(origin) => Some(origin),
                Err(e) => {
                    p.reject_error_with_cx(cx, e);
                    return p;
                },
            }
        } else {
            None
        };

        let floor_origin = if let Some(ref o) = init.floorOrigin {
            match get_origin(o) {
                Ok(origin) => Some(origin),
                Err(e) => {
                    p.reject_error_with_cx(cx, e);
                    return p;
                },
            }
        } else {
            None
        };

        let views = match get_views(&init.views) {
            Ok(views) => views,
            Err(e) => {
                p.reject_error_with_cx(cx, e);
                return p;
            },
        };

        let supported_features = if let Some(ref s) = init.supportedFeatures {
            s.iter().cloned().map(String::from).collect()
        } else {
            vec![]
        };

        let world = if let Some(ref w) = init.world {
            let w = match get_world(w) {
                Ok(w) => w,
                Err(e) => {
                    p.reject_error_with_cx(cx, e);
                    return p;
                },
            };
            Some(w)
        } else {
            None
        };

        let (mut supports_inline, mut supports_vr, mut supports_ar) = (false, false, false);

        if let Some(ref modes) = init.supportedModes {
            for mode in modes {
                match mode {
                    XRSessionMode::Immersive_vr => supports_vr = true,
                    XRSessionMode::Immersive_ar => supports_ar = true,
                    XRSessionMode::Inline => supports_inline = true,
                }
            }
        }

        let init = MockDeviceInit {
            viewer_origin: origin,
            views,
            supports_inline,
            supports_vr,
            supports_ar,
            floor_origin,
            supported_features,
            world,
        };

        let global = self.global();
        let this = Trusted::new(self);
        let mut trusted = Some(TrustedPromise::new(p.clone()));

        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();

        let callback =
            ProfileGenericCallback::new(global.time_profiler_chan().clone(), move |message| {
                let trusted = trusted
                    .take()
                    .expect("SimulateDeviceConnection callback called twice");
                let this = this.clone();
                let message =
                    message.expect("SimulateDeviceConnection callback given incorrect payload");

                task_source.queue(task!(request_session: move |cx| {
                    this.root().device_obtained(cx, message, trusted);
                }));
            })
            .expect("Could not create callback");
        if let Some(mut r) = global.as_window().webxr_registry() {
            r.simulate_device_connection(init, callback);
        }

        p
    }

    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn SimulateUserActivation(&self, cx: &mut JSContext, f: Rc<Function>) {
        let _guard = ScriptThread::user_interacting_guard();
        rooted!(&in(cx) let mut value: JSVal);
        let _ = f.Call__(cx, vec![], value.handle_mut(), ExceptionHandling::Rethrow);
    }

    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn DisconnectAllDevices(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        // XXXManishearth implement device disconnection and session ending
        let p = Promise::new_in_realm(cx);
        let mut devices = self.devices_connected.borrow_mut();
        if devices.is_empty() {
            p.resolve_native_with_cx(cx, &());
        } else {
            let mut len = devices.len();

            let rooted_devices: Vec<_> = devices.iter().map(|x| DomRoot::from_ref(&**x)).collect();
            devices.clear();

            let mut trusted = Some(TrustedPromise::new(p.clone()));
            let global = self.global();
            let task_source = global
                .task_manager()
                .dom_manipulation_task_source()
                .to_sendable();

            let callback =
                ProfileGenericCallback::new(global.time_profiler_chan().clone(), move |_| {
                    len -= 1;
                    if len == 0 {
                        let trusted = trusted
                            .take()
                            .expect("DisconnectAllDevices disconnected more devices than expected");
                        task_source.queue(trusted.resolve_task(()));
                    }
                })
                .expect("Could not create callback");

            for device in rooted_devices {
                device.disconnect(callback.clone());
            }
        };
        p
    }
}
