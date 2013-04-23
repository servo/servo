use dom::bindings::utils::{DOMString, null_string, str};
use dom::node::{Node, NodeTypeId};

use core::str;

pub struct CharacterData {
    parent: Node,
    data: DOMString
}

pub impl CharacterData {
    fn new(id: NodeTypeId, data: ~str) -> CharacterData {
        CharacterData {
            parent: Node::new(id),
            data: str(data)
        }
    }
    
    fn GetData(&self) -> DOMString {
        copy self.data
    }

    fn SetData(&mut self, arg: DOMString) {
        self.data = arg;
    }

    fn Length(&self) -> u32 {
        match self.data {
          str(ref s) => s.len() as u32,
          null_string => 0
        }
    }

    fn SubstringData(&self, offset: u32, count: u32) -> DOMString {
        match self.data {
          str(ref s) => str(s.slice(offset as uint, count as uint).to_str()),
          null_string => null_string
        }
    }

    fn AppendData(&mut self, arg: DOMString) {
        let s = self.data.to_str();
        self.data = str(str::append(s, arg.to_str()));
    }

    fn InsertData(&mut self, _offset: u32, _arg: DOMString) {
        fail!(~"nyi")
    }

    fn DeleteData(&mut self, _offset: u32, _count: u32) {
        fail!(~"nyi")
    }

    fn ReplaceData(&mut self, _offset: u32, _count: u32, _arg: DOMString) {
        fail!(~"nyi")
    }
}
