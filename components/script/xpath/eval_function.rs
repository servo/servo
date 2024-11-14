/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::context::EvaluationCtx;
use super::eval::{try_extract_nodeset, Error, Evaluatable, NodeHelpers};
use super::parser::CoreFunction;
use super::Value;

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-normalize-space>
pub fn normalize_space(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_whitespace = true; // Handles leading whitespace

    for c in s.chars() {
        match c {
            '\x20' | '\x09' | '\x0D' | '\x0A' => {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                }
            },
            other => {
                result.push(other);
                last_was_whitespace = false;
            },
        }
    }

    if last_was_whitespace {
        result.pop();
    }

    result
}

impl Evaluatable for CoreFunction {
    fn evaluate(&self, context: &EvaluationCtx) -> Result<Value, Error> {
        match self {
            CoreFunction::Last => {
                let predicate_ctx = context.predicate_ctx.ok_or_else(|| Error::Internal {
                    msg: "[CoreFunction] last() is only usable as a predicate".to_string(),
                })?;
                Ok(Value::Number(predicate_ctx.size as f64))
            },
            CoreFunction::Position => {
                let predicate_ctx = context.predicate_ctx.ok_or_else(|| Error::Internal {
                    msg: "[CoreFunction] position() is only usable as a predicate".to_string(),
                })?;
                Ok(Value::Number(predicate_ctx.index as f64))
            },
            CoreFunction::Count(expr) => {
                let nodes = expr.evaluate(context).and_then(try_extract_nodeset)?;
                Ok(Value::Number(nodes.len() as f64))
            },
            CoreFunction::String(expr_opt) => match expr_opt {
                Some(expr) => Ok(Value::String(expr.evaluate(context)?.string())),
                None => Ok(Value::String(context.context_node.string_value())),
            },
            CoreFunction::Concat(exprs) => {
                let strings: Result<Vec<_>, _> = exprs
                    .iter()
                    .map(|e| Ok(e.evaluate(context)?.string()))
                    .collect();
                Ok(Value::String(strings?.join("")))
            },
            CoreFunction::Id(_expr) => todo!(),
            CoreFunction::LocalName(expr_opt) => {
                let node = match expr_opt {
                    Some(expr) => expr
                        .evaluate(context)
                        .and_then(try_extract_nodeset)?
                        .first()
                        .cloned(),
                    None => Some(context.context_node.clone()),
                };
                let name = node.and_then(|n| n.local_name()).unwrap_or_default();
                Ok(Value::String(name.to_string()))
            },
            CoreFunction::NamespaceUri(expr_opt) => {
                let node = match expr_opt {
                    Some(expr) => expr
                        .evaluate(context)
                        .and_then(try_extract_nodeset)?
                        .first()
                        .cloned(),
                    None => Some(context.context_node.clone()),
                };
                let ns = node.and_then(|n| n.namespace_uri()).unwrap_or_default();
                Ok(Value::String(ns.to_string()))
            },
            CoreFunction::Name(expr_opt) => {
                let node = match expr_opt {
                    Some(expr) => expr
                        .evaluate(context)
                        .and_then(try_extract_nodeset)?
                        .first()
                        .cloned(),
                    None => Some(context.context_node.clone()),
                };
                let name = node.and_then(|n| n.name()).unwrap_or_default();
                Ok(Value::String(name))
            },
            CoreFunction::StartsWith(str1, str2) => {
                let s1 = str1.evaluate(context)?.string();
                let s2 = str2.evaluate(context)?.string();
                Ok(Value::Boolean(s1.starts_with(&s2)))
            },
            CoreFunction::Contains(str1, str2) => {
                let s1 = str1.evaluate(context)?.string();
                let s2 = str2.evaluate(context)?.string();
                Ok(Value::Boolean(s1.contains(&s2)))
            },
            CoreFunction::SubstringBefore(str1, str2) => {
                let s1 = str1.evaluate(context)?.string();
                let s2 = str2.evaluate(context)?.string();
                let result = match s1.find(&s2) {
                    Some(pos) => s1[..pos].to_string(),
                    None => String::new(),
                };
                Ok(Value::String(result))
            },
            CoreFunction::SubstringAfter(str1, str2) => {
                let s1 = str1.evaluate(context)?.string();
                let s2 = str2.evaluate(context)?.string();
                let result = match s1.find(&s2) {
                    Some(pos) => s1[pos + s2.len()..].to_string(),
                    None => String::new(),
                };
                Ok(Value::String(result))
            },
            CoreFunction::Substring(str1, start, length_opt) => {
                let s = str1.evaluate(context)?.string();
                let start_idx = start.evaluate(context)?.number().round() as isize - 1;
                let len = match length_opt {
                    Some(len) => len.evaluate(context)?.number().round() as isize,
                    None => s.len() as isize,
                };
                let start_idx = start_idx.max(0) as usize;
                let end_idx = (start_idx + len.max(0) as usize).min(s.len());
                Ok(Value::String(s[start_idx..end_idx].to_string()))
            },
            CoreFunction::StringLength(expr_opt) => {
                let s = match expr_opt {
                    Some(expr) => expr.evaluate(context)?.string(),
                    None => context.context_node.string_value(),
                };
                Ok(Value::Number(s.chars().count() as f64))
            },
            CoreFunction::NormalizeSpace(expr_opt) => {
                let s = match expr_opt {
                    Some(expr) => expr.evaluate(context)?.string(),
                    None => context.context_node.string_value(),
                };

                Ok(Value::String(normalize_space(&s)))
            },
            CoreFunction::Translate(str1, str2, str3) => {
                let s = str1.evaluate(context)?.string();
                let from = str2.evaluate(context)?.string();
                let to = str3.evaluate(context)?.string();
                let result = s
                    .chars()
                    .map(|c| match from.find(c) {
                        Some(i) if i < to.chars().count() => to.chars().nth(i).unwrap(),
                        _ => c,
                    })
                    .collect();
                Ok(Value::String(result))
            },
            CoreFunction::Number(expr_opt) => {
                let val = match expr_opt {
                    Some(expr) => expr.evaluate(context)?,
                    None => Value::String(context.context_node.string_value()),
                };
                Ok(Value::Number(val.number()))
            },
            CoreFunction::Sum(expr) => {
                let nodes = expr.evaluate(context).and_then(try_extract_nodeset)?;
                let sum = nodes
                    .iter()
                    .map(|n| Value::String(n.string_value()).number())
                    .sum();
                Ok(Value::Number(sum))
            },
            CoreFunction::Floor(expr) => {
                let num = expr.evaluate(context)?.number();
                Ok(Value::Number(num.floor()))
            },
            CoreFunction::Ceiling(expr) => {
                let num = expr.evaluate(context)?.number();
                Ok(Value::Number(num.ceil()))
            },
            CoreFunction::Round(expr) => {
                let num = expr.evaluate(context)?.number();
                Ok(Value::Number(num.round()))
            },
            CoreFunction::Boolean(expr) => Ok(Value::Boolean(expr.evaluate(context)?.boolean())),
            CoreFunction::Not(expr) => Ok(Value::Boolean(!expr.evaluate(context)?.boolean())),
            CoreFunction::True => Ok(Value::Boolean(true)),
            CoreFunction::False => Ok(Value::Boolean(false)),
            CoreFunction::Lang(_) => Ok(Value::Nodeset(vec![])), // Not commonly used in the DOM, short-circuit it
        }
    }

