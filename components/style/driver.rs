/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements traversal over the DOM tree. The traversal starts in sequential
//! mode, and optionally parallelizes as it discovers work.

#![deny(missing_docs)]

use context::{StyleContext, ThreadLocalStyleContext, TraversalStatistics};
use dom::{SendNode, TElement, TNode};
use parallel;
use parallel::{DispatchMode, WORK_UNIT_MAX};
use rayon;
use scoped_tls::ScopedTLS;
use std::collections::VecDeque;
use std::mem;
use time;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};

/// Do a DOM traversal for top-down and (optionally) bottom-up processing,
/// generic over `D`.
///
/// We use an adaptive traversal strategy. We start out with simple sequential
/// processing, until we arrive at a wide enough level in the DOM that the
/// parallel traversal would parallelize it. If a thread pool is provided, we
/// then transfer control over to the parallel traversal.
///
/// Returns true if the traversal was parallel, and also returns the statistics
/// object containing information on nodes traversed (on nightly only). Not
/// all of its fields will be initialized since we don't call finish().
pub fn traverse_dom<E, D>(
    traversal: &D,
    token: PreTraverseToken<E>,
    pool: Option<&rayon::ThreadPool>
) -> (bool, Option<TraversalStatistics>)
where
    E: TElement,
    D: DomTraversal<E>,
{
    let root =
        token.traversal_root().expect("Should've ensured we needed to traverse");

    let dump_stats = traversal.shared_context().options.dump_style_statistics;
    let is_nightly  = traversal.shared_context().options.is_nightly();
    let mut used_parallel = false;
    let start_time = if dump_stats { Some(time::precise_time_s()) } else { None };

    // Declare the main-thread context, as well as the worker-thread contexts,
    // which we may or may not instantiate. It's important to declare the worker-
    // thread contexts first, so that they get dropped second. This matters because:
    //   * ThreadLocalContexts borrow AtomicRefCells in TLS.
    //   * Dropping a ThreadLocalContext can run SequentialTasks.
    //   * Sequential tasks may call into functions like
    //     Servo_StyleSet_GetBaseComputedValuesForElement, which instantiate a
    //     ThreadLocalStyleContext on the main thread. If the main thread
    //     ThreadLocalStyleContext has not released its TLS borrow by that point,
    //     we'll panic on double-borrow.
    let mut maybe_tls: Option<ScopedTLS<ThreadLocalStyleContext<E>>> = None;
    let mut tlc = ThreadLocalStyleContext::new(traversal.shared_context());
    let mut context = StyleContext {
        shared: traversal.shared_context(),
        thread_local: &mut tlc,
    };

    // Process the nodes breadth-first, just like the parallel traversal does.
    // This helps keep similar traversal characteristics for the style sharing
    // cache.
    let mut discovered =
        VecDeque::<SendNode<E::ConcreteNode>>::with_capacity(WORK_UNIT_MAX * 2);
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
            // If there is enough work to parallelize over, and the caller allows
            // parallelism, switch to the parallel driver. We do this only when
            // moving to the next level in the dom so that we can pass the same
            // depth for all the children.
            if pool.is_some() && discovered.len() > WORK_UNIT_MAX {
                used_parallel = true;
                let pool = pool.unwrap();
                maybe_tls = Some(ScopedTLS::<ThreadLocalStyleContext<E>>::new(pool));
                let root_opaque = root.as_node().opaque();
                let drain = discovered.drain(..);
                pool.install(|| {
                    rayon::scope(|scope| {
                        parallel::traverse_nodes(
                            drain,
                            DispatchMode::TailCall,
                            /* recursion_ok = */ true,
                            root_opaque,
                            PerLevelTraversalData { current_dom_depth: depth },
                            scope,
                            pool,
                            traversal,
                            maybe_tls.as_ref().unwrap()
                        );
                    });
                });
                break;
            }
            nodes_remaining_at_current_depth = discovered.len();
        }
    }
    let mut maybe_stats = None;
    // Accumulate statistics
    if dump_stats || is_nightly {
        let mut aggregate =
            mem::replace(&mut context.thread_local.statistics, Default::default());
        let parallel = maybe_tls.is_some();
        if let Some(ref mut tls) = maybe_tls {
            let slots = unsafe { tls.unsafe_get() };
            aggregate = slots.iter().fold(aggregate, |acc, t| {
                match *t.borrow() {
                    None => acc,
                    Some(ref cx) => &cx.statistics + &acc,
                }
            });
        }

        // dump to stdout if requested
        if dump_stats && aggregate.is_large_traversal() {
            aggregate.finish(traversal, parallel, start_time.unwrap());
             println!("{}", aggregate);
        }
        maybe_stats = Some(aggregate);
    }

    (used_parallel, maybe_stats)
}
