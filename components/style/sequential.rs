/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversal over the DOM tree.

use dom::{TElement, TNode};
use std::borrow::Borrow;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};

pub fn traverse_dom<N, D>(traversal: &D,
                          root: N::ConcreteElement,
                          token: PreTraverseToken)
    where N: TNode,
          D: DomTraversal<N>
{
    debug_assert!(token.should_traverse());

    fn doit<N, D>(traversal: &D, node: N, data: &mut PerLevelTraversalData)
        where N: TNode,
              D: DomTraversal<N>
    {
        traversal.process_preorder(node, data);
        if let Some(el) = node.as_element() {
            if let Some(ref mut depth) = data.current_dom_depth {
                *depth += 1;
            }

            D::traverse_children(el, |kid| doit(traversal, kid, data));

            if let Some(ref mut depth) = data.current_dom_depth {
                *depth -= 1;
            }
        }

        if D::needs_postorder_traversal() {
            traversal.process_postorder(node);
        }
    }

    let mut data = PerLevelTraversalData {
        current_dom_depth: None,
    };

    if token.traverse_unstyled_children_only() {
        for kid in root.as_node().children() {
            if kid.as_element().map_or(false, |el| el.get_data().is_none()) {
                doit(traversal, kid, &mut data);
            }
        }
    } else {
        doit(traversal, root.as_node(), &mut data);
    }

    // Clear the local LRU cache since we store stateful elements inside.
    let tlc = traversal.create_or_get_thread_local_context();
    (*tlc).borrow().style_sharing_candidate_cache.borrow_mut().clear();
}
