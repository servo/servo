/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::fmt;

use html5ever::QualName;

use super::node_test::NodeTest;
use super::parser::{
    AdditiveOp, Axis, EqualityOp, Expr, FilterExpr, Literal, MultiplicativeOp, NumericLiteral,
    PathExpr, PredicateExpr, PredicateListExpr, PrimaryExpr, RelationalOp, StepExpr,
};
use super::Value::{Boolean, Number};
use super::{context, node_test, parser, Node, Value};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::{Castable, NodeTypeId};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::xmlname::validate_and_extract;
use crate::dom::element::Element;
use crate::dom::node::ShadowIncluding;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    NotANodeset,
    InvalidPath,
    UnknownFunction { name: QualName },
    UnknownVariable { name: QualName },
    UnknownNamespace { prefix: String },
    InvalidQName { qname: parser::QName },
    FunctionEvaluation { fname: String },
    Internal { msg: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotANodeset => write!(f, "expression did not evaluate to a nodeset"),
            Error::InvalidPath => write!(f, "invalid path expression"),
            Error::UnknownFunction { name } => write!(f, "unknown function {:?}", name),
            Error::UnknownVariable { name } => write!(f, "unknown variable {:?}", name),
            Error::UnknownNamespace { prefix } => {
                write!(f, "unknown namespace prefix {:?}", prefix)
            },
            Error::InvalidQName { qname } => {
                write!(f, "invalid QName {:?}", qname)
            },
            Error::FunctionEvaluation { fname } => {
                write!(f, "error while evaluating function: {}", fname)
            },
            Error::Internal { msg } => {
                write!(f, "internal error: {}", msg)
            },
        }
    }
}

impl std::error::Error for Error {}

pub fn try_extract_nodeset(v: Value) -> Result<Vec<DomRoot<Node>>, Error> {
    match v {
        Value::Nodeset(ns) => Ok(ns),
        _ => Err(Error::NotANodeset),
    }
}

pub trait NodeHelpers {
    /// Returns e.g. "rect" for `<svg:rect>`
    fn local_name(&self) -> Option<String>;
    /// Returns e.g. "svg:rect" for `<svg:rect>`
    fn name(&self) -> Option<String>;
    /// Returns e.g. the SVG namespace URI for `<svg:rect>`
    fn namespace_uri(&self) -> Option<String>;
    fn string_value(&self) -> String;
}

impl NodeHelpers for Node {
    fn local_name(&self) -> Option<String> {
        if matches!(Node::type_id(self), NodeTypeId::Element(_)) {
            let element = self.downcast::<Element>().unwrap();
            Some(element.local_name().to_string())
        } else {
            None
        }
    }

    fn name(&self) -> Option<String> {
        if matches!(Node::type_id(self), NodeTypeId::Element(_)) {
            let element = self.downcast::<Element>().unwrap();
            if let Some(prefix) = element.prefix().as_ref() {
                Some(format!("{}:{}", prefix, element.local_name()))
            } else {
                Some(element.local_name().to_string())
            }
        } else {
            None
        }
    }
    fn namespace_uri(&self) -> Option<String> {
        if matches!(Node::type_id(self), NodeTypeId::Element(_)) {
            let element = self.downcast::<Element>().unwrap();
            Some(element.namespace().to_string())
        } else {
            None
        }
    }
    fn string_value(&self) -> String {
        self.GetTextContent().unwrap_or_default().to_string()
    }
}

pub trait Evaluatable: fmt::Debug {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error>;
    /// Returns true if this expression evaluates to a primitive value, without needing to touch the DOM
    fn is_primitive(&self) -> bool;
}

impl<T: ?Sized> Evaluatable for Box<T>
where
    T: Evaluatable,
{
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        (**self).evaluate(context)
    }

    fn is_primitive(&self) -> bool {
        (**self).is_primitive()
    }
}

