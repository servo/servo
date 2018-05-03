/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::GamepadEventBinding;
use dom::bindings::codegen::Bindings::GamepadEventBinding::GamepadEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::gamepad::Gamepad;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct GamepadEvent<TH: TypeHolderTrait> {
    event: Event<TH>,
    gamepad: Dom<Gamepad<TH>>,
}

pub enum GamepadEventType {
    Connected,
    Disconnected
}

impl<TH: TypeHolderTrait> GamepadEvent<TH> {
    fn new_inherited(gamepad: &Gamepad<TH>) -> GamepadEvent<TH> {
        GamepadEvent {
            event: Event::new_inherited(),
            gamepad: Dom::from_ref(gamepad),
        }
    }

    pub fn new(global: &GlobalScope<TH>,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               gamepad: &Gamepad<TH>)
               -> DomRoot<GamepadEvent<TH>> {
        let ev = reflect_dom_object(
            Box::new(GamepadEvent::new_inherited(&gamepad)), global, GamepadEventBinding::Wrap
        );
        {
            let event = ev.upcast::<Event<TH>>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn new_with_type(global: &GlobalScope<TH>, event_type: GamepadEventType, gamepad: &Gamepad<TH>)
                         -> DomRoot<GamepadEvent<TH>> {
        let name = match event_type {
            GamepadEventType::Connected => "gamepadconnected",
            GamepadEventType::Disconnected => "gamepaddisconnected"
        };

        GamepadEvent::new(&global,
                          name.into(),
                          false,
                          false,
                          &gamepad)
    }

    // https://w3c.github.io/gamepad/#gamepadevent-interface
    pub fn Constructor(window: &Window<TH>,
                       type_: DOMString,
                       init: &GamepadEventBinding::GamepadEventInit<TH>)
                       -> Fallible<DomRoot<GamepadEvent<TH>>> {
        Ok(GamepadEvent::new(&window.global(),
                             Atom::from(type_),
                             init.parent.bubbles,
                             init.parent.cancelable,
                             &init.gamepad))
    }
}

impl<TH: TypeHolderTrait> GamepadEventMethods<TH> for GamepadEvent<TH> {
    // https://w3c.github.io/gamepad/#gamepadevent-interface
    fn Gamepad(&self) -> DomRoot<Gamepad<TH>> {
        DomRoot::from_ref(&*self.gamepad)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
