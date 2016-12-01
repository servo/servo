/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversal over the DOM tree.

use dom::{TElement, TNode};
use traversal::{DomTraversalContext, PerLevelTraversalData, PreTraverseToken};

pub fn traverse_dom<N, C>(root: N::ConcreteElement,
                          shared: &C::SharedContext,
                          token: PreTraverseToken)
    where N: TNode,
          C: DomTraversalContext<N>
{
    debug_assert!(token.should_traverse());

    fn doit<'a, N, C>(context: &'a C, node: N, data: &mut PerLevelTraversalData)
        where N: TNode,
              C: DomTraversalContext<N>
    {
        context.process_preorder(node, data);
        if let Some(el) = node.as_element() {
            if let Some(ref mut depth) = data.current_dom_depth {
                *depth += 1;
            }

            C::traverse_children(el, |kid| doit::<N, C>(context, kid, data));

            if let Some(ref mut depth) = data.current_dom_depth {
                *depth -= 1;
            }
        }

        if C::needs_postorder_traversal() {
            context.process_postorder(node);
        }
    }

    let mut data = PerLevelTraversalData {
        current_dom_depth: None,
    };
    let context = C::new(shared, root.as_node().opaque());

    if token.should_skip_root() {
        C::traverse_children(root, |kid| doit::<N, C>(&context, kid, &mut data));
    } else {
        doit::<N, C>(&context, root.as_node(), &mut data);
    }

    // Clear the local LRU cache since we store stateful elements inside.
    context.local_context().style_sharing_candidate_cache.borrow_mut().clear();
}
