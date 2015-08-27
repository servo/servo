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

use util::str::{DOMString, slice_chars};

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

impl<'a> CharacterDataMethods for &'a CharacterData {
    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn Data(self) -> DOMString {
        self.data.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn SetData(self, data: DOMString) {
        *self.data.borrow_mut() = data;
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-length
    fn Length(self) -> u32 {
        self.data.borrow().chars().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-substringdataoffset-count
    fn SubstringData(self, offset: u32, count: u32) -> Fallible<DOMString> {
        let data = self.data.borrow();
        // Step 1.
        let length = data.chars().count() as u32;
        if offset > length {
            // Step 2.
            return Err(IndexSize);
        }
        // Steps 3-4.
        let end = if length - offset < count { length } else { offset + count };
        Ok(slice_chars(&*data, offset as usize, end as usize).to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-appenddatadata
    fn AppendData(self, data: DOMString) {
        self.append_data(&*data);
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-insertdataoffset-data
    fn InsertData(self, offset: u32, arg: DOMString) -> ErrorResult {
        self.ReplaceData(offset, 0, arg)
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-deletedataoffset-count
    fn DeleteData(self, offset: u32, count: u32) -> ErrorResult {
        self.ReplaceData(offset, count, "".to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-replacedataoffset-count-data
    fn ReplaceData(self, offset: u32, count: u32, arg: DOMString) -> ErrorResult {
        // Step 1.
        let length = self.data.borrow().chars().count() as u32;
        if offset > length {
            // Step 2.
            return Err(IndexSize);
        }
        // Step 3.
        let count = match length - offset {
            diff if diff < count => diff,
            _ => count,
        };
        // Step 4: Mutation observers.
        // Step 5.
        let mut data = slice_chars(&*self.data.borrow(), 0, offset as usize).to_owned();
        data.push_str(&arg);
        data.push_str(slice_chars(&*self.data.borrow(), (offset + count) as usize, length as usize));
        *self.data.borrow_mut() = data;
        // FIXME: Once we have `Range`, we should implement step7 to step11
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(self) {
        let node = NodeCast::from_ref(self);
        node.remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).preceding_siblings()
                                .filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).following_siblings()
                                .filter_map(ElementCast::to_root).next()
    }
}

/// The different types of CharacterData.
#[derive(JSTraceable, Copy, Clone, PartialEq, Debug, HeapSizeOf)]
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
    unsafe fn data_for_layout<'a>(&'a self) -> &'a str;
}

#[allow(unsafe_code)]
impl LayoutCharacterDataHelpers for LayoutJS<CharacterData> {
    #[inline]
    unsafe fn data_for_layout<'a>(&'a self) -> &'a str {
        &(*self.unsafe_get()).data.borrow_for_layout()
    }
}
