/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataDerived, NodeCast};
use dom::bindings::error::{Fallible, ErrorResult, IndexSize};
use dom::bindings::js::JSRef;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{CommentNodeTypeId, Node, NodeTypeId, TextNodeTypeId, ProcessingInstructionNodeTypeId, NodeHelpers};
use servo_util::str::DOMString;

use std::cell::RefCell;

#[jstraceable]
#[must_root]
pub struct CharacterData {
    pub node: Node,
    pub data: RefCell<DOMString>,
}

impl CharacterDataDerived for EventTarget {
    fn is_characterdata(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(TextNodeTypeId) |
            NodeTargetTypeId(CommentNodeTypeId) |
            NodeTargetTypeId(ProcessingInstructionNodeTypeId) => true,
            _ => false
        }
    }
}

impl CharacterData {
    pub fn new_inherited(id: NodeTypeId, data: DOMString, document: JSRef<Document>) -> CharacterData {
        CharacterData {
            node: Node::new_inherited(id, document),
            data: RefCell::new(data),
        }
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
        Ok(self.data.borrow().as_slice().slice(offset as uint, count as uint).to_string())
    }

    fn AppendData(self, arg: DOMString) -> ErrorResult {
        self.data.borrow_mut().push_str(arg.as_slice());
        Ok(())
    }

    fn InsertData(self, offset: u32, arg: DOMString) -> ErrorResult {
        self.ReplaceData(offset, 0, arg)
    }

    fn DeleteData(self, offset: u32, count: u32) -> ErrorResult {
        self.ReplaceData(offset, count, "".to_string())
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
        let mut data = self.data.borrow().as_slice().slice(0, offset as uint).to_string();
        data.push_str(arg.as_slice());
        data.push_str(self.data.borrow().as_slice().slice((offset + count) as uint, length as uint));
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

impl Reflectable for CharacterData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }
}