impl Evaluatable for Expr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        match self {
            Expr::And(left, right) => {
                let left_bool = left.evaluate(context)?.boolean();
                let v = left_bool && right.evaluate(context)?.boolean();
                Ok(Boolean(v))
            },
            Expr::Or(left, right) => {
                let left_bool = left.evaluate(context)?.boolean();
                let v = left_bool || right.evaluate(context)?.boolean();
                Ok(Boolean(v))
            },
            Expr::Equality(left, equality_op, right) => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;

                fn str_vals(nodes: &[DomRoot<Node>]) -> HashSet<String> {
                    nodes
                        .iter()
                        .map(|n| n.GetTextContent().unwrap_or_default().to_string())
                        .collect()
                }

                fn num_vals(nodes: &[DomRoot<Node>]) -> Vec<f64> {
                    nodes
                        .iter()
                        .map(|n| {
                            Value::String(n.GetTextContent().unwrap_or_default().into()).number()
                        })
                        .collect()
                }

                let is_equal = match (&left_val, &right_val) {
                    (Value::Nodeset(left_nodes), Value::Nodeset(right_nodes)) => {
                        let left_strings = str_vals(left_nodes);
                        let right_strings = str_vals(right_nodes);
                        !left_strings.is_disjoint(&right_strings)
                    },
                    (&Value::Nodeset(ref nodes), &Number(val)) |
                    (&Number(val), &Value::Nodeset(ref nodes)) => {
                        let numbers = num_vals(nodes);
                        numbers.iter().any(|n| *n == val)
                    },
                    (&Value::Nodeset(ref nodes), &Value::String(ref val)) |
                    (&Value::String(ref val), &Value::Nodeset(ref nodes)) => {
                        let strings = str_vals(nodes);
                        strings.contains(val)
                    },
                    (&Boolean(_), _) | (_, &Boolean(_)) => {
                        left_val.boolean() == right_val.boolean()
                    },
                    (&Number(_), _) | (_, &Number(_)) => left_val.number() == right_val.number(),
                    _ => left_val.string() == right_val.string(),
                };

                let v = match equality_op {
                    EqualityOp::Eq => is_equal,
                    EqualityOp::NotEq => !is_equal,
                };

                Ok(Boolean(v))
            },
            Expr::Relational(left, relational_op, right) => {
                let left_val = left.evaluate(context)?.number();
                let right_val = right.evaluate(context)?.number();

                let v = match relational_op {
                    RelationalOp::Lt => left_val < right_val,
                    RelationalOp::Gt => left_val > right_val,
                    RelationalOp::LtEq => left_val <= right_val,
                    RelationalOp::GtEq => left_val >= right_val,
                };
                Ok(Boolean(v))
            },
            Expr::Additive(left, additive_op, right) => {
                let left_val = left.evaluate(context)?.number();
                let right_val = right.evaluate(context)?.number();

                let v = match additive_op {
                    AdditiveOp::Add => left_val + right_val,
                    AdditiveOp::Sub => left_val - right_val,
                };
                Ok(Number(v))
            },
            Expr::Multiplicative(left, multiplicative_op, right) => {
                let left_val = left.evaluate(context)?.number();
                let right_val = right.evaluate(context)?.number();

                let v = match multiplicative_op {
                    MultiplicativeOp::Mul => left_val * right_val,
                    MultiplicativeOp::Div => left_val / right_val,
                    MultiplicativeOp::Mod => left_val % right_val,
                };
                Ok(Number(v))
            },
            Expr::Unary(_, expr) => {
                let v = expr.evaluate(context)?.number();

                Ok(Number(-v))
            },
            Expr::Union(left, right) => {
                let as_nodes = |e: &Expr| e.evaluate(context).and_then(try_extract_nodeset);

                let mut left_nodes = as_nodes(left)?;
                let right_nodes = as_nodes(right)?;

                left_nodes.extend(right_nodes);
                Ok(Value::Nodeset(left_nodes))
            },
            Expr::Path(path_expr) => path_expr.evaluate(context),
        }
    }

    fn is_primitive(&self) -> bool {
        match self {
            Expr::Or(left, right) => left.is_primitive() && right.is_primitive(),
            Expr::And(left, right) => left.is_primitive() && right.is_primitive(),
            Expr::Equality(left, _, right) => left.is_primitive() && right.is_primitive(),
            Expr::Relational(left, _, right) => left.is_primitive() && right.is_primitive(),
            Expr::Additive(left, _, right) => left.is_primitive() && right.is_primitive(),
            Expr::Multiplicative(left, _, right) => left.is_primitive() && right.is_primitive(),
            Expr::Unary(_, expr) => expr.is_primitive(),
            Expr::Union(_, _) => false,
            Expr::Path(path_expr) => path_expr.is_primitive(),
        }
    }
}

