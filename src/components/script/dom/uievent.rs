/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::UIEventBinding;
use dom::bindings::codegen::InheritTypes::{EventCast, UIEventDerived};
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, OptionalSettable};
use dom::bindings::error::Fallible;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventMethods, EventTypeId, UIEventTypeId};
use dom::node::Node;
use dom::window::Window;
use servo_util::str::DOMString;

use serialize::{Encoder, Encodable};

#[deriving(Encodable)]
pub struct UIEvent {
    pub event: Event,
    pub view: Option<JS<Window>>,
    pub detail: i32
}

impl UIEventDerived for Event {
    fn is_uievent(&self) -> bool {
        self.type_id == UIEventTypeId
    }
}

impl UIEvent {
    pub fn new_inherited(type_id: EventTypeId) -> UIEvent {
        UIEvent {
            event: Event::new_inherited(type_id),
            view: None,
            detail: 0
        }
    }

    pub fn new_uninitialized(window: &JSRef<Window>) -> Temporary<UIEvent> {
        reflect_dom_object(~UIEvent::new_inherited(UIEventTypeId),
                           window,
                           UIEventBinding::Wrap)
    }

    pub fn new(window: &JSRef<Window>,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
               view: Option<JSRef<Window>>,
               detail: i32) -> Temporary<UIEvent> {
        let mut ev = UIEvent::new_uninitialized(window).root();
        ev.InitUIEvent(type_, can_bubble, cancelable, view, detail);
        Temporary::from_rooted(&*ev)
    }

    pub fn Constructor(owner: &JSRef<Window>,
                       type_: DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<Temporary<UIEvent>> {
        let event = UIEvent::new(owner, type_,
                                 init.parent.bubbles, init.parent.cancelable,
                                 init.view.root_ref(), init.detail);
        Ok(event)
    }
}

pub trait UIEventMethods {
    fn GetView(&self) -> Option<Temporary<Window>>;
    fn Detail(&self) -> i32;
    fn LayerX(&self) -> i32;
    fn LayerY(&self) -> i32;
    fn PageX(&self) -> i32;
    fn PageY(&self) -> i32;
    fn Which(&self) -> u32;
    fn GetRangeParent(&self) -> Option<Temporary<Node>>;
    fn RangeOffset(&self) -> i32;
    fn CancelBubble(&self) -> bool;
    fn SetCancelBubble(&mut self, _val: bool);
    fn IsChar(&self) -> bool;
    fn InitUIEvent(&mut self,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
                   view: Option<JSRef<Window>>,
                   detail: i32);
}

impl<'a> UIEventMethods for JSRef<'a, UIEvent> {
    fn GetView(&self) -> Option<Temporary<Window>> {
        self.view.clone().map(|view| Temporary::new(view))
    }

    fn Detail(&self) -> i32 {
        self.detail
    }

    fn InitUIEvent(&mut self,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
                   view: Option<JSRef<Window>>,
                   detail: i32) {
        {
            let event: &mut JSRef<Event> = EventCast::from_mut_ref(self);
            event.InitEvent(type_, can_bubble, cancelable);
        }
        self.view.assign(view);
        self.detail = detail;
    }

    fn LayerX(&self) -> i32 {
        //TODO
        0
    }

    fn LayerY(&self) -> i32 {
        //TODO
        0
    }

    fn PageX(&self) -> i32 {
        //TODO
        0
    }

    fn PageY(&self) -> i32 {
        //TODO
        0
    }

    fn Which(&self) -> u32 {
        //TODO
        0
    }

    fn GetRangeParent(&self) -> Option<Temporary<Node>> {
        //TODO
        None
    }

    fn RangeOffset(&self) -> i32 {
        //TODO
        0
    }

    fn CancelBubble(&self) -> bool {
        //TODO
        false
    }

    fn SetCancelBubble(&mut self, _val: bool) {
        //TODO
    }

    fn IsChar(&self) -> bool {
        //TODO
        false
    }
}

impl Reflectable for UIEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.event.mut_reflector()
    }
}
