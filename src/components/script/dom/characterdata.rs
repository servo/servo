/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::AbstractDocument;
use dom::node::{Node, NodeTypeId};

pub struct CharacterData {
    node: Node,
    data: DOMString,
}

impl CharacterData {
    pub fn new_inherited(id: NodeTypeId, data: DOMString, document: AbstractDocument) -> CharacterData {
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
