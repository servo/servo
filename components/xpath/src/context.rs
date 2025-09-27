/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::iter::Enumerate;
use std::vec::IntoIter;

use crate::{Dom, NamespaceResolver, Node};

/// The context during evaluation of an XPath expression.
pub(crate) struct EvaluationCtx<D: Dom> {
    /// Where we started at.
    pub(crate) starting_node: D::Node,
    /// The "current" node in the evaluation.
    pub(crate) context_node: D::Node,
    /// Details needed for evaluating a predicate list.
    pub(crate) predicate_ctx: Option<PredicateCtx>,
    /// The nodes we're currently matching against.
    pub(crate) predicate_nodes: Option<Vec<D::Node>>,
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
            predicate_nodes: None,
            resolver,
        }
    }

    /// Creates a new context using the provided node as the context node
    pub(crate) fn subcontext_for_node(&self, node: D::Node) -> Self {
        EvaluationCtx {
            starting_node: self.starting_node.clone(),
            context_node: node,
            predicate_ctx: self.predicate_ctx,
            predicate_nodes: self.predicate_nodes.clone(),
            resolver: self.resolver.clone(),
        }
    }

    pub(crate) fn update_predicate_nodes(&self, nodes: Vec<D::Node>) -> Self {
        EvaluationCtx {
            starting_node: self.starting_node.clone(),
            context_node: self.context_node.clone(),
            predicate_ctx: None,
            predicate_nodes: Some(nodes),
            resolver: self.resolver.clone(),
        }
    }

    pub(crate) fn subcontext_iter_for_nodes(&self) -> EvalNodesetIter<'_, D> {
        let size = self.predicate_nodes.as_ref().map_or(0, |v| v.len());
        EvalNodesetIter {
            ctx: self,
            nodes_iter: self
                .predicate_nodes
                .as_ref()
                .map_or_else(|| Vec::new().into_iter(), |v| v.clone().into_iter())
                .enumerate(),
            size,
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

/// When evaluating predicates, we need to keep track of the current node being evaluated and
/// the index of that node in the nodeset we're operating on.
pub(crate) struct EvalNodesetIter<'a, D: Dom> {
    ctx: &'a EvaluationCtx<D>,
    nodes_iter: Enumerate<IntoIter<D::Node>>,
    size: usize,
}

impl<D: Dom> Iterator for EvalNodesetIter<'_, D> {
    type Item = EvaluationCtx<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.nodes_iter.next().map(|(idx, node)| EvaluationCtx {
            starting_node: self.ctx.starting_node.clone(),
            context_node: node.clone(),
            predicate_nodes: self.ctx.predicate_nodes.clone(),
            predicate_ctx: Some(PredicateCtx {
                index: idx + 1,
                size: self.size,
            }),
            resolver: self.ctx.resolver.clone(),
        })
    }
}

impl<D: Dom> fmt::Debug for EvaluationCtx<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvaluationCtx")
            .field("starting_node", &self.starting_node)
            .field("context_node", &self.context_node)
            .field("predicate_ctx", &self.predicate_ctx)
            .field("predicate_nodes", &self.predicate_nodes)
            .field("resolver", &"<callback function>")
            .finish()
    }
}
