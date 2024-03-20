/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use webxr_api::{self, Error as XRError, MockDeviceInit, MockDeviceMsg};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::XRSystemBinding::XRSessionMode;
use crate::dom::bindings::codegen::Bindings::XRTestBinding::{FakeXRDeviceInit, XRTestMethods};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::fakexrdevice::{get_origin, get_views, get_world, FakeXRDevice};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_thread::ScriptThread;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct XRTest {
    reflector: Reflector,
    devices_connected: DomRefCell<Vec<Dom<FakeXRDevice>>>,
}

impl XRTest {
    pub fn new_inherited() -> XRTest {
        XRTest {
            reflector: Reflector::new(),
            devices_connected: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRTest> {
        reflect_dom_object(Box::new(XRTest::new_inherited()), global)
    }

    fn device_obtained(
        &self,
        response: Result<IpcSender<MockDeviceMsg>, XRError>,
        trusted: TrustedPromise,
    ) {
        let promise = trusted.root();
        if let Ok(sender) = response {
            let device = FakeXRDevice::new(&self.global(), sender);
            self.devices_connected
                .borrow_mut()
                .push(Dom::from_ref(&device));
            promise.resolve_native(&device);
        } else {
            promise.reject_native(&());
        }
    }
}

impl XRTestMethods for XRTest {
    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    #[allow(unsafe_code)]
    fn SimulateDeviceConnection(&self, init: &FakeXRDeviceInit) -> Rc<Promise> {
        let global = self.global();
        let p = Promise::new(&global);

        let origin = if let Some(ref o) = init.viewerOrigin {
            match get_origin(o) {
                Ok(origin) => Some(origin),
                Err(e) => {
                    p.reject_error(e);
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
                    p.reject_error(e);
                    return p;
                },
            }
        } else {
            None
        };

        let views = match get_views(&init.views) {
            Ok(views) => views,
            Err(e) => {
                p.reject_error(e);
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
                    p.reject_error(e);
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
        let window = global.as_window();
        let this = Trusted::new(self);
        let mut trusted = Some(TrustedPromise::new(p.clone()));

        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
        ROUTER.add_route(
            receiver.to_opaque(),
            Box::new(move |message| {
                let trusted = trusted
                    .take()
                    .expect("SimulateDeviceConnection callback called twice");
                let this = this.clone();
                let message = message
                    .to()
                    .expect("SimulateDeviceConnection callback given incorrect payload");

                let _ = task_source.queue_with_canceller(
                    task!(request_session: move || {
                        this.root().device_obtained(message, trusted);
                    }),
                    &canceller,
                );
            }),
        );
        window
            .webxr_registry()
            .simulate_device_connection(init, sender);

        p
    }

    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn SimulateUserActivation(&self, f: Rc<Function>) {
        ScriptThread::set_user_interacting(true);
        let _ = f.Call__(vec![], ExceptionHandling::Rethrow);
        ScriptThread::set_user_interacting(false);
    }

    /// <https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md>
    fn DisconnectAllDevices(&self) -> Rc<Promise> {
        // XXXManishearth implement device disconnection and session ending
        let global = self.global();
        let p = Promise::new(&global);
        let mut devices = self.devices_connected.borrow_mut();
        if devices.is_empty() {
            p.resolve_native(&());
        } else {
            let mut len = devices.len();

            let (sender, receiver) = ipc::channel(global.time_profiler_chan().clone()).unwrap();
            let mut rooted_devices: Vec<_> =
                devices.iter().map(|x| DomRoot::from_ref(&**x)).collect();
            devices.clear();

            let mut trusted = Some(TrustedPromise::new(p.clone()));
            let (task_source, canceller) = global
                .as_window()
                .task_manager()
                .dom_manipulation_task_source_with_canceller();

            ROUTER.add_route(
                receiver.to_opaque(),
                Box::new(move |_| {
                    len -= 1;
                    if len == 0 {
                        let trusted = trusted
                            .take()
                            .expect("DisconnectAllDevices disconnected more devices than expected");
                        let _ =
                            task_source.queue_with_canceller(trusted.resolve_task(()), &canceller);
                    }
                }),
            );

            for device in rooted_devices.drain(..) {
                device.disconnect(sender.clone());
            }
        };
        p
    }
}
