/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::UIEventBinding;
use dom::bindings::utils::{DOMString, Fallible};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::node::{AbstractNode, ScriptView};
use dom::event::Event;
use dom::window::Window;
use dom::windowproxy::WindowProxy;

use js::jsapi::{JSObject, JSContext};

pub struct UIEvent {
    parent: Event,
    can_bubble: bool,
    cancelable: bool,
    view: Option<@mut WindowProxy>,
    detail: i32
}

impl UIEvent {
    pub fn new_inherited(type_: &DOMString, can_bubble: bool, cancelable: bool,
                         view: Option<@mut WindowProxy>, detail: i32) -> UIEvent {
        UIEvent {
            parent: Event::new_inherited(type_),
            can_bubble: can_bubble,
            cancelable: cancelable,
            view: view,
            detail: detail
        }
    }

    pub fn new(window: @mut Window, type_: &DOMString, can_bubble: bool, cancelable: bool,
               view: Option<@mut WindowProxy>, detail: i32) -> @mut UIEvent {
        reflect_dom_object(@mut UIEvent::new_inherited(type_, can_bubble, cancelable, view, detail),
                           window,
                           UIEventBinding::Wrap)
    }

    pub fn Constructor(owner: @mut Window,
                       type_: &DOMString,
                       init: &UIEventBinding::UIEventInit) -> Fallible<@mut UIEvent> {
        Ok(UIEvent::new(owner, type_, init.parent.bubbles, init.parent.cancelable,
                        init.view, init.detail))
    }

    pub fn GetView(&self) -> Option<@mut WindowProxy> {
        self.view
    }

    pub fn Detail(&self) -> i32 {
        self.detail
    }

    pub fn InitUIEvent(&mut self,
                       type_: &DOMString,
                       can_bubble: bool,
                       cancelable: bool,
                       view: Option<@mut WindowProxy>,
                       detail: i32) {
        self.parent.InitEvent(type_, can_bubble, cancelable);
        self.can_bubble = can_bubble;
        self.cancelable = cancelable;
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

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        unreachable!()
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.parent.GetParentObject(cx)
    }
}
