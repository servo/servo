/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRTestBinding::{
    self, FakeXRDeviceInit, XRTestMethods,
};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::fakexrdevice::{get_origin, get_views, FakeXRDevice};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use euclid::TypedRigidTransform3D;
use ipc_channel::ipc::IpcSender;
use ipc_channel::router::ROUTER;
use profile_traits::ipc;
use std::cell::Cell;
use std::rc::Rc;
use webxr_api::{self, Error as XRError, MockDeviceInit, MockDeviceMsg};

#[dom_struct]
pub struct XRTest {
    reflector: Reflector,
    session_started: Cell<bool>,
}

impl XRTest {
    pub fn new_inherited() -> XRTest {
        XRTest {
            reflector: Reflector::new(),
            session_started: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XRTest> {
        reflect_dom_object(
            Box::new(XRTest::new_inherited()),
            global,
            XRTestBinding::Wrap,
        )
    }

    fn device_obtained(
        &self,
        response: Result<IpcSender<MockDeviceMsg>, XRError>,
        trusted: TrustedPromise,
    ) {
        let promise = trusted.root();
        if let Ok(sender) = response {
            let device = FakeXRDevice::new(&self.global(), sender);
            promise.resolve_native(&device);
        } else {
            promise.reject_native(&());
        }
    }
}

impl XRTestMethods for XRTest {
    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn SimulateDeviceConnection(&self, init: &FakeXRDeviceInit) -> Rc<Promise> {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct MockDevice {
            sender: IpcSender<Result<IpcSender<MockDeviceMsg>, XRError>>,
        }

        #[typetag::serde]
        impl webxr_api::MockDeviceCallback for MockDevice {
            fn callback(&mut self, result: Result<IpcSender<MockDeviceMsg>, XRError>) {
                self.sender
                    .send(result)
                    .expect("mock device callback failed");
            }
        }

        let p = Promise::new(&self.global());

        if !init.supportsImmersive || self.session_started.get() {
            p.reject_native(&());
            return p;
        }

        let origin = if let Some(ref o) = init.viewerOrigin {
            match get_origin(&o) {
                Ok(origin) => origin,
                Err(e) => {
                    p.reject_error(e);
                    return p;
                },
            }
        } else {
            TypedRigidTransform3D::identity()
        };

        let floor_origin = if let Some(ref o) = init.floorOrigin {
            match get_origin(&o) {
                Ok(origin) => origin,
                Err(e) => {
                    p.reject_error(e);
                    return p;
                },
            }
        } else {
            TypedRigidTransform3D::identity()
        };

        let views = match get_views(&init.views) {
            Ok(views) => views,
            Err(e) => {
                p.reject_error(e);
                return p;
            },
        };

        let init = MockDeviceInit {
            viewer_origin: origin,
            views,
            supports_immersive: init.supportsImmersive,
            supports_unbounded: init.supportsUnbounded,
            floor_origin,
        };

        self.session_started.set(true);

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
            .simulate_device_connection(init, MockDevice { sender });

        p
    }
}
