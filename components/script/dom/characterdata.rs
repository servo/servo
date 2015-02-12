/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataDerived, NodeCast};
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::error::Error::IndexSize;
use dom::bindings::js::JSRef;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId};

use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::Ref;

#[dom_struct]
pub struct CharacterData {
    node: Node,
    data: DOMRefCell<DOMString>,
}

impl CharacterDataDerived for EventTarget {
    fn is_characterdata(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(NodeTypeId::Text) |
            EventTargetTypeId::Node(NodeTypeId::Comment) |
            EventTargetTypeId::Node(NodeTypeId::ProcessingInstruction) => true,
            _ => false
        }
    }
}

impl CharacterData {
    pub fn new_inherited(id: NodeTypeId, data: DOMString, document: JSRef<Document>) -> CharacterData {
        CharacterData {
            node: Node::new_inherited(id, document),
            data: DOMRefCell::new(data),
        }
    }

    #[inline]
    pub fn node<'a>(&'a self) -> &'a Node {
        &self.node
    }

    #[inline]
    pub fn data(&self) -> Ref<DOMString> {
        self.data.borrow()
    }

    #[inline]
    pub fn set_data(&self, data: DOMString) {
        *self.data.borrow_mut() = data;
    }

    #[inline]
    pub unsafe fn data_for_layout<'a>(&'a self) -> &'a str {
        self.data.borrow_for_layout().as_slice()
    }

}

impl<'a> CharacterDataMethods for JSRef<'a, CharacterData> {
    fn Data(self) -> DOMString {
        self.data.borrow().clone()
    }

    fn SetData(self, arg: DOMString) -> ErrorResult {
        *self.data.borrow_mut() = arg;
        Ok(())
    }

    fn Length(self) -> u32 {
        self.data.borrow().len() as u32
    }

    fn SubstringData(self, offset: u32, count: u32) -> Fallible<DOMString> {
        Ok(self.data.borrow()[offset as uint .. count as uint].to_owned())
    }

    fn AppendData(self, arg: DOMString) -> ErrorResult {
        self.data.borrow_mut().push_str(arg.as_slice());
        Ok(())
    }

    fn InsertData(self, offset: u32, arg: DOMString) -> ErrorResult {
        self.ReplaceData(offset, 0, arg)
    }

    fn DeleteData(self, offset: u32, count: u32) -> ErrorResult {
        self.ReplaceData(offset, count, "".to_owned())
    }

    fn ReplaceData(self, offset: u32, count: u32, arg: DOMString) -> ErrorResult {
        let length = self.data.borrow().len() as u32;
        if offset > length {
            return Err(IndexSize);
        }
        let count = if offset + count > length {
            length - offset
        } else {
            count
        };
        let mut data = self.data.borrow()[..offset as uint].to_owned();
        data.push_str(arg.as_slice());
        data.push_str(&self.data.borrow()[(offset + count) as uint..]);
        *self.data.borrow_mut() = data;
        // FIXME: Once we have `Range`, we should implement step7 to step11
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(self) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.remove_self();
    }
}

