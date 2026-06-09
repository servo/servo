/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto_and_cx;
use stylo_atoms::Atom;

use crate::dom::audio::audiobuffer::AudioBuffer;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::OfflineAudioCompletionEventBinding::{
    OfflineAudioCompletionEventInit, OfflineAudioCompletionEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;

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
        cx: &mut JSContext,
        window: &Window,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        rendered_buffer: &AudioBuffer,
    ) -> DomRoot<OfflineAudioCompletionEvent> {
        Self::new_with_proto(
            cx,
            window,
            None,
            type_,
            bubbles,
            cancelable,
            rendered_buffer,
        )
    }

    fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        rendered_buffer: &AudioBuffer,
    ) -> DomRoot<OfflineAudioCompletionEvent> {
        let event = Box::new(OfflineAudioCompletionEvent::new_inherited(rendered_buffer));
        let ev = reflect_dom_object_with_proto_and_cx(event, window, proto, cx);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }
}

impl OfflineAudioCompletionEventMethods<crate::DomTypeHolder> for OfflineAudioCompletionEvent {
    /// <https://webaudio.github.io/web-audio-api/#dom-offlineaudiocompletionevent-offlineaudiocompletionevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &OfflineAudioCompletionEventInit,
    ) -> Fallible<DomRoot<OfflineAudioCompletionEvent>> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(OfflineAudioCompletionEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            &init.renderedBuffer,
        ))
    }

    /// <https://webaudio.github.io/web-audio-api/#dom-offlineaudiocompletionevent-renderedbuffer>
    fn RenderedBuffer(&self) -> DomRoot<AudioBuffer> {
        DomRoot::from_ref(&*self.rendered_buffer)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
