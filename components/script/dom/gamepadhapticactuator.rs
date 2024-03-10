/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsval::JSVal;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::GamepadHapticActuatorBinding::{
    GamepadEffectParameters, GamepadHapticActuatorMethods, GamepadHapticEffectType,
};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::Performance_Binding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::JSContext;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct GamepadHapticActuator {
    reflector_: Reflector,
    effects: Vec<GamepadHapticEffectType>,
    #[ignore_malloc_size_of = "promises are hard"]
    playing_effect_promise: DomRefCell<Option<Rc<Promise>>>,
}

impl GamepadHapticActuator {
    fn new_inherited() -> GamepadHapticActuator {
        Self {
            reflector_: Reflector::new(),
            effects: vec![GamepadHapticEffectType::Dual_rumble],
            playing_effect_promise: DomRefCell::new(None),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<GamepadHapticActuator> {
        Self::new_with_proto(global)
    }

    fn new_with_proto(global: &GlobalScope) -> DomRoot<GamepadHapticActuator> {
        let haptic_actuator = reflect_dom_object_with_proto(
            Box::new(GamepadHapticActuator::new_inherited()),
            global,
            None,
        );
        haptic_actuator
    }
}

impl GamepadHapticActuatorMethods for GamepadHapticActuator {
    /// <https://www.w3.org/TR/gamepad/#dom-gamepadhapticactuator-effects>
    fn Effects(&self, cx: JSContext) -> JSVal {
        to_frozen_array(self.effects.as_slice(), cx)
    }

    /// <https://www.w3.org/TR/gamepad/#dom-gamepadhapticactuator-playeffect>
    fn PlayEffect(
        &self,
        type_: GamepadHapticEffectType,
        params: &GamepadEffectParameters,
    ) -> Rc<Promise> {
        let playing_effect_promise = Promise::new(&self.global());

        // <https://www.w3.org/TR/gamepad/#dfn-valid-effect>
        match type_ {
            // <https://www.w3.org/TR/gamepad/#dfn-valid-dual-rumble-effect>
            GamepadHapticEffectType::Dual_rumble => {
                if *params.strongMagnitude < 0.0 || *params.strongMagnitude > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Strong magnitude value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                } else if *params.weakMagnitude < 0.0 || *params.weakMagnitude > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Weak magnitude value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                }
            },
        }

        let document = self.global().as_window().Document();
        if !document.is_fully_active() {
            playing_effect_promise.reject_error(Error::InvalidState);
        }

        if self.playing_effect_promise.borrow().is_some() {
            let trusted_promise = TrustedPromise::new(
                self.playing_effect_promise
                    .borrow()
                    .clone()
                    .expect("Promise is null!"),
            );
            *self.playing_effect_promise.borrow_mut() = None;
            let _ = self.global().gamepad_task_source().queue(
                task!(preempt_promise: move || {
                    let promise = trusted_promise.root();
                    let message = DOMString::from("preempted");
                    promise.resolve_native(&message);
                }),
                &self.global(),
            );
        }

        if !self.effects.contains(&type_) {
            playing_effect_promise.reject_error(Error::NotSupported);
        }

        *self.playing_effect_promise.borrow_mut() = Some(playing_effect_promise.clone());
        let play_effect_timestamp = self.global().performance().Now();

        // TODO: Play haptic effect

        playing_effect_promise
    }

    /// <https://www.w3.org/TR/gamepad/#dom-gamepadhapticactuator-reset>
    fn Reset(&self) -> Rc<Promise> {
        let reset_result_promise = Promise::new(&self.global());

        let document = self.global().as_window().Document();
        if !document.is_fully_active() {
            reset_result_promise.reject_error(Error::InvalidState);
        }

        if self.playing_effect_promise.borrow().is_some() {
            // TODO: Stop existing haptic effect
            let completed_successfully = true;
            if completed_successfully {
                let message = DOMString::from("completed");
                reset_result_promise.resolve_native(&message);
            }
        }

        reset_result_promise
    }
}

impl GamepadHapticActuator {
    pub fn has_playing_effect_promise(&self) -> bool {
        self.playing_effect_promise.borrow().is_some()
    }

    pub fn resolve_playing_effect_promise(&self) {
        let playing_effect_promise = self.playing_effect_promise.borrow().clone();
        if let Some(promise) = playing_effect_promise {
            let message = DOMString::from("completed");
            promise.resolve_native(&message);
        }
    }

    /// <https://www.w3.org/TR/gamepad/#handling-visibility-change>
    #[allow(dead_code)]
    pub fn handle_visibility_change(&self) {
        if self.playing_effect_promise.borrow().is_none() {
            return;
        }

        let trusted_promise = TrustedPromise::new(
            self.playing_effect_promise
                .borrow()
                .clone()
                .expect("Promise is null!"),
        );

        let this = Trusted::new(&*self);

        let _ = self.global().gamepad_task_source().queue(
            task!(stop_playing_effect: move || {
                let promise = trusted_promise.root();
                let actuator = this.root();
                let message = DOMString::from("preempted");
                promise.resolve_native(&message);
                *actuator.playing_effect_promise.borrow_mut() = None;
            }),
            &self.global(),
        );

        // TODO: Stop haptic effect
    }
}
