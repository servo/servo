/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::{Dom, NamespaceResolver, Node};

/// The context during evaluation of an XPath expression.
pub(crate) struct EvaluationCtx<D: Dom> {
    /// Where we started at.
    pub(crate) starting_node: D::Node,
    /// The "current" node in the evaluation.
    pub(crate) context_node: D::Node,
    /// Details needed for evaluating a predicate list.
    pub(crate) predicate_ctx: Option<PredicateCtx>,
    /// A list of known namespace prefixes.
    pub(crate) resolver: Option<D::NamespaceResolver>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PredicateCtx {
    pub(crate) index: usize,
    pub(crate) size: usize,
}

impl<D: Dom> EvaluationCtx<D> {
    /// Prepares the context used while evaluating the XPath expression
    pub(crate) fn new(context_node: D::Node, resolver: Option<D::NamespaceResolver>) -> Self {
        EvaluationCtx {
            starting_node: context_node.clone(),
            context_node,
            predicate_ctx: None,
            resolver,
        }
    }

    /// Creates a new context using the provided node as the context node
    pub(crate) fn subcontext_for_node(&self, node: D::Node) -> Self {
        EvaluationCtx {
            starting_node: self.starting_node.clone(),
            context_node: node,
            predicate_ctx: self.predicate_ctx,
            resolver: self.resolver.clone(),
        }
    }

    /// Resolve a namespace prefix using the context node's document
    pub(crate) fn resolve_namespace(
        &self,
        prefix: Option<&str>,
    ) -> Result<Option<String>, D::JsError> {
        // First check if the prefix is known by our resolver function
        if let Some(resolver) = self.resolver.as_ref() {
            if let Some(namespace_uri) = resolver.resolve_namespace_prefix(prefix)? {
                return Ok(Some(namespace_uri));
            }
        }

        // Then, see if it's defined on the context node
        Ok(self.context_node.lookup_namespace_uri(prefix))
    }
}

impl<D: Dom> fmt::Debug for EvaluationCtx<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvaluationCtx")
            .field("starting_node", &self.starting_node)
            .field("context_node", &self.context_node)
            .field("predicate_ctx", &self.predicate_ctx)
            .field("resolver", &"<callback function>")
            .finish()
    }
}
