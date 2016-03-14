/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use selector_impl::{GeckoSelectorImpl, SharedStyleContext};
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use style::context::{LocalStyleContext, StyleContext};
use style::dom::{OpaqueNode, TNode};
use style::matching::{ApplicableDeclarationsCache, StyleSharingCandidateCache};
use style::traversal::{DomTraversalContext, recalc_style_at};

thread_local!(static LOCAL_CONTEXT_KEY: RefCell<Option<Rc<LocalStyleContext>>> = RefCell::new(None));

// Keep this implementation in sync with the one in components/layout/context.rs.
fn create_or_get_local_context(shared: &SharedStyleContext)
                               -> Rc<LocalStyleContext> {
    LOCAL_CONTEXT_KEY.with(|r| {
        let mut r = r.borrow_mut();
        if let Some(context) = r.clone() {
            if shared.screen_size_changed {
                context.applicable_declarations_cache.borrow_mut().evict_all();
            }
            context
        } else {
            let context = Rc::new(LocalStyleContext {
                applicable_declarations_cache: RefCell::new(ApplicableDeclarationsCache::new()),
                style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
            });
            *r = Some(context.clone());
            context
        }
    })
}

pub struct StandaloneStyleContext<'a> {
    pub shared: &'a SharedStyleContext,
    cached_local_context: Rc<LocalStyleContext>,
}

impl<'a> StandaloneStyleContext<'a> {
    pub fn new(shared: &'a SharedStyleContext) -> Self {
        let local_context = create_or_get_local_context(shared);
        StandaloneStyleContext {
            shared: shared,
            cached_local_context: local_context,
        }
    }
}

impl<'a> StyleContext<'a, GeckoSelectorImpl> for StandaloneStyleContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext {
        &self.shared
    }

    fn local_context(&self) -> &LocalStyleContext {
        &self.cached_local_context
    }
}

pub struct RecalcStyleOnly<'lc> {
    context: StandaloneStyleContext<'lc>,
    root: OpaqueNode,
}

impl<'lc, N: TNode> DomTraversalContext<N> for RecalcStyleOnly<'lc>
    where N::ConcreteElement: ::selectors::Element<Impl=GeckoSelectorImpl> {
    type SharedContext = SharedStyleContext;
    #[allow(unsafe_code)]
    fn new<'a>(shared: &'a Self::SharedContext, root: OpaqueNode) -> Self {
        // See the comment in RecalcStyleAndConstructFlows::new for an explanation of why this is
        // necessary.
        let shared_lc: &'lc SharedStyleContext = unsafe { mem::transmute(shared) };
        RecalcStyleOnly {
            context: StandaloneStyleContext::new(shared_lc),
            root: root,
        }
    }

    fn process_preorder(&self, node: N) { recalc_style_at(&self.context, self.root, node); }
    fn process_postorder(&self, _: N) {}
}

