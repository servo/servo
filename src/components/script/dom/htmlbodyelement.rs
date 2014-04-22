/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLBodyElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBodyElementDerived;
use dom::bindings::error::ErrorResult;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLBodyElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::window::WindowMethods;
use js::jsapi::{JSContext, JSObject};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLBodyElement {
    pub htmlelement: HTMLElement
}

impl HTMLBodyElementDerived for EventTarget {
    fn is_htmlbodyelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLBodyElementTypeId))
    }
}

impl HTMLBodyElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLBodyElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLBodyElementBinding::Wrap)
    }
}

pub trait HTMLBodyElementMethods {
    fn Text(&self) -> DOMString;
    fn SetText(&mut self, _text: DOMString) -> ErrorResult;
    fn Link(&self) -> DOMString;
    fn SetLink(&self, _link: DOMString) -> ErrorResult;
    fn VLink(&self) -> DOMString;
    fn SetVLink(&self, _v_link: DOMString) -> ErrorResult;
    fn ALink(&self) -> DOMString;
    fn SetALink(&self, _a_link: DOMString) -> ErrorResult;
    fn BgColor(&self) -> DOMString;
    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult;
    fn Background(&self) -> DOMString;
    fn SetBackground(&self, _background: DOMString) -> ErrorResult;
    fn GetOnunload(&self, cx: *mut JSContext) -> *mut JSObject;
    fn SetOnunload(&mut self, cx: *mut JSContext, listener: *mut JSObject);
}

impl<'a> HTMLBodyElementMethods for JSRef<'a, HTMLBodyElement> {
    fn Text(&self) -> DOMString {
        "".to_owned()
    }

    fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Link(&self) -> DOMString {
        "".to_owned()
    }

    fn SetLink(&self, _link: DOMString) -> ErrorResult {
        Ok(())
    }

    fn VLink(&self) -> DOMString {
        "".to_owned()
    }

    fn SetVLink(&self, _v_link: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ALink(&self) -> DOMString {
        "".to_owned()
    }

    fn SetALink(&self, _a_link: DOMString) -> ErrorResult {
        Ok(())
    }

    fn BgColor(&self) -> DOMString {
        "".to_owned()
    }

    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Background(&self) -> DOMString {
        "".to_owned()
    }

    fn SetBackground(&self, _background: DOMString) -> ErrorResult {
        Ok(())
    }

    fn GetOnunload(&self, cx: *mut JSContext) -> *mut JSObject {
        let win = window_from_node(self).root();
        win.deref().GetOnunload(cx)
    }

    fn SetOnunload(&mut self, cx: *mut JSContext, listener: *mut JSObject) {
        let mut win = window_from_node(self).root();
        win.SetOnunload(cx, listener)
    }
}
