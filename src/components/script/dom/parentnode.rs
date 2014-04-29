/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::tokenize;
use dom::bindings::js::JS;
use dom::bindings::error::{Fallible, TypeError};
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeBase, NodeCast};
use dom::node::{Node, NodeHelpers};
use dom::element::Element;
use dom::document::Document;
use dom::documentfragment::DocumentFragment;
use dom::htmlcollection::HTMLCollection;
use servo_util::str::{DOMString};
use style::parse_selector_list;
use style::matches_compound_selector;
use style::NamespaceMap;

// http://dom.spec.whatwg.org/#interface-parentnode
pub trait ParentNode : NodeBase {
    // http://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self, abstract_self: &JS<Self>) -> JS<HTMLCollection> {
        let doc: JS<Node> = NodeCast::from(abstract_self);
        let doc = doc.get().owner_doc();
        let doc = doc.get();
        HTMLCollection::children(&doc.window, &NodeCast::from(abstract_self))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, abstract_self: &JS<Self>, selectors: DOMString) -> Fallible<Option<JS<Element>>> {
        // Step 1.
        let namespace = NamespaceMap::new();
        let maybe_selectors = parse_selector_list(tokenize(selectors).map(|(token, _)| token).collect(), &namespace);
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
                        if matches_compound_selector(&*(selector.compound_selectors), &node, &mut shareable) {
                            return Ok(Some(elem));
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

impl ParentNode for Document {}
impl ParentNode for DocumentFragment {}
impl ParentNode for Element {}
