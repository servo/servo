/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::codegen::InheritTypes::{CharacterDataDerived, ElementCast};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::Error::IndexSize;
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::js::{LayoutJS, Root};
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeTypeId};

use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::Ref;

// https://dom.spec.whatwg.org/#characterdata
#[dom_struct]
pub struct CharacterData {
    node: Node,
    data: DOMRefCell<DOMString>,
}

impl CharacterDataDerived for EventTarget {
    fn is_characterdata(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(NodeTypeId::CharacterData(_)) => true,
            _ => false
        }
    }
}

impl CharacterData {
    pub fn new_inherited(id: CharacterDataTypeId, data: DOMString, document: &Document) -> CharacterData {
        CharacterData {
            node: Node::new_inherited(NodeTypeId::CharacterData(id), document),
            data: DOMRefCell::new(data),
        }
    }
}

impl CharacterDataMethods for CharacterData {
    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn Data(&self) -> DOMString {
        self.data.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn SetData(&self, data: DOMString) {
        *self.data.borrow_mut() = data;
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-length
    fn Length(&self) -> u32 {
        self.data.borrow().chars().map(|c| c.len_utf16()).sum::<usize>() as u32
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-substringdata
    fn SubstringData(&self, offset: u32, count: u32) -> Fallible<DOMString> {
        let data = self.data.borrow();
        // Step 1.
        let data_from_offset = match find_utf16_code_unit_offset(&data, offset) {
            Some(offset_bytes) => &data[offset_bytes..],
            // Step 2.
            None => return Err(IndexSize)
        };
        let substring = match find_utf16_code_unit_offset(data_from_offset, count) {
            // Steps 3.
            None => data_from_offset,
            // Steps 4.
            Some(count_bytes) => &data_from_offset[..count_bytes],
        };
        Ok(substring.to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-appenddatadata
    fn AppendData(&self, data: DOMString) {
        self.append_data(&*data);
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-insertdataoffset-data
    fn InsertData(&self, offset: u32, arg: DOMString) -> ErrorResult {
        self.ReplaceData(offset, 0, arg)
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-deletedataoffset-count
    fn DeleteData(&self, offset: u32, count: u32) -> ErrorResult {
        self.ReplaceData(offset, count, "".to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-replacedata
    fn ReplaceData(&self, offset: u32, count: u32, arg: DOMString) -> ErrorResult {
        let new_data = {
            let data = self.data.borrow();
            let (prefix, data_from_offset) = match find_utf16_code_unit_offset(&data, offset) {
                Some(offset_bytes) => data.split_at(offset_bytes),
                // Step 2.
                None => return Err(IndexSize)
            };
            let suffix = match find_utf16_code_unit_offset(data_from_offset, count) {
                // Steps 3.
                None => "",
                Some(count_bytes) => &data_from_offset[count_bytes..],
            };
            // Step 4: Mutation observers.
            // Step 5 to 7.
            let mut new_data = String::with_capacity(prefix.len() + arg.len() + suffix.len());
            new_data.push_str(prefix);
            new_data.push_str(&arg);
            new_data.push_str(suffix);
            new_data
        };
        *self.data.borrow_mut() = new_data;
        // FIXME: Once we have `Range`, we should implement step 8 to step 11
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        let node = NodeCast::from_ref(self);
        node.remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).preceding_siblings()
                                .filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).following_siblings()
                                .filter_map(ElementCast::to_root).next()
    }
}

/// The different types of CharacterData.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CharacterDataTypeId {
    Comment,
    Text,
    ProcessingInstruction,
}


impl CharacterData {
    #[inline]
    pub fn data(&self) -> Ref<DOMString> {
        self.data.borrow()
    }
    #[inline]
    pub fn append_data(&self, data: &str) {
        self.data.borrow_mut().push_str(data)
    }
}

#[allow(unsafe_code)]
pub trait LayoutCharacterDataHelpers {
    unsafe fn data_for_layout(&self) -> &str;
}

#[allow(unsafe_code)]
impl LayoutCharacterDataHelpers for LayoutJS<CharacterData> {
    #[inline]
    unsafe fn data_for_layout(&self) -> &str {
        &(*self.unsafe_get()).data.borrow_for_layout()
    }
}

/// Given a number of UTF-16 code units from the start of the given string,
/// return the corresponding number of UTF-8 bytes.
///
/// s[find_utf16_code_unit_offset(s, o).unwrap()..] == s.to_utf16()[o..].to_utf8()
fn find_utf16_code_unit_offset(s: &str, offset: u32) -> Option<usize> {
    let mut code_units = 0;
    for (i, c) in s.char_indices() {
        if code_units == offset {
            return Some(i)
        }
        code_units += 1;
        if c > '\u{FFFF}' {
            if code_units == offset {
                panic!("\n\n\
                    Would split a surrogate pair in CharacterData API.\n\
                    If you see this in real content, please comment with the URL\n\
                    on https://github.com/servo/servo/issues/6873\n\
                \n");
            }
            code_units += 1;
        }
    }
    if code_units == offset {
        Some(s.len())
    } else {
        None
    }
}
