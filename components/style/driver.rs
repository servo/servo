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

fn with_pool_in_place_scope<'scope, R>(
    work_unit_max: usize,
    pool: Option<&rayon::ThreadPool>,
    closure: impl FnOnce(Option<&rayon::ScopeFifo<'scope>>) -> R,
) -> R {
    if work_unit_max == 0 || pool.is_none() {
        closure(None)
    } else {
        pool.unwrap().in_place_scope_fifo(|scope| {
            closure(Some(scope))
        })
    }
}

/// See documentation of the pref for performance characteristics.
fn work_unit_max() -> usize {
    static_prefs::pref!("layout.css.stylo-work-unit-size") as usize
}

/// Do a DOM traversal for top-down and (optionally) bottom-up processing, generic over `D`.
///
/// We use an adaptive traversal strategy. We start out with simple sequential processing, until we
/// arrive at a wide enough level in the DOM that the parallel traversal would parallelize it.
/// If a thread pool is provided, we then transfer control over to the parallel traversal.
///
/// Returns true if the traversal was parallel, and also returns the statistics object containing
/// information on nodes traversed (on nightly only). Not all of its fields will be initialized
/// since we don't call finish().
pub fn traverse_dom<E, D>(
    traversal: &D,
    token: PreTraverseToken<E>,
    pool: Option<&rayon::ThreadPool>,
) -> E
where
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
    let mut scoped_tls = pool.map(ScopedTLS::<ThreadLocalStyleContext<E>>::new);
    let mut tlc = ThreadLocalStyleContext::new();
    let mut context = StyleContext {
        shared: traversal.shared_context(),
        thread_local: &mut tlc,
    };

    // Process the nodes breadth-first. This helps keep similar traversal characteristics for the
    // style sharing cache.
    let work_unit_max = work_unit_max();
    with_pool_in_place_scope(work_unit_max, pool, |maybe_scope| {
        let mut discovered = VecDeque::with_capacity(work_unit_max * 2);
        discovered.push_back(unsafe { SendNode::new(root.as_node()) });
        parallel::style_trees(
            &mut context,
            discovered,
            root.as_node().opaque(),
            work_unit_max,
            static_prefs::pref!("layout.css.stylo-local-work-queue.in-main-thread") as usize,
            PerLevelTraversalData { current_dom_depth: root.depth() },
            maybe_scope,
            traversal,
            scoped_tls.as_ref(),
        );
    });

    // Collect statistics from thread-locals if requested.
    if dump_stats || report_stats {
        let mut aggregate = mem::replace(&mut context.thread_local.statistics, Default::default());
        let parallel = pool.is_some();
        if let Some(ref mut tls) = scoped_tls {
            for slot in tls.slots() {
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

    root
}