impl Evaluatable for PathExpr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        let mut current_nodes = vec![context.context_node.clone()];

        // If path starts with '//', add an implicit descendant-or-self::node() step
        if self.is_descendant {
            current_nodes = current_nodes
                .iter()
                .flat_map(|n| n.traverse_preorder(ShadowIncluding::No))
                .collect();
        }

        trace!("[PathExpr] Evaluating path expr: {:?}", self);

        let have_multiple_steps = self.steps.len() > 1;

        for step in &self.steps {
            let mut next_nodes = Vec::new();
            for node in current_nodes {
                let step_context = context.subcontext_for_node(&node);
                let step_result = step.evaluate(&step_context)?;
                match (have_multiple_steps, step_result) {
                    (_, Value::Nodeset(mut nodes)) => {
                        // as long as we evaluate to nodesets, keep going
                        next_nodes.append(&mut nodes);
                    },
                    (false, value) => {
                        trace!("[PathExpr] Got single primitive value: {:?}", value);
                        return Ok(value);
                    },
                    (true, value) => {
                        error!(
                        "Expected nodeset from step evaluation, got: {:?} node: {:?}, step: {:?}",
                        value, node, step
                    );
                        return Ok(value);
                    },
                }
            }
            current_nodes = next_nodes;
        }

        trace!("[PathExpr] Got nodes: {:?}", current_nodes);

        Ok(Value::Nodeset(current_nodes))
    }

    fn is_primitive(&self) -> bool {
        !self.is_absolute &&
            !self.is_descendant &&
            self.steps.len() == 1 &&
            self.steps[0].is_primitive()
    }
}

impl TryFrom<&parser::QName> for QualName {
    type Error = Error;

    fn try_from(qname: &parser::QName) -> Result<Self, Self::Error> {
        let qname_as_str = qname.to_string();
        if let Ok((ns, prefix, local)) = validate_and_extract(None, &qname_as_str) {
            Ok(QualName { prefix, ns, local })
        } else {
            Err(Error::InvalidQName {
                qname: qname.clone(),
            })
        }
    }
}

fn apply_node_test(
    test: &parser::NodeTest,
    context: &context::EvaluationCtx,
    node: &Node,
) -> Result<bool, Error> {
    let result = match test {
        parser::NodeTest::Name(qname) => {
            // Convert the unvalidated "parser QName" into the proper QualName structure
            let qname_validated: QualName = qname.try_into()?;
            let name_test = node_test::NameTest {
                qname: qname_validated,
                strict_ns_comparison: node.owner_doc().is_xhtml_document(),
            };
            node_test::ElementTest::new(name_test).test(context, node)
        },
        parser::NodeTest::Wildcard => true,
        parser::NodeTest::Kind(kind) => match kind {
            parser::KindTest::PI(target) => {
                node_test::ProcessingInstructionTest::new(target.clone()).test(context, node)
            },
            parser::KindTest::Comment => node_test::CommentTest.test(context, node),
            parser::KindTest::Text => node_test::TextTest.test(context, node),
            parser::KindTest::Node => true,
        },
    };
    Ok(result)
}

