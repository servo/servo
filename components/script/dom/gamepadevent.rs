/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::GamepadEventBinding;
use crate::dom::bindings::codegen::Bindings::GamepadEventBinding::GamepadEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::gamepad::Gamepad;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub struct GamepadEvent {
    event: Event,
    gamepad: Dom<Gamepad>,
}

pub enum GamepadEventType {
    Connected,
    Disconnected,
}

impl GamepadEvent {
    fn new_inherited(gamepad: &Gamepad) -> GamepadEvent {
        GamepadEvent {
            event: Event::new_inherited(),
            gamepad: Dom::from_ref(gamepad),
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        gamepad: &Gamepad,
    ) -> DomRoot<GamepadEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, gamepad)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        gamepad: &Gamepad,
    ) -> DomRoot<GamepadEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(GamepadEvent::new_inherited(gamepad)),
            global,
            proto,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn new_with_type(
        global: &GlobalScope,
        event_type: GamepadEventType,
        gamepad: &Gamepad,
    ) -> DomRoot<GamepadEvent> {
        let name = match event_type {
            GamepadEventType::Connected => "gamepadconnected",
            GamepadEventType::Disconnected => "gamepaddisconnected",
        };

        GamepadEvent::new(global, name.into(), false, false, gamepad)
    }

    // https://w3c.github.io/gamepad/#gamepadevent-interface
    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &GamepadEventBinding::GamepadEventInit,
    ) -> Fallible<DomRoot<GamepadEvent>> {
        Ok(GamepadEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.gamepad,
        ))
    }
}

impl GamepadEventMethods for GamepadEvent {
    // https://w3c.github.io/gamepad/#gamepadevent-interface
    fn Gamepad(&self) -> DomRoot<Gamepad> {
        DomRoot::from_ref(&*self.gamepad)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
