/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::UIEventBinding;
use dom::bindings::codegen::InheritTypes::{UIEventBase, UIEventDerived};
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, Fallible};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object2};
use dom::node::{AbstractNode, ScriptView};
use dom::event::{Event, EventTypeId, UIEventTypeId};
use dom::window::Window;
use dom::windowproxy::WindowProxy;

pub struct UIEvent {
    parent: Event,
    view: Option<@mut WindowProxy>,
    detail: i32
}

impl UIEventDerived for Event {
    fn is_uievent(&self) -> bool {
        self.type_id == UIEventTypeId
    }
}

impl UIEvent {
    pub fn from<T: UIEventBase>(derived: JSManaged<T>) -> JSManaged<UIEvent> {
        derived.transmute()
    }

    pub fn to<T: UIEventDerived>(base: JSManaged<T>) -> JSManaged<UIEvent> {
        assert!(base.value().is_uievent());
        base.transmute()
    }
}

impl UIEvent {
    pub fn new_inherited(type_id: EventTypeId) -> UIEvent {
        UIEvent {
            parent: Event::new_inherited(type_id),
            view: None,
            detail: 0
        }
    }

    pub fn new(window: @mut Window) -> JSManaged<UIEvent> {
        reflect_dom_object2(~UIEvent::new_inherited(UIEventTypeId),
                            window,
                            UIEventBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window,
                       type_: DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<JSManaged<UIEvent>> {
        let mut ev = UIEvent::new(owner);
        ev.mut_value().InitUIEvent(type_, init.parent.bubbles, init.parent.cancelable,
                                   init.view, init.detail);
        Ok(ev)
    }

    pub fn GetView(&self) -> Option<@mut WindowProxy> {
        self.view
    }

    pub fn Detail(&self) -> i32 {
        self.detail
    }

    pub fn InitUIEvent(&mut self,
                       type_: DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       view: Option<@mut WindowProxy>,
                       detail: i32) {
        self.parent.InitEvent(type_, can_bubble, cancelable);
        self.view = view;
        self.detail = detail;
    }

    pub fn LayerX(&self) -> i32 {
        //TODO
        0
    }

    pub fn LayerY(&self) -> i32 {
        //TODO
        0
    }

    pub fn PageX(&self) -> i32 {
        //TODO
        0
    }

    pub fn PageY(&self) -> i32 {
        //TODO
        0
    }

    pub fn Which(&self) -> u32 {
        //TODO
        0
    }

    pub fn GetRangeParent(&self) -> Option<AbstractNode<ScriptView>> {
        //TODO
        None
    }

    pub fn RangeOffset(&self) -> i32 {
        //TODO
        0
    }

    pub fn CancelBubble(&self) -> bool {
        //TODO
        false
    }

    pub fn SetCancelBubble(&mut self, _val: bool) {
        //TODO
    }

    pub fn IsChar(&self) -> bool {
        //TODO
        false
    }
}

impl Reflectable for UIEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.parent.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.parent.mut_reflector()
    }
}
