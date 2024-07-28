/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::ClipboardEventBinding::{
    ClipboardEventInit, ClipboardEventMethods,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::event::Event;
use crate::dom::window::Window;

#[dom_struct]
pub struct ClipboardEvent {
    event: Event,
}

impl ClipboardEvent {
    pub fn new_inherited() -> ClipboardEvent {
        todo!()
    }

    pub fn new() {}

    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &ClipboardEventInit,
    ) -> DomRoot<ClipboardEvent> {
        todo!()
    }
}

impl ClipboardEventMethods for ClipboardEvent {
    // https://www.w3.org/TR/clipboard-apis/#dom-clipboardevent-clipboarddata
    fn GetClipboardData(&self) -> Option<DomRoot<DataTransfer>> {
        todo!()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        todo!()
    }
}
