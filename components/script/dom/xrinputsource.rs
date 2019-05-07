/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
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
use webvr_traits::{WebVRGamepadData, WebVRGamepadHand, WebVRGamepadState, WebVRPose};

#[dom_struct]
pub struct XRInputSource {
    reflector: Reflector,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    data: WebVRGamepadData,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    state: DomRefCell<WebVRGamepadState>,
    target_ray_space: MutNullableDom<XRSpace>,
}

impl XRInputSource {
    pub fn new_inherited(
        session: &XRSession,
        data: WebVRGamepadData,
        state: WebVRGamepadState,
    ) -> XRInputSource {
        XRInputSource {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            data,
            state: DomRefCell::new(state),
            target_ray_space: Default::default(),
        }
    }

    pub fn new(
        global: &GlobalScope,
        session: &XRSession,
        data: WebVRGamepadData,
        state: WebVRGamepadState,
    ) -> DomRoot<XRInputSource> {
        reflect_dom_object(
            Box::new(XRInputSource::new_inherited(session, data, state)),
            global,
            XRInputSourceBinding::Wrap,
        )
    }

    pub fn update_state(&self, state: WebVRGamepadState) {
        *self.state.borrow_mut() = state;
    }

    pub fn pose(&self) -> WebVRPose {
        self.state.borrow().pose
    }
}

impl XRInputSourceMethods for XRInputSource {
    /// https://immersive-web.github.io/webxr/#dom-xrinputsource-handedness
    fn Handedness(&self) -> XRHandedness {
        match self.data.hand {
            WebVRGamepadHand::Unknown => XRHandedness::None,
            WebVRGamepadHand::Left => XRHandedness::Left,
            WebVRGamepadHand::Right => XRHandedness::Right,
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
