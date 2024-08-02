/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::ClipboardEventBinding::{
    ClipboardEventInit, ClipboardEventMethods,
};
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::event::Event;
use crate::dom::window::Window;

#[dom_struct]
pub struct ClipboardEvent {
    event: Event,
    clipboardData: MutNullableDom<DataTransfer>,
}

impl ClipboardEvent {
    pub fn new_inherited() -> ClipboardEvent {
        ClipboardEvent {
            event: Event::new_inherited(),
            clipboardData: MutNullableDom::new(None),
        }
    }

    pub fn new(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        clipboardData: Option<&DataTransfer>,
    ) -> DomRoot<ClipboardEvent> {
        let ev =
            reflect_dom_object_with_proto(Box::new(ClipboardEvent::new_inherited()), window, proto);
        ev.upcast::<Event>().InitEvent(type_, true, true);
        ev.clipboardData.set(clipboardData);
        ev
    }

    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &ClipboardEventInit,
    ) -> DomRoot<ClipboardEvent> {
        ClipboardEvent::new(window, proto, type_, init.clipboardData.as_deref())
    }
}

impl ClipboardEventMethods for ClipboardEvent {
    // https://www.w3.org/TR/clipboard-apis/#dom-clipboardevent-clipboarddata
    fn GetClipboardData(&self) -> Option<DomRoot<DataTransfer>> {
        self.clipboardData.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
