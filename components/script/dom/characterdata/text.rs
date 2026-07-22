/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::TextBinding::TextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::html::htmlslotelement::{HTMLSlotElement, Slottable};
use crate::dom::node::Node;
use crate::dom::window::Window;

/// An HTML text node.
#[dom_struct]
pub(crate) struct Text {
    characterdata: CharacterData,
}

impl Text {
    pub(crate) fn new_inherited(text: DOMString, document: &Document) -> Text {
        Text {
            characterdata: CharacterData::new_inherited(text, document),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        text: DOMString,
        document: &Document,
    ) -> DomRoot<Text> {
        Self::new_with_proto(cx, text, document, None)
    }

    fn new_with_proto(
        cx: &mut js::context::JSContext,
        text: DOMString,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<Text> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(Text::new_inherited(text, document)),
            document,
            proto,
        )
    }
}

impl TextMethods<crate::DomTypeHolder> for Text {
    /// <https://dom.spec.whatwg.org/#dom-text-text>
    fn Constructor(
        cx: &mut js::context::JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        text: DOMString,
    ) -> Fallible<DomRoot<Text>> {
        let document = window.Document();
        Ok(Text::new_with_proto(cx, text, &document, proto))
    }

    // https://dom.spec.whatwg.org/#dom-text-splittext
    /// <https://dom.spec.whatwg.org/#concept-text-split>
    fn SplitText(&self, cx: &mut JSContext, offset: u32) -> Fallible<DomRoot<Text>> {
        let cdata = self.upcast::<CharacterData>();
        // Step 1.
        let length = cdata.Length();
        if offset > length {
            // Step 2.
            return Err(Error::IndexSize(None));
        }
        // Step 3.
        let count = length - offset;
        // Step 4.
        let new_data = cdata.SubstringData(offset, count).unwrap();
        // Step 5.
        let node = self.upcast::<Node>();
        let owner_doc = node.owner_doc();
        let new_node = owner_doc.CreateTextNode(cx, new_data);
        // Step 6.
        let parent = node.GetParentNode();
        if let Some(ref parent) = parent {
            // Step 7.1.
            parent
                .InsertBefore(cx, new_node.upcast(), node.GetNextSibling().as_deref())
                .unwrap();

            // Step 7.2. For each live range whose start node is node and start offset is
            // greater than offset, set its start node to newNode and decrease its start
            // offset by offset.
            //
            // Step 7.3. For each live range whose end node is node and end offset is
            // greater than offset, set its end node to newNode and decrease its end
            // offset by offset.
            if let Some(weak_ranges) = node.weak_ranges_mut() {
                weak_ranges.move_to_following_text_sibling_above(node, offset, new_node.upcast());
            }

            // Step 7.4. For each live range whose start node is parent and start offset
            // is equal to the index of node plus 1, increase its start offset by 1.
            //
            // Step 7.5. For each live range whose end node is parent and end offset is
            // equal to the index of node plus 1, increase its end offset by 1.
            if let Some(parent_weak_ranges) = parent.weak_ranges_mut() {
                parent_weak_ranges.increment_at(parent, node.index() + 1);
            }
        }
        // Step 8.
        cdata.DeleteData(cx, offset, count).unwrap();
        // Step 9.
        Ok(new_node)
    }

    /// <https://dom.spec.whatwg.org/#dom-text-wholetext>
    fn WholeText(&self, cx: &JSContext) -> DOMString {
        let first = self
            .upcast::<Node>()
            .inclusively_preceding_siblings_unrooted(cx.no_gc())
            .take_while(|node| node.is::<Text>())
            .last()
            .unwrap();
        let nodes = first
            .inclusively_following_siblings_unrooted(cx.no_gc())
            .take_while(|node| node.is::<Text>());
        let mut text = String::new();
        for ref node in nodes {
            let cdata = node.downcast::<CharacterData>().unwrap();
            text.push_str(&cdata.data());
        }
        DOMString::from(text)
    }

    /// <https://dom.spec.whatwg.org/#dom-slotable-assignedslot>
    fn GetAssignedSlot(&self, cx: &JSContext) -> Option<DomRoot<HTMLSlotElement>> {
        // > The assignedSlot getter steps are to return the result of
        // > find a slot given this and with the open flag set.
        rooted!(&in(cx) let slottable = Slottable(Dom::from_ref(self.upcast::<Node>())));
        slottable.find_a_slot(true)
    }
}
