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
use crate::dom::live_range_text_split_steps;
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
        // Step 1: Let length be node’s length.
        let length = cdata.Length();
        // Step 2: If offset is greater than length, then throw an "IndexSizeError" DOMException.
        if offset > length {
            return Err(Error::IndexSize(None));
        }
        // Step 3: Let count be length − offset.
        let count = length - offset;
        // Step 4: Let newData be the result of substringing data of node with offset and count.
        let new_data = cdata.SubstringData(offset, count).unwrap();
        // Step 5: Let newNode be the result of creating a text node given node’s node document and newData.
        let node = self.upcast::<Node>();
        let owner_doc = node.owner_doc();
        let new_node = owner_doc.CreateTextNode(cx, new_data);
        // Step 6: Let parent be node’s parent.
        let parent = node.GetParentNode();
        // Step 7: If parent is non-null:
        if let Some(ref parent) = parent {
            // Step 7.1: Insert newNode into parent before node’s next sibling.
            parent
                .InsertBefore(cx, new_node.upcast(), node.GetNextSibling().as_deref())
                .unwrap();
            // Steps 7.2-7.5: The live range update steps.
            live_range_text_split_steps(parent, node, offset, new_node.upcast());
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
        slottable.find_a_slot(cx.no_gc(), true)
    }
}
