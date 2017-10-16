/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CompositionEventBinding::{self, CompositionEventMethods};
use dom::bindings::codegen::Bindings::UIEventBinding::UIEventBinding::UIEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{DomRoot, RootedReference};
use dom::bindings::str::DOMString;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct CompositionEvent {
    uievent: UIEvent,
    data: DOMString,
}

impl CompositionEvent {
    pub fn new(window: &Window,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
               view: Option<&Window>,
               detail: i32,
               data: DOMString) -> DomRoot<CompositionEvent> {
        let ev = reflect_dom_object(Box::new(CompositionEvent {
                                        uievent: UIEvent::new_inherited(),
                                        data: data,
                                    }),
                                    window,
                                    CompositionEventBinding::Wrap);
        ev.uievent.InitUIEvent(type_, can_bubble, cancelable, view, detail);
        ev
    }

    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &CompositionEventBinding::CompositionEventInit)
                       -> Fallible<DomRoot<CompositionEvent>> {
        let event = CompositionEvent::new(window,
                                    type_,
                                    init.parent.parent.bubbles,
                                    init.parent.parent.cancelable,
                                    init.parent.view.r(),
                                    init.parent.detail,
                                    init.data.clone());
        Ok(event)
    }
}

impl CompositionEventMethods for CompositionEvent {
    // https://w3c.github.io/uievents/#dom-compositionevent-data
    fn Data(&self) -> DOMString {
        self.data.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
