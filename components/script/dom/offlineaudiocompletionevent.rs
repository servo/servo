/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use stylo_atoms::Atom;

use crate::dom::audiobuffer::AudioBuffer;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::OfflineAudioCompletionEventBinding::{
    OfflineAudioCompletionEventInit, OfflineAudioCompletionEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OfflineAudioCompletionEvent {
    event: Event,
    rendered_buffer: Dom<AudioBuffer>,
}

impl OfflineAudioCompletionEvent {
    pub(crate) fn new_inherited(rendered_buffer: &AudioBuffer) -> OfflineAudioCompletionEvent {
        OfflineAudioCompletionEvent {
            event: Event::new_inherited(),
            rendered_buffer: Dom::from_ref(rendered_buffer),
        }
    }

    pub(crate) fn new(
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        rendered_buffer: &AudioBuffer,
        can_gc: CanGc,
    ) -> DomRoot<OfflineAudioCompletionEvent> {
        Self::new_with_proto(
            window,
            None,
            type_,
            bubbles,
            cancelable,
            rendered_buffer,
            can_gc,
        )
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        rendered_buffer: &AudioBuffer,
        can_gc: CanGc,
    ) -> DomRoot<OfflineAudioCompletionEvent> {
        let event = Box::new(OfflineAudioCompletionEvent::new_inherited(rendered_buffer));
        let ev = reflect_dom_object_with_proto(event, window, proto, can_gc);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }
}

impl OfflineAudioCompletionEventMethods<crate::DomTypeHolder> for OfflineAudioCompletionEvent {
    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocompletionevent-offlineaudiocompletionevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &OfflineAudioCompletionEventInit,
    ) -> Fallible<DomRoot<OfflineAudioCompletionEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(OfflineAudioCompletionEvent::new_with_proto(
            window,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            &init.renderedBuffer,
            can_gc,
        ))
    }

    // https://webaudio.github.io/web-audio-api/#dom-offlineaudiocompletionevent-renderedbuffer
    fn RenderedBuffer(&self) -> DomRoot<AudioBuffer> {
        DomRoot::from_ref(&*self.rendered_buffer)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
