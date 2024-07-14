/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use base::id::GamepadRouterId;
use dom_struct::dom_struct;
use embedder_traits::{DualRumbleEffectParams, EmbedderMsg};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsval::JSVal;
use script_traits::{GamepadSupportedHapticEffects, ScriptMsg};

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
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::JSContext;
use crate::task::TaskCanceller;
use crate::task_source::gamepad::GamepadTaskSource;
use crate::task_source::{TaskSource, TaskSourceName};

struct HapticEffectStopListener {
    canceller: TaskCanceller,
    task_source: GamepadTaskSource,
    context: Trusted<GamepadHapticActuator>,
}

impl HapticEffectStopListener {
    fn handle(&self, stopped_successfully: bool) {
        let context = self.context.clone();

        let _ = self.task_source.queue_with_canceller(
            task!(handle_haptic_effect_stopped: move || {
                let actuator = context.root();
                actuator.handle_haptic_effect_stopped(stopped_successfully);
            }),
            &self.canceller,
        );
    }
}

#[dom_struct]
pub struct GamepadHapticActuator {
    reflector_: Reflector,
    gamepad_index: u32,
    effects: Vec<GamepadHapticEffectType>,
    #[ignore_malloc_size_of = "Rc is hard"]
    playing_effect_promise: DomRefCell<Option<Rc<Promise>>>,
    sequence_id: Cell<u32>,
    effect_sequence_id: Cell<u32>,
    reset_sequence_id: Cell<u32>,
}

impl GamepadHapticActuator {
    fn new_inherited(
        gamepad_index: u32,
        supported_haptic_effects: GamepadSupportedHapticEffects,
    ) -> GamepadHapticActuator {
        let mut effects = vec![];
        if supported_haptic_effects.supports_dual_rumble {
            effects.push(GamepadHapticEffectType::Dual_rumble);
        }
        if supported_haptic_effects.supports_trigger_rumble {
            effects.push(GamepadHapticEffectType::Trigger_rumble);
        }
        Self {
            reflector_: Reflector::new(),
            gamepad_index: gamepad_index.into(),
            effects,
            playing_effect_promise: DomRefCell::new(None),
            sequence_id: Cell::new(0),
            effect_sequence_id: Cell::new(0),
            reset_sequence_id: Cell::new(0),
        }
    }

    pub fn new(
        global: &GlobalScope,
        gamepad_index: u32,
        supported_haptic_effects: GamepadSupportedHapticEffects,
    ) -> DomRoot<GamepadHapticActuator> {
        Self::new_with_proto(global, gamepad_index, supported_haptic_effects)
    }

