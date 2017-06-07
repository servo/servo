/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! DOM bindings for `CharacterData`.

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataTypeId, NodeTypeId};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::str::DOMString;
use dom::comment::Comment;
use dom::document::Document;
use dom::element::Element;
use dom::node::{ChildrenMutation, Node, NodeDamage};
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use dom::virtualmethods::vtable_for;
use dom_struct::dom_struct;
use servo_config::opts;
use std::cell::Ref;

// https://dom.spec.whatwg.org/#characterdata
#[dom_struct]
pub struct CharacterData {
    node: Node,
    data: DOMRefCell<DOMString>,
}

impl CharacterData {
    pub fn new_inherited(data: DOMString, document: &Document) -> CharacterData {
        CharacterData {
            node: Node::new_inherited(document),
            data: DOMRefCell::new(data),
        }
    }

    pub fn clone_with_data(&self, data: DOMString, document: &Document) -> Root<Node> {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => {
                Root::upcast(Comment::new(data, &document))
            }
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                let pi = self.downcast::<ProcessingInstruction>().unwrap();
                Root::upcast(ProcessingInstruction::new(pi.Target(), data, &document))
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                Root::upcast(Text::new(data, &document))
            },
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn data(&self) -> Ref<DOMString> {
        self.data.borrow()
    }

    #[inline]
    pub fn append_data(&self, data: &str) {
        self.data.borrow_mut().push_str(data);
        self.content_changed();
    }

    fn content_changed(&self) {
        let node = self.upcast::<Node>();
        node.dirty(NodeDamage::OtherNodeDamage);
    }
}

