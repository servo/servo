/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use html5ever::{local_name, namespace_prefix, namespace_url, ns, QualName};

use super::parser::{
    AdditiveOp, Axis, EqualityOp, Expr, FilterExpr, KindTest, Literal, MultiplicativeOp, NodeTest,
    NumericLiteral, PathExpr, PredicateExpr, PredicateListExpr, PrimaryExpr,
    QName as ParserQualName, RelationalOp, StepExpr, UnaryOp,
};
use super::{EvaluationCtx, Value};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, NodeTypeId};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::xmlname::validate_and_extract;
use crate::dom::element::Element;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::processinginstruction::ProcessingInstruction;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Error {
    NotANodeset,
    InvalidPath,
    UnknownFunction { name: QualName },
    UnknownVariable { name: QualName },
    UnknownNamespace { prefix: String },
    InvalidQName { qname: ParserQualName },
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

pub(crate) fn try_extract_nodeset(v: Value) -> Result<Vec<DomRoot<Node>>, Error> {
    match v {
        Value::Nodeset(ns) => Ok(ns),
        _ => Err(Error::NotANodeset),
    }
}

pub(crate) trait Evaluatable: fmt::Debug {
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error>;
    /// Returns true if this expression evaluates to a primitive value, without needing to touch the DOM
    fn is_primitive(&self) -> bool;
}

impl<T: ?Sized> Evaluatable for Box<T>
where
    T: Evaluatable,
{
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
        (**self).evaluate(context)
    }

    fn is_primitive(&self) -> bool {
        (**self).is_primitive()
    }
}

impl Evaluatable for Expr {
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
        match self {
            Expr::And(left, right) => {
                let left_bool = left.evaluate(context)?.boolean();
                let v = left_bool && right.evaluate(context)?.boolean();
                Ok(Value::Boolean(v))
            },
            Expr::Or(left, right) => {
                let left_bool = left.evaluate(context)?.boolean();
                let v = left_bool || right.evaluate(context)?.boolean();
                Ok(Value::Boolean(v))
            },
            Expr::Equality(left, equality_op, right) => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;

                let v = match equality_op {
                    EqualityOp::Eq => left_val == right_val,
                    EqualityOp::NotEq => left_val != right_val,
                };

                Ok(Value::Boolean(v))
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
                Ok(Value::Boolean(v))
            },
            Expr::Additive(left, additive_op, right) => {
                let left_val = left.evaluate(context)?.number();
                let right_val = right.evaluate(context)?.number();

                let v = match additive_op {
                    AdditiveOp::Add => left_val + right_val,
                    AdditiveOp::Sub => left_val - right_val,
                };
                Ok(Value::Number(v))
            },
            Expr::Multiplicative(left, multiplicative_op, right) => {
                let left_val = left.evaluate(context)?.number();
                let right_val = right.evaluate(context)?.number();

                let v = match multiplicative_op {
                    MultiplicativeOp::Mul => left_val * right_val,
                    MultiplicativeOp::Div => left_val / right_val,
                    MultiplicativeOp::Mod => left_val % right_val,
                };
                Ok(Value::Number(v))
            },
            Expr::Unary(unary_op, expr) => {
                let v = expr.evaluate(context)?.number();

                match unary_op {
                    UnaryOp::Minus => Ok(Value::Number(-v)),
                }
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
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
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

impl TryFrom<&ParserQualName> for QualName {
    type Error = Error;

    fn try_from(qname: &ParserQualName) -> Result<Self, Self::Error> {
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

pub(crate) enum NameTestComparisonMode {
    /// Namespaces must match exactly
    XHtml,
    /// Missing namespace information is treated as the HTML namespace
    Html,
}

pub(crate) fn element_name_test(
    expected_name: QualName,
    element_qualname: QualName,
    comparison_mode: NameTestComparisonMode,
) -> bool {
    let is_wildcard = expected_name.local == local_name!("*");

    let test_prefix = expected_name
        .prefix
        .clone()
        .unwrap_or(namespace_prefix!(""));
    let test_ns_uri = match test_prefix {
        namespace_prefix!("*") => ns!(*),
        namespace_prefix!("html") => ns!(html),
        namespace_prefix!("xml") => ns!(xml),
        namespace_prefix!("xlink") => ns!(xlink),
        namespace_prefix!("svg") => ns!(svg),
        namespace_prefix!("mathml") => ns!(mathml),
        namespace_prefix!("") => {
            if matches!(comparison_mode, NameTestComparisonMode::XHtml) {
                ns!()
            } else {
                ns!(html)
            }
        },
        _ => {
            // We don't support custom namespaces, use fallback or panic depending on strictness
            if matches!(comparison_mode, NameTestComparisonMode::XHtml) {
                panic!("Unrecognized namespace prefix: {}", test_prefix)
            } else {
                ns!(html)
            }
        },
    };

    if is_wildcard {
        test_ns_uri == element_qualname.ns
    } else {
        test_ns_uri == element_qualname.ns && expected_name.local == element_qualname.local
    }
}

fn apply_node_test(test: &NodeTest, node: &Node) -> Result<bool, Error> {
    let result = match test {
        NodeTest::Name(qname) => {
            // Convert the unvalidated "parser QualName" into the proper QualName structure
            let wanted_name: QualName = qname.try_into()?;
            if matches!(node.type_id(), NodeTypeId::Element(_)) {
                let element = node.downcast::<Element>().unwrap();
                let comparison_mode = if node.owner_doc().is_xhtml_document() {
                    NameTestComparisonMode::XHtml
                } else {
                    NameTestComparisonMode::Html
                };
                let element_qualname = QualName::new(
                    element.prefix().as_ref().cloned(),
                    element.namespace().clone(),
                    element.local_name().clone(),
                );
                element_name_test(wanted_name, element_qualname, comparison_mode)
            } else {
                false
            }
        },
        NodeTest::Wildcard => true,
        NodeTest::Kind(kind) => match kind {
            KindTest::PI(target) => {
                if NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) ==
                    node.type_id()
                {
                    let pi = node.downcast::<ProcessingInstruction>().unwrap();
                    match (target, pi.target()) {
                        (Some(target_name), node_target_name)
                            if target_name == &node_target_name.to_string() =>
                        {
                            true
                        },
                        (Some(_), _) => false,
                        (None, _) => true,
                    }
                } else {
                    false
                }
            },
            KindTest::Comment => matches!(
                node.type_id(),
                NodeTypeId::CharacterData(CharacterDataTypeId::Comment)
            ),
            KindTest::Text => matches!(
                node.type_id(),
                NodeTypeId::CharacterData(CharacterDataTypeId::Text(_))
            ),
            KindTest::Node => true,
        },
    };
    Ok(result)
}

impl Evaluatable for StepExpr {
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
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
                        apply_node_test(&axis_step.node_test, &node)
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
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
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
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
        let narrowed_nodes: Result<Vec<DomRoot<Node>>, Error> = context
            .subcontext_iter_for_nodes()
            .filter_map(|ctx| {
                if let Some(predicate_ctx) = ctx.predicate_ctx {
                    let eval_result = self.expr.evaluate(&ctx);

                    let v = match eval_result {
                        Ok(Value::Number(v)) => Ok(predicate_ctx.index == v as usize),
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
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
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
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
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
    fn evaluate(&self, _context: &EvaluationCtx) -> Result<Value, Error> {
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