    fn is_primitive(&self) -> bool {
        match self {
            CoreFunction::Last => false,
            CoreFunction::Position => false,
            CoreFunction::Count(_) => false,
            CoreFunction::Id(_) => false,
            CoreFunction::LocalName(_) => false,
            CoreFunction::NamespaceUri(_) => false,
            CoreFunction::Name(_) => false,
            CoreFunction::String(expr_opt) => expr_opt
                .as_ref()
                .map(|expr| expr.is_primitive())
                .unwrap_or(false),
            CoreFunction::Concat(vec) => vec.iter().all(|expr| expr.is_primitive()),
            CoreFunction::StartsWith(expr, substr) => expr.is_primitive() && substr.is_primitive(),
            CoreFunction::Contains(expr, substr) => expr.is_primitive() && substr.is_primitive(),
            CoreFunction::SubstringBefore(expr, substr) => {
                expr.is_primitive() && substr.is_primitive()
            },
            CoreFunction::SubstringAfter(expr, substr) => {
                expr.is_primitive() && substr.is_primitive()
            },
            CoreFunction::Substring(expr, start_pos, length_opt) => {
                expr.is_primitive() &&
                    start_pos.is_primitive() &&
                    length_opt
                        .as_ref()
                        .map(|length| length.is_primitive())
                        .unwrap_or(false)
            },
            CoreFunction::StringLength(expr_opt) => expr_opt
                .as_ref()
                .map(|expr| expr.is_primitive())
                .unwrap_or(false),
            CoreFunction::NormalizeSpace(expr_opt) => expr_opt
                .as_ref()
                .map(|expr| expr.is_primitive())
                .unwrap_or(false),
            CoreFunction::Translate(expr, from_chars, to_chars) => {
                expr.is_primitive() && from_chars.is_primitive() && to_chars.is_primitive()
            },
            CoreFunction::Number(expr_opt) => expr_opt
                .as_ref()
                .map(|expr| expr.is_primitive())
                .unwrap_or(false),
            CoreFunction::Sum(expr) => expr.is_primitive(),
            CoreFunction::Floor(expr) => expr.is_primitive(),
            CoreFunction::Ceiling(expr) => expr.is_primitive(),
            CoreFunction::Round(expr) => expr.is_primitive(),
            CoreFunction::Boolean(expr) => expr.is_primitive(),
            CoreFunction::Not(expr) => expr.is_primitive(),
            CoreFunction::True => true,
            CoreFunction::False => true,
            CoreFunction::Lang(_) => false,
        }
    }
}