impl Evaluatable for StepExpr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        match self {
            StepExpr::Filter(filter_expr) => filter_expr.evaluate(context),
            StepExpr::Axis(axis_step) => {
                let nodes: Vec<DomRoot<Node>> = match axis_step.axis {
                    Axis::Child => context.context_node.children().collect(),
                    Axis::Descendant => context
                        .context_node
                        .traverse_preorder(ShadowIncluding::No)
                        .skip(1)
                        .collect(),
                    Axis::Parent => vec![context.context_node.GetParentNode()]
                        .into_iter()
                        .flatten()
                        .collect(),
                    Axis::Ancestor => context.context_node.ancestors().collect(),
                    Axis::Following => context
                        .context_node
                        .following_nodes(&context.context_node)
                        .skip(1)
                        .collect(),
                    Axis::Preceding => context
                        .context_node
                        .preceding_nodes(&context.context_node)
                        .skip(1)
                        .collect(),
                    Axis::FollowingSibling => context.context_node.following_siblings().collect(),
                    Axis::PrecedingSibling => context.context_node.preceding_siblings().collect(),
                    Axis::Attribute => {
                        if matches!(Node::type_id(&context.context_node), NodeTypeId::Element(_)) {
                            let element = context.context_node.downcast::<Element>().unwrap();
                            element
                                .attrs()
                                .iter()
                                .map(|attr| attr.upcast::<Node>())
                                .map(DomRoot::from_ref)
                                .collect()
                        } else {
                            vec![]
                        }
                    },
                    Axis::Self_ => vec![context.context_node.clone()],
                    Axis::DescendantOrSelf => context
                        .context_node
                        .traverse_preorder(ShadowIncluding::No)
                        .collect(),
                    Axis::AncestorOrSelf => context
                        .context_node
                        .inclusive_ancestors(ShadowIncluding::No)
                        .collect(),
                    Axis::Namespace => Vec::new(), // Namespace axis is not commonly implemented
                };

                trace!("[StepExpr] Axis {:?} got nodes {:?}", axis_step.axis, nodes);

                // Filter nodes according to the step's node_test. Will error out if any NodeTest
                // application errors out.
                let filtered_nodes: Vec<DomRoot<Node>> = nodes
                    .into_iter()
                    .map(|node| {
                        apply_node_test(&axis_step.node_test, context, &node)
                            .map(|matches| matches.then_some(node))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect();

                trace!("[StepExpr] Filtering got nodes {:?}", filtered_nodes);

                if axis_step.predicates.predicates.is_empty() {
                    trace!(
                        "[StepExpr] No predicates, returning nodes {:?}",
                        filtered_nodes
                    );
                    Ok(Value::Nodeset(filtered_nodes))
                } else {
                    // Apply predicates
                    let predicate_list_subcontext = context
                        .update_predicate_nodes(filtered_nodes.iter().map(|n| &**n).collect());
                    axis_step.predicates.evaluate(&predicate_list_subcontext)
                }
            },
        }
    }

    fn is_primitive(&self) -> bool {
        match self {
            StepExpr::Filter(filter_expr) => filter_expr.is_primitive(),
            StepExpr::Axis(_) => false,
        }
    }
}

impl Evaluatable for PredicateListExpr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        if let Some(ref predicate_nodes) = context.predicate_nodes {
            // Initializing: every node the predicates act on is matched
            let mut matched_nodes: Vec<DomRoot<Node>> = predicate_nodes.clone();

            // apply each predicate to the nodes matched by the previous predicate
            for predicate_expr in &self.predicates {
                let context_for_predicate =
                    context.update_predicate_nodes(matched_nodes.iter().map(|n| &**n).collect());

                let narrowed_nodes = predicate_expr
                    .evaluate(&context_for_predicate)
                    .and_then(try_extract_nodeset)?;
                matched_nodes = narrowed_nodes;
                trace!(
                    "[PredicateListExpr] Predicate {:?} matched nodes {:?}",
                    predicate_expr,
                    matched_nodes
                );
            }
            Ok(Value::Nodeset(matched_nodes))
        } else {
            Err(Error::Internal {
                msg: "[PredicateListExpr] No nodes on stack for predicate to operate on"
                    .to_string(),
            })
        }
    }

    fn is_primitive(&self) -> bool {
        self.predicates.len() == 1 && self.predicates[0].is_primitive()
    }
}

