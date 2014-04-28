/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::tokenize;
use dom::bindings::codegen::InheritTypes::{DocumentFragmentDerived, ElementCast, NodeCast};
use dom::bindings::codegen::DocumentFragmentBinding;
use dom::bindings::js::JS;
use dom::bindings::error::{Fallible, TypeError};
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::{DocumentFragmentNodeTypeId, Node, NodeHelpers};
use dom::window::Window;
use servo_util::str::DOMString;
use style::parse_selector_list;
use style::matches_compound_selector;
use style::NamespaceMap;

#[deriving(Encodable)]
pub struct DocumentFragment {
    pub node: Node,
}

impl DocumentFragmentDerived for EventTarget {
    fn is_documentfragment(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(DocumentFragmentNodeTypeId) => true,
            _ => false
        }
    }
}

impl DocumentFragment {
    /// Creates a new DocumentFragment.
    pub fn new_inherited(document: JS<Document>) -> DocumentFragment {
        DocumentFragment {
            node: Node::new_inherited(DocumentFragmentNodeTypeId, document),
        }
    }

    pub fn new(document: &JS<Document>) -> JS<DocumentFragment> {
        let node = DocumentFragment::new_inherited(document.clone());
        Node::reflect_node(~node, document, DocumentFragmentBinding::Wrap)
    }
}

impl DocumentFragment {
    pub fn Constructor(owner: &JS<Window>) -> Fallible<JS<DocumentFragment>> {
        Ok(DocumentFragment::new(&owner.get().Document()))
    }
}

// http://dom.spec.whatwg.org/#interface-parentnode
impl DocumentFragment {
    // http://dom.spec.whatwg.org/#dom-parentnode-children
    pub fn Children(&self, abstract_self: &JS<DocumentFragment>) -> JS<HTMLCollection> {
        let doc = self.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::children(&doc.window, &NodeCast::from(abstract_self))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn QuerySelector(&self, abstract_self: &JS<DocumentFragment>, selectors: DOMString) -> Fallible<Option<JS<Element>>> {
        // Step 1.
        let namespace = NamespaceMap::new();
        let maybe_selectors = parse_selector_list(tokenize(selectors).map(|(token, _)| token).to_owned_vec(), &namespace);
        match maybe_selectors {
            // Step 2.
            None => return Err(TypeError),
            // Step 3.
            Some(ref selectors) => {
                for selector in selectors.iter() {
                    assert!(selector.pseudo_element.is_none());
                    let root: JS<Node> = NodeCast::from(abstract_self);
                    for node in root.traverse_preorder().filter(|node| node.is_element()) {
                        let elem: JS<Element> = ElementCast::to(&node).unwrap();
                        let mut shareable: bool = false;
                        if matches_compound_selector(selector.compound_selectors.get(), &node, &mut shareable) {
                            return Ok(Some(elem));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}
