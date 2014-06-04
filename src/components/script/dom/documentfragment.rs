/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::tokenize;
use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, ElementCast, NodeCast};
use dom::bindings::codegen::Bindings::DocumentFragmentBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::{Fallible, Syntax};
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{DocumentFragmentNodeTypeId, Node, NodeHelpers, window_from_node};
use dom::window::{Window, WindowMethods};
use servo_util::str::DOMString;
use style::{parse_selector_list, matches_compound_selector, NamespaceMap};

#[deriving(Encodable)]
pub struct DocumentFragment {
    pub node: Node,
}

impl DocumentFragmentDerived for EventTarget {
    fn is_documentfragment(&self) -> bool {
        self.type_id == NodeTargetTypeId(DocumentFragmentNodeTypeId)
    }
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub fn new_inherited(document: &JSRef<Document>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn new(document: &JSRef<Document>) -> Temporary<DocumentFragment> {
        let node = DocumentFragment::new_inherited(document);
        Node::reflect_node(box node, document, DocumentFragmentBinding::Wrap)
    }

    pub fn Constructor(owner: &JSRef<Window>) -> Fallible<Temporary<DocumentFragment>> {
        let document = owner.Document();
        let document = document.root();

        Ok(DocumentFragment::new(&document.root_ref()))
    }
}

pub trait DocumentFragmentMethods {
    fn Children(&self) -> Temporary<HTMLCollection>;
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>>;
}

impl<'a> DocumentFragmentMethods for JSRef<'a, DocumentFragment> {
    fn Children(&self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(&window.root_ref(), NodeCast::from_ref(self))
    }

    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        // Step 1.
        let namespace = NamespaceMap::new();
        match parse_selector_list(tokenize(selectors).map(|(token, _)| token).collect(), &namespace) {
            // Step 2.
            None => return Err(Syntax),
            // Step 3.
            Some(ref selectors) => {
                for selector in selectors.iter() {
                    assert!(selector.pseudo_element.is_none());
                    let root: &JSRef<Node> = NodeCast::from_ref(self);
                    for node in root.traverse_preorder().filter(|node| node.is_element()) {
                        let elem: Option<&JSRef<Element>> = ElementCast::to_ref(&node);
                        let mut shareable: bool = false;
                        if matches_compound_selector(selector.compound_selectors.deref(), &node, &mut shareable) {
                            return Ok(elem.map(|elem| Temporary::from_rooted(elem)));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}
