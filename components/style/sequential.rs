/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversal over the DOM tree.

use dom::TNode;
use traversal::{RestyleResult, DomTraversalContext};

pub fn traverse_dom<N, C>(root: N,
                          shared: &C::SharedContext)
    where N: TNode,
          C: DomTraversalContext<N>
{
    fn doit<'a, N, C>(context: &'a C, node: N)
        where N: TNode,
              C: DomTraversalContext<N>
    {
        debug_assert!(context.should_process(node));
        if let RestyleResult::Continue = context.process_preorder(node) {
            for kid in node.children() {
                context.pre_process_child_hook(node, kid);
                if context.should_process(kid) {
                    doit::<N, C>(context, kid);
                }
            }
        }

        if context.needs_postorder_traversal() {
            context.process_postorder(node);
        }
    }

    let context = C::new(shared, root.opaque());
    if context.should_process(root) {
        doit::<N, C>(&context, root);
    }
    // Clear the local LRU cache since we store stateful elements inside.
    context.local_context().style_sharing_candidate_cache.borrow_mut().clear();
}
