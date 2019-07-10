/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding;
use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding::{
    XRHandedness, XRInputSourceMethods,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrsession::XRSession;
use crate::dom::xrspace::XRSpace;
use dom_struct::dom_struct;
use webxr_api::{Handedness, InputInfo};

#[dom_struct]
pub struct XRInputSource {
    reflector: Reflector,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    info: InputInfo,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    target_ray_space: MutNullableDom<XRSpace>,
}

impl XRInputSource {
    pub fn new_inherited(session: &XRSession, info: InputInfo) -> XRInputSource {
        XRInputSource {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            info,
            target_ray_space: Default::default(),
        }
    }

    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        info: InputInfo,
    ) -> DomRoot<XRInputSource> {
        reflect_dom_object(
            Box::new(XRInputSource::new_inherited(session, info)),
            global,
            XRInputSourceBinding::Wrap,
        )
    }

    pub fn id(&self) -> u32 {
        self.info.id
    }
}

impl XRInputSourceMethods for XRInputSource {
    /// https://immersive-web.github.io/webxr/#dom-xrinputsource-handedness
    fn Handedness(&self) -> XRHandedness {
        match self.info.handedness {
            Handedness::None => XRHandedness::None,
            Handedness::Left => XRHandedness::Left,
            Handedness::Right => XRHandedness::Right,
        }
    }

    /// https://immersive-web.github.io/webxr/#dom-xrinputsource-targetrayspace
    fn TargetRaySpace(&self) -> DomRoot<XRSpace> {
        self.target_ray_space.or_init(|| {
            let global = self.global();
            XRSpace::new_inputspace(&global, &self.session, &self)
        })
    }
}
