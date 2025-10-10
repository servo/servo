/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use markup5ever::{LocalName, Namespace, Prefix, QualName, local_name, namespace_prefix, ns};

use crate::ast::{
    Axis, BinaryOperator, Expression, FilterExpression, KindTest, Literal, LocationStepExpression,
    NodeTest, PathExpression, PredicateListExpression,
};
use crate::context::PredicateCtx;
use crate::{
    Attribute, Document, Dom, Element, Error, EvaluationCtx, Node, ProcessingInstruction, Value,
};

pub(crate) fn try_extract_nodeset<E, N: Node>(v: Value<N>) -> Result<Vec<N>, Error<E>> {
    match v {
        Value::Nodeset(ns) => Ok(ns),
        _ => Err(Error::NotANodeset),
    }
}

impl Expression {
    pub(crate) fn evaluate<D: Dom>(
        &self,
        context: &EvaluationCtx<D>,
    ) -> Result<Value<D::Node>, Error<D::JsError>> {
        match self {
            // And/Or expression are seperated because they can sometimes be evaluated
            // without evaluating both operands.
            Expression::Binary(left, BinaryOperator::And, right) => {
                let left_bool = left.evaluate(context)?.convert_to_boolean();
                let v = left_bool && right.evaluate(context)?.convert_to_boolean();
                Ok(Value::Boolean(v))
            },
            Expression::Binary(left, BinaryOperator::Or, right) => {
                let left_bool = left.evaluate(context)?.convert_to_boolean();
                let v = left_bool || right.evaluate(context)?.convert_to_boolean();
                Ok(Value::Boolean(v))
            },
            Expression::Binary(left, binary_operator, right) => {
                let left_value = left.evaluate(context)?;
                let right_value = right.evaluate(context)?;

                let value = match binary_operator {
                    BinaryOperator::Equal => (left_value == right_value).into(),
                    BinaryOperator::NotEqual => (left_value != right_value).into(),
                    BinaryOperator::LessThan => (left_value.convert_to_number() < right_value.convert_to_number()).into(),
                    BinaryOperator::GreaterThan => {
                        (left_value.convert_to_number() > right_value.convert_to_number()).into()
                    },
                    BinaryOperator::LessThanOrEqual => {
                        (left_value.convert_to_number() <= right_value.convert_to_number()).into()
                    },
                    BinaryOperator::GreaterThanOrEqual => {
                        (left_value.convert_to_number() >= right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Add => (left_value.convert_to_number() + right_value.convert_to_number()).into(),
                    BinaryOperator::Subtract => (left_value.convert_to_number() - right_value.convert_to_number()).into(),
                    BinaryOperator::Multiply => (left_value.convert_to_number() * right_value.convert_to_number()).into(),
                    BinaryOperator::Divide => (left_value.convert_to_number() / right_value.convert_to_number()).into(),
                    BinaryOperator::Modulo => (left_value.convert_to_number() % right_value.convert_to_number()).into(),
                    BinaryOperator::Union => {
                        let as_nodes =
                            |e: &Expression| e.evaluate(context).and_then(try_extract_nodeset);
                        let mut left_nodes = as_nodes(left)?;
                        let right_nodes = as_nodes(right)?;

                        left_nodes.extend(right_nodes);
                        Value::Nodeset(left_nodes)
                    },
                    _ => unreachable!("And/Or were handled above"),
                };

                Ok(value)
            },
            Expression::Negate(expr) => {
                let value = -expr.evaluate(context)?.convert_to_number();
                Ok(value.into())
            },
            Expression::Path(path_expr) => path_expr.evaluate(context),
            Expression::LocationStep(location_step_expression) => {
                location_step_expression.evaluate(context)
            },
            Expression::Filter(filter_expression) => filter_expression.evaluate(context),
            Expression::Literal(literal) => Ok(literal.evaluate::<D>()),
            Expression::Function(function) => function.evaluate(context),
            Expression::ContextItem => Ok(Value::Nodeset(vec![context.context_node.clone()])),
            Expression::Variable(_) => Err(Error::CannotUseVariables),
        }
    }
}

impl PathExpression {
    fn evaluate<D: Dom>(
        &self,
        context: &EvaluationCtx<D>,
    ) -> Result<Value<D::Node>, Error<D::JsError>> {
        // Use starting_node for absolute paths, context_node otherwise
        let mut current_nodes = if self.is_absolute {
            vec![context.starting_node.clone()]
        } else {
            vec![context.context_node.clone()]
        };

        // If path starts with '//', add an implicit descendant-or-self::node() step
        if self.has_implicit_descendant_or_self_step {
            current_nodes = current_nodes
                .iter()
                .flat_map(|node| node.traverse_preorder())
                .collect();
        }

        log::trace!("[PathExpr] Evaluating path expr: {:?}", self);

        let have_multiple_steps = self.steps.len() > 1;

        for step in &self.steps {
            let mut next_nodes = Vec::new();
            for node in current_nodes {
                let step_context = context.subcontext_for_node(node.clone());
                let step_result = step.evaluate(&step_context)?;
                match (have_multiple_steps, step_result) {
                    (_, Value::Nodeset(mut nodes)) => {
                        // as long as we evaluate to nodesets, keep going
                        next_nodes.append(&mut nodes);
                    },
                    (false, value) => {
                        log::trace!("[PathExpr] Got single primitive value: {:?}", value);
                        return Ok(value);
                    },
                    (true, value) => {
                        log::error!(
                            "Expected nodeset from step evaluation, got: {:?} node: {:?}, step: {:?}",
                            value,
                            node,
                            step
                        );
                        return Ok(value);
                    },
                }
            }
            current_nodes = next_nodes;
        }

        log::trace!("[PathExpr] Got nodes: {:?}", current_nodes);

        Ok(Value::Nodeset(current_nodes))
    }
}

#[derive(Debug)]
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

fn apply_node_test<D: Dom>(
    context: &EvaluationCtx<D>,
    test: &NodeTest,
    node: &D::Node,
) -> Result<bool, Error<D::JsError>> {
    let result = match test {
        NodeTest::Name(qname) => {
            let namespace = context
                .resolve_namespace(qname.prefix.as_deref())
                .map_err(Error::JsException)?
                .map(Namespace::from)
                .unwrap_or_default();

            let wanted_name = QualName {
                prefix: qname.prefix.as_deref().map(Prefix::from),
                ns: namespace,
                local: LocalName::from(qname.local_part.as_str()),
            };

            if let Some(element) = node.as_element() {
                let comparison_mode = if node.owner_document().is_html_document() {
                    NameTestComparisonMode::Html
                } else {
                    NameTestComparisonMode::XHtml
                };
                let element_qualname = QualName::new(
                    element.prefix(),
                    element.namespace().clone(),
                    element.local_name().clone(),
                );
                element_name_test(wanted_name, element_qualname, comparison_mode)
            } else if let Some(attribute) = node.as_attribute() {
                let attr_qualname = QualName::new(
                    attribute.prefix(),
                    attribute.namespace().clone(),
                    attribute.local_name().clone(),
                );
                // attributes are always compared with strict namespace matching
                let comparison_mode = NameTestComparisonMode::XHtml;
                element_name_test(wanted_name, attr_qualname, comparison_mode)
            } else {
                false
            }
        },
        NodeTest::Wildcard => node.as_element().is_some(),
        NodeTest::Kind(kind) => match kind {
            KindTest::PI(target) => {
                if let Some(processing_instruction) = node.as_processing_instruction() {
                    match (target, processing_instruction.target()) {
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
            KindTest::Comment => node.is_comment(),
            KindTest::Text => node.is_text(),
            KindTest::Node => true,
        },
    };
    Ok(result)
}

impl LocationStepExpression {
    fn evaluate<D: Dom>(
        &self,
        context: &EvaluationCtx<D>,
    ) -> Result<Value<D::Node>, Error<D::JsError>> {
        let nodes: Vec<D::Node> = match self.axis {
            Axis::Child => context.context_node.children().collect(),
            Axis::Descendant => context.context_node.traverse_preorder().skip(1).collect(),
            Axis::Parent => vec![context.context_node.parent()]
                .into_iter()
                .flatten()
                .collect(),
            Axis::Ancestor => context.context_node.inclusive_ancestors().skip(1).collect(),
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
                if let Some(element) = context.context_node.as_element() {
                    element
                        .attributes()
                        .map(|attribute| attribute.as_node())
                        .collect()
                } else {
                    vec![]
                }
            },
            Axis::Self_ => vec![context.context_node.clone()],
            Axis::DescendantOrSelf => context.context_node.traverse_preorder().collect(),
            Axis::AncestorOrSelf => context.context_node.inclusive_ancestors().collect(),
            Axis::Namespace => Vec::new(), // Namespace axis is not commonly implemented
        };

        log::trace!("[StepExpr] Axis {:?} got nodes {:?}", self.axis, nodes);

        // Filter nodes according to the step's node_test. Will error out if any NodeTest
        // application errors out.
        let filtered_nodes: Vec<D::Node> = nodes
            .into_iter()
            .map(|node| {
                apply_node_test(context, &self.node_test, &node)
                    .map(|matches| matches.then_some(node))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        log::trace!("[StepExpr] Filtering got nodes {:?}", filtered_nodes);

        if self.predicate_list.predicates.is_empty() {
            log::trace!(
                "[StepExpr] No predicates, returning nodes {:?}",
                filtered_nodes
            );
            Ok(Value::Nodeset(filtered_nodes))
        } else {
            // Apply predicates
            self.predicate_list
                .evaluate(context, filtered_nodes.clone())
        }
    }
}

impl PredicateListExpression {
    fn evaluate<D: Dom>(
        &self,
        context: &EvaluationCtx<D>,
        mut matched_nodes: Vec<D::Node>,
    ) -> Result<Value<D::Node>, Error<D::JsError>> {
        for predicate_expr in &self.predicates {
            let size = matched_nodes.len();
            let mut new_matched = Vec::new();

            for (i, node) in matched_nodes.iter().enumerate() {
                // 1-based position, per XPath spec
                let predicate_ctx: EvaluationCtx<D> = EvaluationCtx {
                    starting_node: context.starting_node.clone(),
                    context_node: node.clone(),
                    predicate_ctx: Some(PredicateCtx { index: i + 1, size }),
                    resolver: context.resolver.clone(),
                };

                let eval_result = predicate_expr.evaluate(&predicate_ctx);

                let keep = match eval_result {
                    Ok(Value::Number(number)) => (i + 1) as f64 == number,
                    Ok(Value::Boolean(boolean)) => boolean,
                    Ok(value) => value.convert_to_boolean(),
                    Err(_) => false,
                };

                if keep {
                    new_matched.push(node.clone());
                }
            }

            matched_nodes = new_matched;
            log::trace!(
                "[PredicateListExpr] Predicate {:?} matched nodes {:?}",
                predicate_expr,
                matched_nodes
            );
        }
        Ok(Value::Nodeset(matched_nodes))
    }
}

impl FilterExpression {
    fn evaluate<D: Dom>(
        &self,
        context: &EvaluationCtx<D>,
    ) -> Result<Value<D::Node>, Error<D::JsError>> {
        let primary_result = self.expression.evaluate(context)?;
        let have_predicates = !self.predicates.predicates.is_empty();

        match (have_predicates, &primary_result) {
            (false, _) => {
                log::trace!(
                    "[FilterExpr] No predicates, returning primary result: {:?}",
                    primary_result
                );
                Ok(primary_result)
            },
            (true, Value::Nodeset(vec)) => {
                let result_filtered_by_predicates = self.predicates.evaluate(context, vec.clone());
                log::trace!(
                    "[FilterExpr] Result filtered by predicates: {:?}",
                    result_filtered_by_predicates
                );
                result_filtered_by_predicates
            },
            // You can't use filtering expressions `[]` on other than node-sets
            (true, _) => Err(Error::NotANodeset),
        }
    }
}

impl Literal {
    fn evaluate<D: Dom>(&self) -> Value<D::Node> {
        match self {
            Literal::Integer(integer) => Value::Number(*integer as f64),
            Literal::Decimal(decimal) => Value::Number(*decimal),
            Literal::String(s) => Value::String(s.into()),
        }
    }
}