impl CharacterDataMethods for CharacterData {
    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn Data(&self) -> DOMString {
        self.data.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-data
    fn SetData(&self, data: DOMString) {
        let old_length = self.Length();
        let new_length = data.encode_utf16().count() as u32;
        *self.data.borrow_mut() = data;
        self.content_changed();
        let node = self.upcast::<Node>();
        node.ranges().replace_code_units(node, 0, old_length, new_length);

        // If this is a Text node, we might need to re-parse (say, if our parent
        // is a <style> element.) We don't need to if this is a Comment or
        // ProcessingInstruction.
        if let Some(_) = self.downcast::<Text>() {
            if let Some(parent_node) = node.GetParentNode() {
                let mutation = ChildrenMutation::ChangeText;
                vtable_for(&parent_node).children_changed(&mutation);
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-length
    fn Length(&self) -> u32 {
        self.data.borrow().encode_utf16().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-substringdata
    fn SubstringData(&self, offset: u32, count: u32) -> Fallible<DOMString> {
        let data = self.data.borrow();
        // Step 1.
        let mut substring = String::new();
        let remaining;
        match split_at_utf16_code_unit_offset(&data, offset) {
            Ok((_, astral, s)) => {
                // As if we had split the UTF-16 surrogate pair in half
                // and then transcoded that to UTF-8 lossily,
                // since our DOMString is currently strict UTF-8.
                if astral.is_some() {
                    substring = substring + "\u{FFFD}";
                }
                remaining = s;
            }
            // Step 2.
            Err(()) => return Err(Error::IndexSize),
        }
        match split_at_utf16_code_unit_offset(remaining, count) {
            // Steps 3.
            Err(()) => substring = substring + remaining,
            // Steps 4.
            Ok((s, astral, _)) => {
                substring = substring + s;
                // As if we had split the UTF-16 surrogate pair in half
                // and then transcoded that to UTF-8 lossily,
                // since our DOMString is currently strict UTF-8.
                if astral.is_some() {
                    substring = substring + "\u{FFFD}";
                }
            }
        };
        Ok(DOMString::from(substring))
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-appenddatadata
    fn AppendData(&self, data: DOMString) {
        // FIXME(ajeffrey): Efficient append on DOMStrings?
        self.append_data(&*data);
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-insertdataoffset-data
    fn InsertData(&self, offset: u32, arg: DOMString) -> ErrorResult {
        self.ReplaceData(offset, 0, arg)
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-deletedataoffset-count
    fn DeleteData(&self, offset: u32, count: u32) -> ErrorResult {
        self.ReplaceData(offset, count, DOMString::new())
    }

    // https://dom.spec.whatwg.org/#dom-characterdata-replacedata
    fn ReplaceData(&self, offset: u32, count: u32, arg: DOMString) -> ErrorResult {
        let mut new_data;
        {
            let data = self.data.borrow();
            let prefix;
            let replacement_before;
            let remaining;
            match split_at_utf16_code_unit_offset(&data, offset) {
                Ok((p, astral, r)) => {
                    prefix = p;
                    // As if we had split the UTF-16 surrogate pair in half
                    // and then transcoded that to UTF-8 lossily,
                    // since our DOMString is currently strict UTF-8.
                    replacement_before = if astral.is_some() { "\u{FFFD}" } else { "" };
                    remaining = r;
                }
                // Step 2.
                Err(()) => return Err(Error::IndexSize),
            };
            let replacement_after;
            let suffix;
            match split_at_utf16_code_unit_offset(remaining, count) {
                // Steps 3.
                Err(()) => {
                    replacement_after = "";
                    suffix = "";
                }
                Ok((_, astral, s)) => {
                    // As if we had split the UTF-16 surrogate pair in half
                    // and then transcoded that to UTF-8 lossily,
                    // since our DOMString is currently strict UTF-8.
                    replacement_after = if astral.is_some() { "\u{FFFD}" } else { "" };
                    suffix = s;
                }
            };
            // Step 4: Mutation observers.
            // Step 5 to 7.
            new_data = String::with_capacity(
                prefix.len() +
                replacement_before.len() +
                arg.len() +
                replacement_after.len() +
                suffix.len());
            new_data.push_str(prefix);
            new_data.push_str(replacement_before);
            new_data.push_str(&arg);
            new_data.push_str(replacement_after);
            new_data.push_str(suffix);
        }
        *self.data.borrow_mut() = DOMString::from(new_data);
        self.content_changed();
        // Steps 8-11.
        let node = self.upcast::<Node>();
        node.ranges().replace_code_units(
            node, offset, count, arg.encode_utf16().count() as u32);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        let node = self.upcast::<Node>();
        node.remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().preceding_siblings().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().following_siblings().filter_map(Root::downcast).next()
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

/// Split the given string at the given position measured in UTF-16 code units from the start.
///
/// * `Err(())` indicates that `offset` if after the end of the string
/// * `Ok((before, None, after))` indicates that `offset` is between Unicode code points.
///   The two string slices are such that:
///   `before == s.to_utf16()[..offset].to_utf8()` and
///   `after == s.to_utf16()[offset..].to_utf8()`
/// * `Ok((before, Some(ch), after))` indicates that `offset` is "in the middle"
///   of a single Unicode code point that would be represented in UTF-16 by a surrogate pair
///   of two 16-bit code units.
///   `ch` is that code point.
///   The two string slices are such that:
///   `before == s.to_utf16()[..offset - 1].to_utf8()` and
///   `after == s.to_utf16()[offset + 1..].to_utf8()`
///
/// # Panics
///
/// Note that the third variant is only ever returned when the `-Z replace-surrogates`
/// command-line option is specified.
/// When it *would* be returned but the option is *not* specified, this function panics.
fn split_at_utf16_code_unit_offset(s: &str, offset: u32) -> Result<(&str, Option<char>, &str), ()> {
    let mut code_units = 0;
    for (i, c) in s.char_indices() {
        if code_units == offset {
            let (a, b) = s.split_at(i);
            return Ok((a, None, b));
        }
        code_units += 1;
        if c > '\u{FFFF}' {
            if code_units == offset {
                if opts::get().replace_surrogates {
                    debug_assert!(c.len_utf8() == 4);
                    return Ok((&s[..i], Some(c), &s[i + c.len_utf8()..]))
                }
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
        Ok((s, None, ""))
    } else {
        Err(())
    }
}
