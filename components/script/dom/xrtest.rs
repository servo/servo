/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRTestBinding::{
    self, FakeXRDeviceInit, XRTestMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::fakexrdevice::{get_origin, get_views, FakeXRDevice};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use dom_struct::dom_struct;
use std::cell::Cell;
use std::rc::Rc;
use webvr_traits::{MockVRInit, WebVRMsg};

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
}

impl XRTestMethods for XRTest {
    /// https://github.com/immersive-web/webxr-test-api/blob/master/explainer.md
    fn SimulateDeviceConnection(&self, init: &FakeXRDeviceInit) -> Rc<Promise> {
        let p = Promise::new(&self.global());

        if !init.supportsImmersive || self.session_started.get() {
            p.reject_native(&());
            return p;
        }

        let origin = match get_origin(&init.viewerOrigin) {
            Ok(origin) => origin,
            Err(e) => {
                p.reject_error(e);
                return p;
            },
        };

        let views = match get_views(&init.views) {
            Ok(views) => views,
            Err(e) => {
                p.reject_error(e);
                return p;
            },
        };

        let init = MockVRInit {
            viewer_origin: Some(origin),
            views: Some(views),
            eye_level: None,
        };

        self.session_started.set(true);
        self.global()
            .as_window()
            .webvr_thread()
            .unwrap()
            .send(WebVRMsg::CreateMockDisplay(init))
            .unwrap();
        p.resolve_native(&FakeXRDevice::new(&self.global()));

        p
    }
}
