/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::AnimationEventBinding::{
    AnimationEventInit, AnimationEventMethods,
};
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::window::Window;

#[dom_struct]
pub struct AnimationEvent {
    event: Event,
    #[no_trace]
    animation_name: Atom,
    elapsed_time: Finite<f32>,
    pseudo_element: DOMString,
}

impl AnimationEvent {
    fn new_inherited(init: &AnimationEventInit) -> AnimationEvent {
        AnimationEvent {
            event: Event::new_inherited(),
            animation_name: Atom::from(init.animationName.clone()),
            elapsed_time: init.elapsedTime,
            pseudo_element: init.pseudoElement.clone(),
        }
    }

    pub fn new(window: &Window, type_: Atom, init: &AnimationEventInit) -> DomRoot<AnimationEvent> {
        Self::new_with_proto(window, None, type_, init)
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        init: &AnimationEventInit,
    ) -> DomRoot<AnimationEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(AnimationEvent::new_inherited(init)),
            window,
            proto,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, init.parent.bubbles, init.parent.cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &AnimationEventInit,
    ) -> DomRoot<AnimationEvent> {
        AnimationEvent::new_with_proto(window, proto, Atom::from(type_), init)
    }
}

impl AnimationEventMethods for AnimationEvent {
    // https://drafts.csswg.org/css-animations/#interface-animationevent-attributes
    fn AnimationName(&self) -> DOMString {
        DOMString::from(&*self.animation_name)
    }

    // https://drafts.csswg.org/css-animations/#interface-animationevent-attributes
    fn ElapsedTime(&self) -> Finite<f32> {
        self.elapsed_time
    }

    // https://drafts.csswg.org/css-animations/#interface-animationevent-attributes
    fn PseudoElement(&self) -> DOMString {
        self.pseudo_element.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.upcast::<Event>().IsTrusted()
    }
}
