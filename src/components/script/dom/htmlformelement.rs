/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLFormElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFormElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLFormElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, Static};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLFormElement {
    pub htmlelement: HTMLElement
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFormElementTypeId))
    }
}

impl HTMLFormElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLFormElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLFormElementBinding::Wrap)
    }
}

pub trait HTMLFormElementMethods {
    fn AcceptCharset(&self) -> DOMString;
    fn SetAcceptCharset(&mut self, _accept_charset: DOMString) -> ErrorResult;
    fn Action(&self) -> DOMString;
    fn SetAction(&mut self, _action: DOMString) -> ErrorResult;
    fn Autocomplete(&self) -> DOMString;
    fn SetAutocomplete(&mut self, _autocomplete: DOMString) -> ErrorResult;
    fn Enctype(&self) -> DOMString;
    fn SetEnctype(&mut self, _enctype: DOMString) -> ErrorResult;
    fn Encoding(&self) -> DOMString;
    fn SetEncoding(&mut self, _encoding: DOMString) -> ErrorResult;
    fn Method(&self) -> DOMString;
    fn SetMethod(&mut self, _method: DOMString) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn NoValidate(&self) -> bool;
    fn SetNoValidate(&mut self, _no_validate: bool) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&mut self, _target: DOMString) -> ErrorResult;
    fn Elements(&self) -> Temporary<HTMLCollection>;
    fn Length(&self) -> i32;
    fn Submit(&self) -> ErrorResult;
    fn Reset(&self);
    fn CheckValidity(&self) -> bool;
    fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Temporary<Element>;
}

impl<'a> HTMLFormElementMethods for JSRef<'a, HTMLFormElement> {
    fn AcceptCharset(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAcceptCharset(&mut self, _accept_charset: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Action(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAction(&mut self, _action: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Autocomplete(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAutocomplete(&mut self, _autocomplete: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Enctype(&self) -> DOMString {
        "".to_owned()
    }

    fn SetEnctype(&mut self, _enctype: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Encoding(&self) -> DOMString {
        "".to_owned()
    }

    fn SetEncoding(&mut self, _encoding: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Method(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMethod(&mut self, _method: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn NoValidate(&self) -> bool {
        false
    }

    fn SetNoValidate(&mut self, _no_validate: bool) -> ErrorResult {
        Ok(())
    }

    fn Target(&self) -> DOMString {
        "".to_owned()
    }

    fn SetTarget(&mut self, _target: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Elements(&self) -> Temporary<HTMLCollection> {
        // FIXME: https://github.com/mozilla/servo/issues/1844
        let window = window_from_node(self).root();
        HTMLCollection::new(&*window, Static(vec!()))
    }

    fn Length(&self) -> i32 {
        0
    }

    fn Submit(&self) -> ErrorResult {
        Ok(())
    }

    fn Reset(&self) {
    }

    fn CheckValidity(&self) -> bool {
        false
    }

    fn IndexedGetter(&self, _index: u32, _found: &mut bool) -> Temporary<Element> {
        fail!("Not implemented.")
    }
}