impl Evaluatable for PredicateExpr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        let narrowed_nodes: Result<Vec<DomRoot<Node>>, Error> = context
            .subcontext_iter_for_nodes()
            .filter_map(|ctx| {
                if let Some(predicate_ctx) = ctx.predicate_ctx {
                    let eval_result = self.expr.evaluate(&ctx);

                    let v = match eval_result {
                        Ok(Number(v)) => Ok(predicate_ctx.index == v as usize),
                        Ok(v) => Ok(v.boolean()),
                        Err(e) => Err(e),
                    };

                    match v {
                        Ok(true) => Some(Ok(ctx.context_node)),
                        Ok(false) => None,
                        Err(e) => Some(Err(e)),
                    }
                } else {
                    Some(Err(Error::Internal {
                        msg: "[PredicateExpr] No predicate context set".to_string(),
                    }))
                }
            })
            .collect();

        Ok(Value::Nodeset(narrowed_nodes?))
    }

    fn is_primitive(&self) -> bool {
        self.expr.is_primitive()
    }
}

impl Evaluatable for FilterExpr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        let primary_result = self.primary.evaluate(context)?;
        let have_predicates = !self.predicates.predicates.is_empty();

        match (have_predicates, &primary_result) {
            (false, _) => {
                trace!(
                    "[FilterExpr] No predicates, returning primary result: {:?}",
                    primary_result
                );
                Ok(primary_result)
            },
            (true, Value::Nodeset(vec)) => {
                let predicate_list_subcontext =
                    context.update_predicate_nodes(vec.iter().map(|n| &**n).collect());
                let result_filtered_by_predicates =
                    self.predicates.evaluate(&predicate_list_subcontext);
                trace!(
                    "[FilterExpr] Result filtered by predicates: {:?}",
                    result_filtered_by_predicates
                );
                result_filtered_by_predicates
            },
            // You can't use filtering expressions `[]` on other than node-sets
            (true, _) => Err(Error::NotANodeset),
        }
    }

    fn is_primitive(&self) -> bool {
        self.predicates.predicates.is_empty() && self.primary.is_primitive()
    }
}

impl Evaluatable for PrimaryExpr {
    fn evaluate(&self, context: &context::EvaluationCtx) -> Result<Value, Error> {
        match self {
            PrimaryExpr::Literal(literal) => literal.evaluate(context),
            PrimaryExpr::Variable(_qname) => todo!(),
            PrimaryExpr::Parenthesized(expr) => expr.evaluate(context),
            PrimaryExpr::ContextItem => Ok(Value::Nodeset(vec![context.context_node.clone()])),
            PrimaryExpr::Function(core_function) => core_function.evaluate(context),
        }
    }

    fn is_primitive(&self) -> bool {
        match self {
            PrimaryExpr::Literal(_) => true,
            PrimaryExpr::Variable(_qname) => false,
            PrimaryExpr::Parenthesized(expr) => expr.is_primitive(),
            PrimaryExpr::ContextItem => false,
            PrimaryExpr::Function(_) => false,
        }
    }
}

impl Evaluatable for Literal {
    fn evaluate(&self, _context: &context::EvaluationCtx) -> Result<Value, Error> {
        match self {
            Literal::Numeric(numeric_literal) => match numeric_literal {
                // We currently make no difference between ints and floats
                NumericLiteral::Integer(v) => Ok(Value::Number(*v as f64)),
                NumericLiteral::Decimal(v) => Ok(Value::Number(*v)),
            },
            Literal::String(s) => Ok(Value::String(s.into())),
        }
    }

    fn is_primitive(&self) -> bool {
        true
    }
}
