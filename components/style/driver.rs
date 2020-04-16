/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements traversal over the DOM tree. The traversal starts in sequential
//! mode, and optionally parallelizes as it discovers work.

#![deny(missing_docs)]

use crate::context::{PerThreadTraversalStatistics, StyleContext};
use crate::context::{ThreadLocalStyleContext, TraversalStatistics};
use crate::dom::{SendNode, TElement, TNode};
use crate::parallel;
use crate::parallel::{DispatchMode, WORK_UNIT_MAX};
use crate::scoped_tls::ScopedTLS;
use crate::traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};
use rayon;
use std::collections::VecDeque;
use std::mem;
use time;

#[cfg(feature = "servo")]
fn should_report_statistics() -> bool {
    false
}

#[cfg(feature = "gecko")]
fn should_report_statistics() -> bool {
    unsafe { crate::gecko_bindings::structs::ServoTraversalStatistics_sActive }
}

#[cfg(feature = "servo")]
fn report_statistics(_stats: &PerThreadTraversalStatistics) {
    unreachable!("Servo never report stats");
}

#[cfg(feature = "gecko")]
fn report_statistics(stats: &PerThreadTraversalStatistics) {
    // This should only be called in the main thread, or it may be racy
    // to update the statistics in a global variable.
    debug_assert!(unsafe { crate::gecko_bindings::bindings::Gecko_IsMainThread() });
    let gecko_stats =
        unsafe { &mut crate::gecko_bindings::structs::ServoTraversalStatistics_sSingleton };
    gecko_stats.mElementsTraversed += stats.elements_traversed;
    gecko_stats.mElementsStyled += stats.elements_styled;
    gecko_stats.mElementsMatched += stats.elements_matched;
    gecko_stats.mStylesShared += stats.styles_shared;
    gecko_stats.mStylesReused += stats.styles_reused;
}

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
    pool: Option<&rayon::ThreadPool>,
) where
    E: TElement,
    D: DomTraversal<E>,
{
    let root = token
        .traversal_root()
        .expect("Should've ensured we needed to traverse");

    let report_stats = should_report_statistics();
    let dump_stats = traversal.shared_context().options.dump_style_statistics;
    let start_time = if dump_stats {
        Some(time::precise_time_s())
    } else {
        None
    };

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
    let mut tls_slots = None;
    let mut tlc = ThreadLocalStyleContext::new(traversal.shared_context());
    let mut context = StyleContext {
        shared: traversal.shared_context(),
        thread_local: &mut tlc,
    };

    // Process the nodes breadth-first, just like the parallel traversal does.
    // This helps keep similar traversal characteristics for the style sharing
    // cache.
    let mut discovered = VecDeque::<SendNode<E::ConcreteNode>>::with_capacity(WORK_UNIT_MAX * 2);
    let mut depth = root.depth();
    let mut nodes_remaining_at_current_depth = 1;
    discovered.push_back(unsafe { SendNode::new(root.as_node()) });
    while let Some(node) = discovered.pop_front() {
        let mut children_to_process = 0isize;
        let traversal_data = PerLevelTraversalData {
            current_dom_depth: depth,
        };
        traversal.process_preorder(&traversal_data, &mut context, *node, |n| {
            children_to_process += 1;
            discovered.push_back(unsafe { SendNode::new(n) });
        });

        traversal.handle_postorder_traversal(
            &mut context,
            root.as_node().opaque(),
            *node,
            children_to_process,
        );

        nodes_remaining_at_current_depth -= 1;
        if nodes_remaining_at_current_depth == 0 {
            depth += 1;
            // If there is enough work to parallelize over, and the caller allows
            // parallelism, switch to the parallel driver. We do this only when
            // moving to the next level in the dom so that we can pass the same
            // depth for all the children.
            if pool.is_some() && discovered.len() > WORK_UNIT_MAX {
                let pool = pool.unwrap();
                let tls = ScopedTLS::<ThreadLocalStyleContext<E>>::new(pool);
                let root_opaque = root.as_node().opaque();
                let drain = discovered.drain(..);
                pool.install(|| {
                    // Enable a breadth-first rayon traversal. This causes the work
                    // queue to be always FIFO, rather than FIFO for stealers and
                    // FILO for the owner (which is what rayon does by default). This
                    // ensures that we process all the elements at a given depth before
                    // proceeding to the next depth, which is important for style sharing.
                    rayon::scope_fifo(|scope| {
                        profiler_label!(Style);
                        parallel::traverse_nodes(
                            drain,
                            DispatchMode::TailCall,
                            /* recursion_ok = */ true,
                            root_opaque,
                            PerLevelTraversalData {
                                current_dom_depth: depth,
                            },
                            scope,
                            pool,
                            traversal,
                            &tls,
                        );
                    });
                });

                tls_slots = Some(tls.into_slots());
                break;
            }
            nodes_remaining_at_current_depth = discovered.len();
        }
    }

    // Collect statistics from thread-locals if requested.
    if dump_stats || report_stats {
        let mut aggregate = mem::replace(&mut context.thread_local.statistics, Default::default());
        let parallel = tls_slots.is_some();
        if let Some(ref mut tls) = tls_slots {
            for slot in tls.iter_mut() {
                if let Some(cx) = slot.get_mut() {
                    aggregate += cx.statistics.clone();
                }
            }
        }

        if report_stats {
            report_statistics(&aggregate);
        }
        // dump statistics to stdout if requested
        if dump_stats {
            let stats =
                TraversalStatistics::new(aggregate, traversal, parallel, start_time.unwrap());
            if stats.is_large {
                println!("{}", stats);
            }
        }
    }
}
