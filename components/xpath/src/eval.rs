/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter;

use markup5ever::{QualName, local_name, ns};

use crate::ast::{
    Axis, BinaryOperator, Expression, FilterExpression, KindTest, Literal, LocationStepExpression,
    NodeTest, PathExpression, PredicateListExpression,
};
use crate::context::PredicateCtx;
use crate::{
    Attribute, Dom, Element, Error, EvaluationCtx, Node, NodeSet, ProcessingInstruction, Value,
};

pub(crate) fn try_extract_nodeset<N: Node>(v: Value<N>) -> Result<NodeSet<N>, Error> {
    match v {
        Value::NodeSet(node_set) => Ok(node_set),
        _ => Err(Error::NotANodeset),
    }
}

impl Expression {
    pub(crate) fn evaluate<D: Dom>(
        &self,
        context: &EvaluationCtx<D>,
    ) -> Result<Value<D::Node>, Error> {
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
                    BinaryOperator::LessThan => {
                        (left_value.convert_to_number() < right_value.convert_to_number()).into()
                    },
                    BinaryOperator::GreaterThan => {
                        (left_value.convert_to_number() > right_value.convert_to_number()).into()
                    },
                    BinaryOperator::LessThanOrEqual => {
                        (left_value.convert_to_number() <= right_value.convert_to_number()).into()
                    },
                    BinaryOperator::GreaterThanOrEqual => {
                        (left_value.convert_to_number() >= right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Add => {
                        (left_value.convert_to_number() + right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Subtract => {
                        (left_value.convert_to_number() - right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Multiply => {
                        (left_value.convert_to_number() * right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Divide => {
                        (left_value.convert_to_number() / right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Modulo => {
                        (left_value.convert_to_number() % right_value.convert_to_number()).into()
                    },
                    BinaryOperator::Union => {
                        let as_nodes =
                            |e: &Expression| e.evaluate(context).and_then(try_extract_nodeset);
                        let mut left_nodes = as_nodes(left)?;
                        let right_nodes = as_nodes(right)?;

                        left_nodes.extend(right_nodes);
                        left_nodes.sort();
                        Value::NodeSet(left_nodes)
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
            Expression::ContextItem => {
                let mut result = NodeSet::default();
                result.push(context.context_node.clone());
                Ok(Value::NodeSet(result))
            },
            Expression::Variable(_) => Err(Error::CannotUseVariables),
        }
    }
}

impl PathExpression {
    fn evaluate<D: Dom>(&self, context: &EvaluationCtx<D>) -> Result<Value<D::Node>, Error> {
        // Use root node for absolute paths, context_node otherwise
        let starting_node = if self.is_absolute {
            context.context_node.get_root_node()
        } else {
            context.context_node.clone()
        };

        // If path starts with '//', add an implicit descendant-or-self::node() step
        let mut current_nodes = NodeSet::default();
        if self.has_implicit_descendant_or_self_step {
            for node in starting_node.traverse_preorder() {
                current_nodes.push(node);
            }
        } else {
            current_nodes.push(starting_node);
        }
        current_nodes.assume_sorted();

        log::trace!("[PathExpr] Evaluating path expr: {:?}", self);

        let have_multiple_steps = self.steps.len() > 1;

        for step_expression in &self.steps {
            let mut next_nodes = NodeSet::default();
            for node in current_nodes {
                let step_context = context.subcontext_for_node(node.clone());
                let step_result = step_expression.evaluate(&step_context)?;
                match (have_multiple_steps, step_result) {
                    (_, Value::NodeSet(nodes)) => {
                        // as long as we evaluate to nodesets, keep going
                        next_nodes.extend(nodes);
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
                            step_expression
                        );
                        return Ok(value);
                    },
                }
            }
            current_nodes = next_nodes;
        }

        log::trace!("[PathExpr] Got nodes: {:?}", current_nodes);

        Ok(Value::NodeSet(current_nodes))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum NameTestComparisonMode {
    /// Namespaces must match exactly
    XHtml,
    /// Missing namespace information is treated as the HTML namespace
    Html,
}

pub(crate) fn element_name_test(
    expected_name: &QualName,
    actual_name: QualName,
    comparison_mode: NameTestComparisonMode,
) -> bool {
    if expected_name.prefix.is_none() && expected_name.local == local_name!("*") {
        return true;
    }

    let should_compare_namespaces =
        comparison_mode == NameTestComparisonMode::XHtml || expected_name.ns != ns!();
    if should_compare_namespaces && expected_name.ns != actual_name.ns {
        return false;
    }

    if expected_name.local == local_name!("*") {
        return true;
    }

    expected_name.local == actual_name.local
}

fn apply_node_test<D: Dom>(test: &NodeTest, node: &D::Node) -> Result<bool, Error> {
    let result = match test {
        NodeTest::Name(expected_name) => {
            if let Some(element) = node.as_element() {
                let comparison_mode = if element.is_html_element_in_html_document() {
                    NameTestComparisonMode::Html
                } else {
                    NameTestComparisonMode::XHtml
                };
                let actual_name = QualName::new(
                    element.prefix(),
                    element.namespace().clone(),
                    element.local_name().clone(),
                );
                element_name_test(expected_name, actual_name, comparison_mode)
            } else if let Some(attribute) = node.as_attribute() {
                let actual_name = QualName::new(
                    attribute.prefix(),
                    attribute.namespace().clone(),
                    attribute.local_name().clone(),
                );
                // attributes are always compared with strict namespace matching
                let comparison_mode = NameTestComparisonMode::XHtml;
                element_name_test(expected_name, actual_name, comparison_mode)
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
    fn evaluate<D: Dom>(&self, context: &EvaluationCtx<D>) -> Result<Value<D::Node>, Error> {
        let mut nodes: NodeSet<D::Node> = match self.axis {
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
                    Default::default()
                }
            },
            Axis::Self_ => iter::once(context.context_node.clone()).collect(),
            Axis::DescendantOrSelf => context.context_node.traverse_preorder().collect(),
            Axis::AncestorOrSelf => context.context_node.inclusive_ancestors().collect(),
            Axis::Namespace => Default::default(), // Namespace axis is not commonly implemented
        };

        if matches!(
            self.axis,
            Axis::Child |
                Axis::Descendant |
                Axis::Parent |
                Axis::Following |
                Axis::FollowingSibling |
                Axis::Attribute |
                Axis::Self_ |
                Axis::DescendantOrSelf
        ) {
            // The elements on these axis values are already in tree order
            nodes.assume_sorted();
        } else {
            // The elements on these axis values are in inverse tree order
            nodes.reverse();
            nodes.assume_sorted();
        }

        log::trace!("[StepExpr] Axis {:?} got nodes {:?}", self.axis, nodes);

        // Filter nodes according to the step's node_test. Will error out if any NodeTest
        // application errors out.
        // FIXME: Invent something like try_retain and use it here
        let filtered_nodes: NodeSet<D::Node> = nodes
            .into_iter()
            .filter_map(|node| match apply_node_test::<D>(&self.node_test, &node) {
                Ok(false) => None,
                Ok(true) => Some(Ok(node)),
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<NodeSet<_>, _>>()?;

        log::trace!("[StepExpr] Filtering got nodes {:?}", filtered_nodes);

        if self.predicate_list.predicates.is_empty() {
            log::trace!(
                "[StepExpr] No predicates, returning nodes {:?}",
                filtered_nodes
            );
            Ok(Value::NodeSet(filtered_nodes))
        } else {
            // Apply predicates
            self.predicate_list.evaluate::<D>(filtered_nodes)
        }
    }
}

impl PredicateListExpression {
    fn evaluate<D: Dom>(
        &self,
        mut matched_nodes: NodeSet<D::Node>,
    ) -> Result<Value<D::Node>, Error> {
        for predicate_expr in &self.predicates {
            let size = matched_nodes.len();

            // 1-based position, per XPath spec
            let mut position = 1;
            matched_nodes.retain(|node| {
                let predicate_ctx: EvaluationCtx<D> = EvaluationCtx {
                    context_node: node.clone(),
                    predicate_ctx: Some(PredicateCtx {
                        index: position,
                        size,
                    }),
                };
                let eval_result = predicate_expr.evaluate(&predicate_ctx);

                let keep = match eval_result {
                    Ok(Value::Number(number)) => position as f64 == number,
                    Ok(Value::Boolean(boolean)) => boolean,
                    Ok(value) => value.convert_to_boolean(),
                    Err(_) => false,
                };
                position += 1;

                keep
            });

            log::trace!(
                "[PredicateListExpr] Predicate {:?} matched nodes {:?}",
                predicate_expr,
                matched_nodes
            );
        }
        Ok(Value::NodeSet(matched_nodes))
    }
}

impl FilterExpression {
    fn evaluate<D: Dom>(&self, context: &EvaluationCtx<D>) -> Result<Value<D::Node>, Error> {
        debug_assert!(!self.predicates.predicates.is_empty());

        let Value::NodeSet(node_set) = self.expression.evaluate(context)? else {
            // You can't use filtering expressions `[]` on other than node-sets
            return Err(Error::NotANodeset);
        };
        let result_filtered_by_predicates = self.predicates.evaluate::<D>(node_set);
        log::trace!(
            "[FilterExpr] Result filtered by predicates: {:?}",
            result_filtered_by_predicates
        );
        result_filtered_by_predicates
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
