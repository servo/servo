/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem;
use std::rc::Rc;
use style::context::{LocalStyleContext, SharedStyleContext, StyleContext};
use style::dom::{OpaqueNode, TNode};
use style::traversal::{DomTraversalContext, recalc_style_at};

pub struct StandaloneStyleContext<'a> {
    pub shared: &'a SharedStyleContext,
    cached_local_style_context: Rc<LocalStyleContext>,
}

impl<'a> StandaloneStyleContext<'a> {
    pub fn new(_: &'a SharedStyleContext) -> Self { panic!("Not implemented") }
}

impl<'a> StyleContext<'a> for StandaloneStyleContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext {
        &self.shared
    }

    fn local_context(&self) -> &LocalStyleContext {
        &self.cached_local_style_context
    }
}

pub struct RecalcStyleOnly<'lc> {
    context: StandaloneStyleContext<'lc>,
    root: OpaqueNode,
}

impl<'lc, 'ln, N: TNode<'ln>> DomTraversalContext<'ln, N> for RecalcStyleOnly<'lc> {
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

