/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::utils::{DOMString, null_string, str};
use dom::node::{Node, NodeTypeId};

use core::str;

pub struct CharacterData {
    parent: Node,
    data: DOMString
}

impl CharacterData {
    pub fn new(id: NodeTypeId, data: ~str) -> CharacterData {
        CharacterData {
            parent: Node::new(id),
            data: str(data)
        }
    }
    
    pub fn GetData(&self) -> DOMString {
        copy self.data
    }

    pub fn SetData(&mut self, arg: DOMString) {
        self.data = arg;
    }

    pub fn Length(&self) -> u32 {
        match self.data {
            str(ref s) => s.len() as u32,
            null_string => 0
        }
    }

    pub fn SubstringData(&self, offset: u32, count: u32) -> DOMString {
        match self.data {
            str(ref s) => str(s.slice(offset as uint, count as uint).to_str()),
            null_string => null_string
        }
    }

    pub fn AppendData(&mut self, arg: DOMString) {
        let s = self.data.to_str();
        self.data = str(str::append(s, arg.to_str()));
    }

    pub fn InsertData(&mut self, _offset: u32, _arg: DOMString) {
        fail!("CharacterData::InsertData() is unimplemented")
    }

    pub fn DeleteData(&mut self, _offset: u32, _count: u32) {
        fail!("CharacterData::DeleteData() is unimplemented")
    }

    pub fn ReplaceData(&mut self, _offset: u32, _count: u32, _arg: DOMString) {
        fail!("CharacterData::ReplaceData() is unimplemented")
    }
}

