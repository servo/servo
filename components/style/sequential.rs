/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversal over the DOM tree.

use dom::TNode;
use traversal::{DomTraversalContext, PerLevelTraversalData};

pub fn traverse_dom<N, C>(root: N,
                          shared: &C::SharedContext)
    where N: TNode,
          C: DomTraversalContext<N>
{
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

        if context.needs_postorder_traversal() {
            context.process_postorder(node);
        }
    }

    let mut data = PerLevelTraversalData {
        current_dom_depth: None,
    };
    let context = C::new(shared, root.opaque());
    doit::<N, C>(&context, root, &mut data);

    // Clear the local LRU cache since we store stateful elements inside.
    context.local_context().style_sharing_candidate_cache.borrow_mut().clear();
}
