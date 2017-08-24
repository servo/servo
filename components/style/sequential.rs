/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversal over the DOM tree.

#![deny(missing_docs)]

use context::{StyleContext, ThreadLocalStyleContext};
use dom::{SendNode, TElement, TNode};
use parallel::WORK_UNIT_MAX;
use std::collections::VecDeque;
use time;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};

/// Do a sequential DOM traversal for layout or styling, generic over `D`.
pub fn traverse_dom<E, D>(traversal: &D,
                          root: E,
                          token: PreTraverseToken)
    where E: TElement,
          D: DomTraversal<E>,
{
    let dump_stats = traversal.shared_context().options.dump_style_statistics;
    let start_time = if dump_stats { Some(time::precise_time_s()) } else { None };

    debug_assert!(!traversal.is_parallel());
    debug_assert!(token.should_traverse());

    let mut discovered =
        VecDeque::<SendNode<E::ConcreteNode>>::with_capacity(WORK_UNIT_MAX * 2);
    let mut tlc = ThreadLocalStyleContext::new(traversal.shared_context());
    let mut context = StyleContext {
        shared: traversal.shared_context(),
        thread_local: &mut tlc,
    };


    // Process the nodes breadth-first, just like the parallel traversal does.
    // This helps keep similar traversal characteristics for the style sharing
    // cache.
    let mut depth = root.depth();
    let mut nodes_remaining_at_current_depth = 1;
    discovered.push_back(unsafe { SendNode::new(root.as_node()) });
    while let Some(node) = discovered.pop_front() {
        let mut children_to_process = 0isize;
        let traversal_data = PerLevelTraversalData { current_dom_depth: depth };
        traversal.process_preorder(&traversal_data, &mut context, *node, |n| {
            children_to_process += 1;
            discovered.push_back(unsafe { SendNode::new(n) });
        });

        traversal.handle_postorder_traversal(&mut context, root.as_node().opaque(),
                                             *node, children_to_process);
        nodes_remaining_at_current_depth -= 1;
        if nodes_remaining_at_current_depth == 0 {
            depth += 1;
            nodes_remaining_at_current_depth = discovered.len();
        }
    }

    // Dump statistics to stdout if requested.
    if dump_stats {
        context.thread_local.statistics.finish(traversal, start_time.unwrap());
        if context.thread_local.statistics.is_large_traversal() {
            println!("{}", context.thread_local.statistics);
        }
    }
}
