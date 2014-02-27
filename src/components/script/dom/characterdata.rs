/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::codegen::InheritTypes::CharacterDataDerived;
use dom::bindings::js::JS;
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::{CommentNodeTypeId, Node, NodeTypeId, TextNodeTypeId, ProcessingInstructionNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct CharacterData {
    node: Node,
    data: DOMString,
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
    pub fn new_inherited(id: NodeTypeId, data: DOMString, document: JS<Document>) -> CharacterData {
        CharacterData {
            node: Node::new_inherited(id, document),
            data: data
        }
    }
    
    pub fn Data(&self) -> DOMString {
        self.data.clone()
    }

    pub fn SetData(&mut self, arg: DOMString) -> ErrorResult {
        self.data = arg;
        Ok(())
    }

    pub fn Length(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn SubstringData(&self, offset: u32, count: u32) -> Fallible<DOMString> {
        Ok(self.data.slice(offset as uint, count as uint).to_str())
    }

    pub fn AppendData(&mut self, arg: DOMString) -> ErrorResult {
        self.data.push_str(arg);
        Ok(())
    }

    pub fn InsertData(&mut self, _offset: u32, _arg: DOMString) -> ErrorResult {
        fail!("CharacterData::InsertData() is unimplemented")
    }

    pub fn DeleteData(&mut self, _offset: u32, _count: u32) -> ErrorResult {
        fail!("CharacterData::DeleteData() is unimplemented")
    }

    pub fn ReplaceData(&mut self, _offset: u32, _count: u32, _arg: DOMString) -> ErrorResult {
        fail!("CharacterData::ReplaceData() is unimplemented")
    }
}

impl Reflectable for CharacterData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }
}
