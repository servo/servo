/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::iter::Enumerate;
use std::rc::Rc;
use std::vec::IntoIter;

use script_bindings::error::Fallible;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;

use super::Node;
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::root::DomRoot;

/// The context during evaluation of an XPath expression.
pub(crate) struct EvaluationCtx {
    /// Where we started at.
    pub(crate) starting_node: DomRoot<Node>,
    /// The "current" node in the evaluation.
    pub(crate) context_node: DomRoot<Node>,
    /// Details needed for evaluating a predicate list.
    pub(crate) predicate_ctx: Option<PredicateCtx>,
    /// The nodes we're currently matching against.
    pub(crate) predicate_nodes: Option<Vec<DomRoot<Node>>>,
    /// A list of known namespace prefixes.
    pub(crate) resolver: Option<Rc<XPathNSResolver>>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PredicateCtx {
    pub(crate) index: usize,
    pub(crate) size: usize,
}

impl EvaluationCtx {
    /// Prepares the context used while evaluating the XPath expression
    pub(crate) fn new(context_node: &Node, resolver: Option<Rc<XPathNSResolver>>) -> EvaluationCtx {
        EvaluationCtx {
            starting_node: DomRoot::from_ref(context_node),
            context_node: DomRoot::from_ref(context_node),
            predicate_ctx: None,
            predicate_nodes: None,
            resolver,
        }
    }

    /// Creates a new context using the provided node as the context node
    pub(crate) fn subcontext_for_node(&self, node: &Node) -> EvaluationCtx {
        EvaluationCtx {
            starting_node: self.starting_node.clone(),
            context_node: DomRoot::from_ref(node),
            predicate_ctx: self.predicate_ctx,
            predicate_nodes: self.predicate_nodes.clone(),
            resolver: self.resolver.clone(),
        }
    }

    pub(crate) fn update_predicate_nodes(&self, nodes: Vec<&Node>) -> EvaluationCtx {
        EvaluationCtx {
            starting_node: self.starting_node.clone(),
            context_node: self.context_node.clone(),
            predicate_ctx: None,
            predicate_nodes: Some(nodes.into_iter().map(DomRoot::from_ref).collect()),
            resolver: self.resolver.clone(),
        }
    }

    pub(crate) fn subcontext_iter_for_nodes(&self) -> EvalNodesetIter<'_> {
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
        can_gc: CanGc,
    ) -> Fallible<Option<DOMString>> {
        // First check if the prefix is known by our resolver function
        if let Some(resolver) = self.resolver.as_ref() {
            if let Some(namespace_uri) = resolver.LookupNamespaceURI__(
                prefix.map(DOMString::from),
                ExceptionHandling::Rethrow,
                can_gc,
            )? {
                return Ok(Some(namespace_uri));
            }
        }

        // Then, see if it's defined on the context node
        Ok(self
            .context_node
            .LookupNamespaceURI(prefix.map(DOMString::from)))
    }
}

/// When evaluating predicates, we need to keep track of the current node being evaluated and
/// the index of that node in the nodeset we're operating on.
pub(crate) struct EvalNodesetIter<'a> {
    ctx: &'a EvaluationCtx,
    nodes_iter: Enumerate<IntoIter<DomRoot<Node>>>,
    size: usize,
}

impl Iterator for EvalNodesetIter<'_> {
    type Item = EvaluationCtx;

    fn next(&mut self) -> Option<EvaluationCtx> {
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

impl fmt::Debug for EvaluationCtx {
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
