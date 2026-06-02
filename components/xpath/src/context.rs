/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::Dom;

/// The context during evaluation of an XPath expression.
#[derive(Debug)]
pub(crate) struct EvaluationCtx<D: Dom> {
    /// The "current" node in the evaluation.
    pub(crate) context_node: D::Node,
    /// Details needed for evaluating a predicate list.
    pub(crate) predicate_ctx: Option<PredicateCtx>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PredicateCtx {
    pub(crate) index: usize,
    pub(crate) size: usize,
}

impl<D: Dom> EvaluationCtx<D> {
    /// Prepares the context used while evaluating the XPath expression.
    pub(crate) fn new(context_node: D::Node) -> Self {
        EvaluationCtx {
            context_node,
            predicate_ctx: None,
        }
    }

    /// Creates a new context using the provided node as the context node.
    pub(crate) fn subcontext_for_node(&self, node: D::Node) -> Self {
        EvaluationCtx {
            context_node: node,
            predicate_ctx: self.predicate_ctx,
        }
    }
}
