/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::match_domstring_ascii;
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
use crate::drag::drag_data_store::Kind;

/// The types of clipboard events in the Clipboard APIs specification:
/// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions>.
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) enum ClipboardEventType {
    Change,
    Copy,
    Cut,
    Paste,
    Other(Atom),
}

impl ClipboardEventType {
    /// Convert this [`ClipboardEventType`] to an `Atom` for use in creating DOM events.
    pub(crate) fn as_atom(&self) -> Atom {
        match self {
            ClipboardEventType::Change => "clipboardchange".into(),
            ClipboardEventType::Copy => "copy".into(),
            ClipboardEventType::Cut => "cut".into(),
            ClipboardEventType::Paste => "paste".into(),
            ClipboardEventType::Other(atom) => atom.clone(),
        }
    }
}

impl From<DOMString> for ClipboardEventType {
    fn from(value: DOMString) -> Self {
        match_domstring_ascii!(value,
            "clipboardchange" => return ClipboardEventType::Change,
            "copy" => return ClipboardEventType::Copy,
            "cut" => return ClipboardEventType::Cut,
            "paste" => return ClipboardEventType::Paste,
            _ => {},
        );
        ClipboardEventType::Other(value.into())
    }
}

#[dom_struct]
pub(crate) struct ClipboardEvent {
    event: Event,
    #[no_trace]
    clipboard_event_type: ClipboardEventType,
    clipboard_data: MutNullableDom<DataTransfer>,
}

impl ClipboardEvent {
    fn new_inherited(
        clipboard_event_type: ClipboardEventType,
        clipboard_data: Option<&DataTransfer>,
    ) -> ClipboardEvent {
        ClipboardEvent {
            event: Event::new_inherited(),
            clipboard_event_type,
            clipboard_data: MutNullableDom::new(clipboard_data),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        clipboard_event_type: ClipboardEventType,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        clipboard_data: Option<&DataTransfer>,
    ) -> DomRoot<ClipboardEvent> {
        let event_type = clipboard_event_type.as_atom();
        let event = reflect_dom_object_with_proto(
            cx,
            Box::new(ClipboardEvent::new_inherited(
                clipboard_event_type,
                clipboard_data,
            )),
            window,
            proto,
        );
        event.upcast::<Event>().init_event(
            event_type,
            bool::from(can_bubble),
            bool::from(cancelable),
        );
        event
    }

    pub(crate) fn clipboard_event_type(&self) -> &ClipboardEventType {
        &self.clipboard_event_type
    }

    pub(crate) fn set_clipboard_data(&self, clipboard_data: Option<&DataTransfer>) {
        self.clipboard_data.set(clipboard_data);
    }

    pub(crate) fn clipboard_data(&self) -> Option<DomRoot<DataTransfer>> {
        self.clipboard_data.get()
    }

    /// Returns the text content of this [`ClipboardEvent`]'s [`DataTransfer`] object if
    /// any exists.
    pub(crate) fn text_content(&self) -> Option<String> {
        self.clipboard_data()?
            .data_store()?
            .iter_item_list()
            .find_map(|item| match item {
                Kind::Text { data, .. } if !data.is_empty() => Some(data.to_string()),
                _ => None,
            })
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
