/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::str::DOMString;
use style::Atom;

use crate::dom::bindings::codegen::Bindings::ClipboardEventBinding::{
    ClipboardEventInit, ClipboardEventMethods,
};
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::datatransfer::DataTransfer;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::window::Window;

/// The types of clipboard events in the Clipboard APIs specification:
/// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions>.
#[derive(Clone, Debug)]
pub(crate) enum ClipboardEventType {
    Change,
    Copy,
    Cut,
    Paste,
}

impl ClipboardEventType {
    /// Convert this [`ClipboardEventType`] to a `&str` for use in creating DOM events.
    pub(crate) fn as_str(&self) -> &str {
        match *self {
            ClipboardEventType::Change => "clipboardchange",
            ClipboardEventType::Copy => "copy",
            ClipboardEventType::Cut => "cut",
            ClipboardEventType::Paste => "paste",
        }
    }
}

#[dom_struct]
pub(crate) struct ClipboardEvent {
    event: Event,
    clipboard_data: MutNullableDom<DataTransfer>,
}

impl ClipboardEvent {
    fn new_inherited() -> ClipboardEvent {
        ClipboardEvent {
            event: Event::new_inherited(),
            clipboard_data: MutNullableDom::new(None),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        event_type: Atom,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        clipboard_data: Option<&DataTransfer>,
    ) -> DomRoot<ClipboardEvent> {
        let ev = reflect_dom_object_with_proto(
            cx,
            Box::new(ClipboardEvent::new_inherited()),
            window,
            proto,
        );
        ev.upcast::<Event>()
            .init_event(event_type, bool::from(can_bubble), bool::from(cancelable));
        ev.clipboard_data.set(clipboard_data);
        ev
    }

    pub(crate) fn set_clipboard_data(&self, clipboard_data: Option<&DataTransfer>) {
        self.clipboard_data.set(clipboard_data);
    }

    pub(crate) fn get_clipboard_data(&self) -> Option<DomRoot<DataTransfer>> {
        self.clipboard_data.get()
    }
}

impl ClipboardEventMethods<crate::DomTypeHolder> for ClipboardEvent {
    /// <https://www.w3.org/TR/clipboard-apis/#dom-clipboardevent-clipboardevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        event_type: DOMString,
        init: &ClipboardEventInit,
    ) -> DomRoot<ClipboardEvent> {
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        let event = ClipboardEvent::new(
            cx,
            window,
            proto,
            event_type.into(),
            bubbles,
            cancelable,
            init.clipboardData.as_deref(),
        );
        event.upcast::<Event>().set_composed(init.parent.composed);
        event
    }

    /// <https://www.w3.org/TR/clipboard-apis/#dom-clipboardevent-clipboarddata>
    fn GetClipboardData(&self) -> Option<DomRoot<DataTransfer>> {
        self.clipboard_data.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
