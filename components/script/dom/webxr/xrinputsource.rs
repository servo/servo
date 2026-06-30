/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::GamepadSupportedHapticEffects;
use js::context::JSContext;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::MutableHandleValue;
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use webxr_api::{Handedness, InputFrame, InputId, InputSource, TargetRayMode};

use crate::dom::bindings::codegen::Bindings::XRInputSourceBinding::{
    XRHandedness, XRInputSourceMethods, XRTargetRayMode,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::gamepad::Gamepad;
use crate::dom::window::Window;
use crate::dom::xrhand::XRHand;
use crate::dom::xrsession::XRSession;
use crate::dom::xrspace::XRSpace;
use crate::realms::enter_auto_realm;

#[dom_struct]
pub(crate) struct XRInputSource {
    reflector: Reflector,
    session: Dom<XRSession>,
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
        cx: &mut JSContext,
        window: &Window,
        session: &XRSession,
        info: InputSource,
    ) -> XRInputSource {
        // <https://www.w3.org/TR/webxr-gamepads-module-1/#gamepad-differences>
        let gamepad = Gamepad::new(
            cx,
            window,
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

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        session: &XRSession,
        info: InputSource,
    ) -> DomRoot<XRInputSource> {
        let source = reflect_dom_object_with_cx(
            Box::new(XRInputSource::new_inherited(cx, window, session, info)),
            window,
            cx,
        );

        let mut realm = enter_auto_realm(cx, window);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut profiles = UndefinedValue());
        source
            .info
            .profiles
            .safe_to_jsval(cx, profiles.handle_mut());
        source.profiles.set(profiles.get());
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
    fn TargetRaySpace(&self, cx: &mut JSContext) -> DomRoot<XRSpace> {
        self.target_ray_space.or_init(|| {
            let global = self.global();
            XRSpace::new_inputspace(cx, &global, &self.session, self, false)
        })
    }

    /// <https://immersive-web.github.io/webxr/#dom-xrinputsource-gripspace>
    fn GetGripSpace(&self, cx: &mut JSContext) -> Option<DomRoot<XRSpace>> {
        if self.info.supports_grip {
            Some(self.grip_space.or_init(|| {
                let global = self.global();
                XRSpace::new_inputspace(cx, &global, &self.session, self, true)
            }))
        } else {
            None
        }
    }
    /// <https://immersive-web.github.io/webxr/#dom-xrinputsource-profiles>
    fn Profiles(&self, mut retval: MutableHandleValue) {
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

    /// <https://github.com/immersive-web/webxr-hands-input/blob/master/explainer.md>
    fn GetHand(&self, cx: &mut JSContext) -> Option<DomRoot<XRHand>> {
        self.info.hand_support.as_ref().map(|hand| {
            self.hand
                .or_init(|| XRHand::new(cx, &self.global(), self, hand.clone()))
        })
    }
}
