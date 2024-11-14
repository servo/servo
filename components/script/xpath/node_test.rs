/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use html5ever::{local_name, namespace_prefix, namespace_url, ns, QualName};

use super::context;
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use crate::dom::element::Element;
use crate::dom::node::Node;
use crate::dom::processinginstruction::ProcessingInstruction;

pub trait NodeTest: fmt::Debug {
    fn test(&self, context: &context::EvaluationCtx, node: &Node) -> bool;
}

impl<T: ?Sized> NodeTest for Box<T>
where
    T: NodeTest,
{
    fn test(&self, context: &context::EvaluationCtx, node: &Node) -> bool {
        (**self).test(context, node)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NameTest {
    pub qname: QualName,
    /// XML requires exact match for namespace, HTML maps null namespace to html
    pub strict_ns_comparison: bool,
}

impl NameTest {
    fn matches(&self, node_name: &QualName) -> bool {
        let is_wildcard = self.qname.local == local_name!("*");

        let test_prefix = self.qname.prefix.clone().unwrap_or(namespace_prefix!(""));
        let test_ns_uri = match test_prefix {
            namespace_prefix!("*") => ns!(*),
            namespace_prefix!("html") => ns!(html),
            namespace_prefix!("xml") => ns!(xml),
            namespace_prefix!("xlink") => ns!(xlink),
            namespace_prefix!("svg") => ns!(svg),
            namespace_prefix!("mathml") => ns!(mathml),
            namespace_prefix!("") => {
                if self.strict_ns_comparison {
                    ns!()
                } else {
                    ns!(html)
                }
            },
            _ => {
                // We don't support custom namespaces, use fallback or panic depending on strictness
                if self.strict_ns_comparison {
                    panic!("Unrecognized namespace prefix: {}", test_prefix)
                } else {
                    ns!(html)
                }
            },
        };

        if is_wildcard {
            test_ns_uri == node_name.ns
        } else {
            test_ns_uri == node_name.ns && self.qname.local == node_name.local
        }
    }
}

#[derive(Debug)]
pub struct ElementTest {
    name_test: NameTest,
}

impl ElementTest {
    pub fn new(name: NameTest) -> ElementTest {
        ElementTest { name_test: name }
    }
}

impl NodeTest for ElementTest {
    fn test(&self, _context: &context::EvaluationCtx, node: &Node) -> bool {
        if let NodeTypeId::Element(_) = node.type_id() {
            if let Some(el) = node.downcast::<Element>() {
                let qname = QualName::new(
                    el.prefix().as_ref().cloned(),
                    el.namespace().clone(),
                    el.local_name().clone(),
                );
                return self.name_test.matches(&qname);
            }
        }
        false
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct NodeTestFallback;

impl NodeTest for NodeTestFallback {
    fn test(&self, _context: &context::EvaluationCtx, _node: &Node) -> bool {
        true
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct TextTest;

impl NodeTest for TextTest {
    fn test(&self, _context: &context::EvaluationCtx, node: &Node) -> bool {
        matches!(
            node.type_id(),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_))
        )
    }
}

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct CommentTest;

impl NodeTest for CommentTest {
    fn test(&self, _context: &context::EvaluationCtx, node: &Node) -> bool {
        matches!(
            node.type_id(),
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment)
        )
    }
}

#[derive(Debug)]
pub struct ProcessingInstructionTest {
    target: Option<String>,
}

impl ProcessingInstructionTest {
    pub fn new(target: Option<String>) -> ProcessingInstructionTest {
        ProcessingInstructionTest { target }
    }
}

impl NodeTest for ProcessingInstructionTest {
    fn test(&self, _context: &context::EvaluationCtx, node: &Node) -> bool {
        if NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) == node.type_id() {
            if let Some(pi) = node.downcast::<ProcessingInstruction>() {
                return match (&self.target, pi.target()) {
                    (Some(target_name), node_target_name)
                        if target_name == &node_target_name.to_string() =>
                    {
                        true
                    },
                    (Some(_), _) => false,
                    (None, _) => true,
                };
            }
        }
        false
    }
}
