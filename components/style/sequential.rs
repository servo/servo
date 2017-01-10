/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversal over the DOM tree.

#![deny(missing_docs)]

use dom::{TElement, TNode};
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};

/// Do a sequential DOM traversal for layout or styling, generic over `D`.
pub fn traverse_dom<E, D>(traversal: &D,
                          root: E,
                          token: PreTraverseToken)
    where E: TElement,
          D: DomTraversal<E>,
{
    debug_assert!(token.should_traverse());

    fn doit<E, D>(traversal: &D, traversal_data: &mut PerLevelTraversalData,
                  thread_local: &mut D::ThreadLocalContext, node: E::ConcreteNode)
        where E: TElement,
              D: DomTraversal<E>
    {
        traversal.process_preorder(traversal_data, thread_local, node);
        if let Some(el) = node.as_element() {
            if let Some(ref mut depth) = traversal_data.current_dom_depth {
                *depth += 1;
            }

            traversal.traverse_children(thread_local, el, |tlc, kid| {
                doit(traversal, traversal_data, tlc, kid)
            });

            if let Some(ref mut depth) = traversal_data.current_dom_depth {
                *depth -= 1;
            }
        }

        if D::needs_postorder_traversal() {
            traversal.process_postorder(thread_local, node);
        }
    }

    let mut traversal_data = PerLevelTraversalData {
        current_dom_depth: None,
    };

    let mut tlc = traversal.create_thread_local_context();
    if token.traverse_unstyled_children_only() {
        for kid in root.as_node().children() {
            if kid.as_element().map_or(false, |el| el.get_data().is_none()) {
                doit(traversal, &mut traversal_data, &mut tlc, kid);
            }
        }
    } else {
        doit(traversal, &mut traversal_data, &mut tlc, root.as_node());
    }
}