    fn new_with_proto(
        global: &GlobalScope,
        gamepad_index: u32,
        supported_haptic_effects: GamepadSupportedHapticEffects,
    ) -> DomRoot<GamepadHapticActuator> {
        let haptic_actuator = reflect_dom_object_with_proto(
            Box::new(GamepadHapticActuator::new_inherited(
                gamepad_index,
                supported_haptic_effects,
            )),
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
        let in_realm_proof = AlreadyInRealm::assert();
        let playing_effect_promise =
            Promise::new_in_current_realm(InRealm::Already(&in_realm_proof));

        // <https://www.w3.org/TR/gamepad/#dfn-valid-effect>
        match type_ {
            // <https://www.w3.org/TR/gamepad/#dfn-valid-dual-rumble-effect>
            GamepadHapticEffectType::Dual_rumble => {
                if *params.strongMagnitude < 0.0 || *params.strongMagnitude > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Strong magnitude value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                    return playing_effect_promise;
                } else if *params.weakMagnitude < 0.0 || *params.weakMagnitude > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Weak magnitude value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                    return playing_effect_promise;
                }
            },
            // <https://www.w3.org/TR/gamepad/#dfn-valid-trigger-rumble-effect>
            GamepadHapticEffectType::Trigger_rumble => {
                if *params.strongMagnitude < 0.0 || *params.strongMagnitude > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Strong magnitude value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                    return playing_effect_promise;
                } else if *params.weakMagnitude < 0.0 || *params.weakMagnitude > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Weak magnitude value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                    return playing_effect_promise;
                } else if *params.leftTrigger < 0.0 || *params.leftTrigger > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Left trigger value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                    return playing_effect_promise;
                } else if *params.rightTrigger < 0.0 || *params.rightTrigger > 1.0 {
                    playing_effect_promise.reject_error(Error::Type(
                        "Right trigger value is not within range of 0.0 to 1.0.".to_string(),
                    ));
                    return playing_effect_promise;
                }
            },
        }

        let document = self.global().as_window().Document();
        if !document.is_fully_active() {
            playing_effect_promise.reject_error(Error::InvalidState);
        }

        self.sequence_id.set(self.sequence_id.get().wrapping_add(1));

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

        assert!(self.playing_effect_promise.borrow().is_none());

        *self.playing_effect_promise.borrow_mut() = Some(playing_effect_promise.clone());
        self.effect_sequence_id.set(self.sequence_id.get());

        // XXX The spec says we SHOULD also pass a playEffectTimestamp for more precise playback timing
        // when start_delay is non-zero, but this is left more as a footnote without much elaboration.

        let params = DualRumbleEffectParams {
            duration: params.duration as f64,
            start_delay: params.startDelay as f64,
            strong_magnitude: *params.strongMagnitude,
            weak_magnitude: *params.weakMagnitude,
        };
        let event = EmbedderMsg::PlayGamepadHapticEffect(
            self.gamepad_index as usize,
            embedder_traits::GamepadHapticEffectType::DualRumble(params),
        );
        self.global().as_window().send_to_embedder(event);

        playing_effect_promise
    }

    /// <https://www.w3.org/TR/gamepad/#dom-gamepadhapticactuator-reset>
    fn Reset(&self) -> Rc<Promise> {
        let in_realm_proof = AlreadyInRealm::assert();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof));

        let document = self.global().as_window().Document();
        if !document.is_fully_active() {
            promise.reject_error(Error::InvalidState);
            return promise;
        }

        self.sequence_id.set(self.sequence_id.get().wrapping_add(1));

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

        assert!(self.playing_effect_promise.borrow().is_none());

        *self.playing_effect_promise.borrow_mut() = Some(promise.clone());

        self.reset_sequence_id.set(self.sequence_id.get());

        let context = Trusted::new(self);
        let (effect_stop_sender, effect_stop_receiver) =
            ipc::channel().expect("ipc channel failure");
        let (task_source, canceller) = (
            self.global().gamepad_task_source(),
            self.global().task_canceller(TaskSourceName::Gamepad),
        );
        let listener = HapticEffectStopListener {
            canceller,
            task_source,
            context,
        };

        ROUTER.add_route(
            effect_stop_receiver.to_opaque(),
            Box::new(move |message| {
                let msg = message.to::<bool>();
                match msg {
                    Ok(msg) => listener.handle(msg),
                    Err(err) => warn!("Error receiving a GamepadMsg: {:?}", err),
                }
            }),
        );

        let router_id = GamepadRouterId::new();
        let _ = self
            .global()
            .script_to_constellation_chan()
            .send(ScriptMsg::NewGamepadRouter(
                router_id,
                effect_stop_sender.clone(),
                self.global().origin().immutable().clone(),
            ));

        let event =
            EmbedderMsg::StopGamepadHapticEffect(self.gamepad_index as usize, effect_stop_sender);
        self.global().as_window().send_to_embedder(event);

        self.playing_effect_promise.borrow().clone().unwrap()
    }
}

impl GamepadHapticActuator {
    pub fn has_playing_effect_promise(&self) -> bool {
        self.playing_effect_promise.borrow().is_some()
    }

    /// <https://www.w3.org/TR/gamepad/#dom-gamepadhapticactuator-playeffect>
    /// We are in the task queued by the "in-parallel" steps.
    pub fn handle_haptic_effect_completed(&self) {
        if self.effect_sequence_id.get() != self.sequence_id.get() {
            return;
        }
        let playing_effect_promise = self.playing_effect_promise.borrow_mut().take();
        if let Some(promise) = playing_effect_promise {
            let message = DOMString::from("complete");
            promise.resolve_native(&message);
        }
    }

    pub fn handle_haptic_effect_stopped(&self, stopped_successfully: bool) {
        if !stopped_successfully {
            return;
        }

        let playing_effect_promise = self.playing_effect_promise.borrow_mut().take();

        if let Some(promise) = playing_effect_promise {
            let trusted_promise = TrustedPromise::new(promise);
            let sequence_id = self.sequence_id.get();
            let reset_sequence_id = self.reset_sequence_id.get();
            let _ = self.global().gamepad_task_source().queue(
                task!(complete_promise: move || {
                    if sequence_id != reset_sequence_id {
                        println!("Mismatched sequence/reset sequence ids: {} != {}", sequence_id, reset_sequence_id);
                        return;
                    }
                    let promise = trusted_promise.root();
                    let message = DOMString::from("complete");
                    promise.resolve_native(&message);
                }),
                &self.global(),
            );
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

        let (send, _rcv) = ipc::channel().expect("ipc channel failure");

        let event = EmbedderMsg::StopGamepadHapticEffect(self.gamepad_index as usize, send);
        self.global().as_window().send_to_embedder(event);
    }
}
