/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::audiobuffer::AudioBuffer;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::OfflineAudioCompletionEventBinding;
use dom::bindings::codegen::Bindings::OfflineAudioCompletionEventBinding::OfflineAudioCompletionEventInit;
use dom::bindings::codegen::Bindings::OfflineAudioCompletionEventBinding::OfflineAudioCompletionEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot, RootedReference};
use dom::bindings::str::DOMString;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

#[dom_struct]
pub struct OfflineAudioCompletionEvent {
    event: Event,
    rendered_buffer: Dom<AudioBuffer>,
}

impl OfflineAudioCompletionEvent {
    pub fn new_inherited(rendered_buffer: &AudioBuffer) -> OfflineAudioCompletionEvent {
        OfflineAudioCompletionEvent {
            event: Event::new_inherited(),
            rendered_buffer: Dom::from_ref(rendered_buffer),
        }
    }

    pub fn new(
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        rendered_buffer: &AudioBuffer,
    ) -> DomRoot<OfflineAudioCompletionEvent> {
        let event = Box::new(OfflineAudioCompletionEvent::new_inherited(rendered_buffer));
        let ev = reflect_dom_object(event, window, OfflineAudioCompletionEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }

    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &OfflineAudioCompletionEventInit,
    ) -> Fallible<DomRoot<OfflineAudioCompletionEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(OfflineAudioCompletionEvent::new(
            window,
            Atom::from(type_),
            bubbles,
            cancelable,
            init.renderedBuffer.r(),
        ))
    }
}

impl OfflineAudioCompletionEventMethods for OfflineAudioCompletionEvent {
    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocompletionevent-renderedbuffer
    fn RenderedBuffer(&self) -> DomRoot<AudioBuffer> {
        DomRoot::from_ref(&*self.rendered_buffer)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
