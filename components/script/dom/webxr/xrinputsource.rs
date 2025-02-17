/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::GamepadSupportedHapticEffects;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;
use webxr_api::{Handedness, InputFrame, InputId, InputSource, TargetRayMode};

use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding::{
    XRHandedness, XRInputSourceMethods, XRTargetRayMode,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::gamepad::Gamepad;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrhand::XRHand;
use crate::dom::xrsession::XRSession;
use crate::dom::xrspace::XRSpace;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct XRInputSource {
    reflector: Reflector,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "Defined in rust-webxr"]
    #[no_trace]
    info: InputSource,
    target_ray_space: MutNullableDom<XRSpace>,
    grip_space: MutNullableDom<XRSpace>,
    hand: MutNullableDom<XRHand>,
    #[ignore_malloc_size_of = "mozjs"]
    profiles: Heap<JSVal>,
    gamepad: DomRoot<Gamepad>,
}

impl XRInputSource {
    pub(crate) fn new_inherited(
        global: &GlobalScope,
        session: &XRSession,
        info: InputSource,
        can_gc: CanGc,
    ) -> XRInputSource {
        // <https://www.w3.org/TR/webxr-gamepads-module-1/#gamepad-differences>
        let gamepad = Gamepad::new(
            global,
            0,
            "".into(),
            "xr-standard".into(),
            (-1.0, 1.0),
            (0.0, 1.0),
            GamepadSupportedHapticEffects {
                supports_dual_rumble: false,
                supports_trigger_rumble: false,
            },
            true,
            can_gc,
        );
        XRInputSource {
            reflector: Reflector::new(),
            session: Dom::from_ref(session),
            info,
            target_ray_space: Default::default(),
            grip_space: Default::default(),
            hand: Default::default(),
            profiles: Heap::default(),
            gamepad,
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn new(
        global: &GlobalScope,
        session: &XRSession,
        info: InputSource,
        can_gc: CanGc,
    ) -> DomRoot<XRInputSource> {
        let source = reflect_dom_object(
            Box::new(XRInputSource::new_inherited(global, session, info, can_gc)),
            global,
            can_gc,
        );

        let _ac = enter_realm(global);
        let cx = GlobalScope::get_cx();
        unsafe {
            rooted!(in(*cx) let mut profiles = UndefinedValue());
            source.info.profiles.to_jsval(*cx, profiles.handle_mut());
            source.profiles.set(profiles.get());
        }
        source
    }

    pub(crate) fn id(&self) -> InputId {
        self.info.id
    }

    pub(crate) fn session(&self) -> &XRSession {
        &self.session
    }

    pub(crate) fn update_gamepad_state(&self, frame: InputFrame) {
        frame
            .button_values
            .iter()
            .enumerate()
            .for_each(|(i, value)| {
                self.gamepad.map_and_normalize_buttons(i, *value as f64);
            });
        frame.axis_values.iter().enumerate().for_each(|(i, value)| {
            self.gamepad.map_and_normalize_axes(i, *value as f64);
        });
    }

    pub(crate) fn gamepad(&self) -> &DomRoot<Gamepad> {
        &self.gamepad
    }
}

impl XRInputSourceMethods<crate::DomTypeHolder> for XRInputSource {
    /// <https://immersive-web.github.io/webxr/#dom-xrinputsource-handedness>
    fn Handedness(&self) -> XRHandedness {
        match self.info.handedness {
            Handedness::None => XRHandedness::None,
            Handedness::Left => XRHandedness::Left,
            Handedness::Right => XRHandedness::Right,
        }
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrinputsource-targetraymode>
    fn TargetRayMode(&self) -> XRTargetRayMode {
        match self.info.target_ray_mode {
            TargetRayMode::Gaze => XRTargetRayMode::Gaze,
            TargetRayMode::TrackedPointer => XRTargetRayMode::Tracked_pointer,
            TargetRayMode::Screen => XRTargetRayMode::Screen,
            TargetRayMode::TransientPointer => XRTargetRayMode::Transient_pointer,
        }
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrinputsource-targetrayspace>
    fn TargetRaySpace(&self) -> DomRoot<XRSpace> {
        self.target_ray_space.or_init(|| {
            let global = self.global();
            XRSpace::new_inputspace(&global, &self.session, self, false)
        })
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrinputsource-gripspace>
    fn GetGripSpace(&self) -> Option<DomRoot<XRSpace>> {
        if self.info.supports_grip {
            Some(self.grip_space.or_init(|| {
                let global = self.global();
                XRSpace::new_inputspace(&global, &self.session, self, true)
            }))
        } else {
            None
        }
    }
    // https://immersive-web.github.io/webxr/#dom-xrinputsource-profiles
    fn Profiles(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.profiles.get())
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrinputsource-skiprendering>
    fn SkipRendering(&self) -> bool {
        // Servo is not currently supported anywhere that would allow for skipped
        // controller rendering.
        false
    }

    /// <https://www.w3.org/TR/webxr-gamepads-module-1/#xrinputsource-interface>
    fn GetGamepad(&self) -> Option<DomRoot<Gamepad>> {
        Some(DomRoot::from_ref(&*self.gamepad))
    }

    // https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md
    fn GetHand(&self) -> Option<DomRoot<XRHand>> {
        self.info.hand_support.as_ref().map(|hand| {
            self.hand
                .or_init(|| XRHand::new(&self.global(), self, hand.clone()))
        })
    }
}
