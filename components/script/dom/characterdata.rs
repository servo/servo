/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataDerived, ElementCast};
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::error::Error::IndexSize;
use dom::bindings::js::{JSRef, LayoutJS, Temporary};
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId};

use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::Ref;
use std::cmp;

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
    pub fn new_inherited(id: CharacterDataTypeId, data: DOMString, document: JSRef<Document>) -> CharacterData {
        CharacterData {
            node: Node::new_inherited(NodeTypeId::CharacterData(id), document),
            data: DOMRefCell::new(data),
        }
    }
}

impl<'a> CharacterDataMethods for JSRef<'a, CharacterData> {
    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn Data(self) -> DOMString {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let data = self.data.borrow();
        data.clone()
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn SetData(self, data: DOMString) {
        *self.data.borrow_mut() = data;
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-length
    fn Length(self) -> u32 {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let data = self.data.borrow();
        data.chars().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-substringdata
    fn SubstringData(self, offset: u32, count: u32) -> Fallible<DOMString> {
        let data = self.data.borrow();
        // Step 1.
        let len = data.chars().count();
        if offset as usize > len {
            // Step 2.
            return Err(IndexSize);
        }
        // Step 3.
        let end = cmp::min((offset + count) as usize, len);
        // Step 4.
        Ok(data.slice_chars(offset as usize, end).to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-appenddata
    fn AppendData(self, data: DOMString) {
        self.data.borrow_mut().push_str(&data);
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-insertdata
    fn InsertData(self, offset: u32, arg: DOMString) -> ErrorResult {
        self.ReplaceData(offset, 0, arg)
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-deletedata
    fn DeleteData(self, offset: u32, count: u32) -> ErrorResult {
        self.ReplaceData(offset, count, "".to_owned())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-replacedata
    fn ReplaceData(self, offset: u32, count: u32, arg: DOMString) -> ErrorResult {
        let length = self.data.borrow().chars().count() as u32;
        if offset > length {
            return Err(IndexSize);
        }
        let count = if offset + count > length {
            length - offset
        } else {
            count
        };
        let mut data = self.data.borrow().slice_chars(0, offset as usize).to_owned();
        data.push_str(&arg);
        data.push_str(&self.data.borrow().slice_chars((offset + count) as usize, length as usize));
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
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).preceding_siblings()
                                .filter_map(ElementCast::to_temporary).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).following_siblings()
                                .filter_map(ElementCast::to_temporary).next()
    }
}

/// The different types of CharacterData.
#[derive(Copy, Clone, PartialEq, Debug)]
#[jstraceable]
pub enum CharacterDataTypeId {
    Comment,
    Text,
    ProcessingInstruction,
}

pub trait CharacterDataHelpers<'a> {
    fn data(self) -> Ref<'a, DOMString>;
}

impl<'a> CharacterDataHelpers<'a> for JSRef<'a, CharacterData> {
    #[inline]
    fn data(self) -> Ref<'a, DOMString> {
        self.extended_deref().data.borrow()
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
