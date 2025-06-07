/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::Atom;

use super::Value;
use super::context::EvaluationCtx;
use super::eval::{Error, Evaluatable, try_extract_nodeset};
use super::parser::CoreFunction;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::{Castable, NodeTypeId};
use crate::dom::bindings::root::DomRoot;
use crate::dom::element::Element;
use crate::dom::node::Node;

/// Returns e.g. "rect" for `<svg:rect>`
fn local_name(node: &Node) -> Option<String> {
    if matches!(Node::type_id(node), NodeTypeId::Element(_)) {
        let element = node.downcast::<Element>().unwrap();
        Some(element.local_name().to_string())
    } else {
        None
    }
}

/// Returns e.g. "svg:rect" for `<svg:rect>`
fn name(node: &Node) -> Option<String> {
    if matches!(Node::type_id(node), NodeTypeId::Element(_)) {
        let element = node.downcast::<Element>().unwrap();
        if let Some(prefix) = element.prefix().as_ref() {
            Some(format!("{}:{}", prefix, element.local_name()))
        } else {
            Some(element.local_name().to_string())
        }
    } else {
        None
    }
}

/// Returns e.g. the SVG namespace URI for `<svg:rect>`
fn namespace_uri(node: &Node) -> Option<String> {
    if matches!(Node::type_id(node), NodeTypeId::Element(_)) {
        let element = node.downcast::<Element>().unwrap();
        Some(element.namespace().to_string())
    } else {
        None
    }
}

/// Returns the text contents of the Node, or empty string if none.
fn string_value(node: &Node) -> String {
    node.GetTextContent().unwrap_or_default().to_string()
}

/// If s2 is found inside s1, return everything *before* s2. Return all of s1 otherwise.
fn substring_before(s1: &str, s2: &str) -> String {
    match s1.find(s2) {
        Some(pos) => s1[..pos].to_string(),
        None => String::new(),
    }
}

/// If s2 is found inside s1, return everything *after* s2. Return all of s1 otherwise.
fn substring_after(s1: &str, s2: &str) -> String {
    match s1.find(s2) {
        Some(pos) => s1[pos + s2.len()..].to_string(),
        None => String::new(),
    }
}

fn substring(s: &str, start_idx: isize, len: Option<isize>) -> String {
    let s_len = s.len();
    let len = len.unwrap_or(s_len as isize).max(0) as usize;
    let start_idx = start_idx.max(0) as usize;
    let end_idx = (start_idx + len.max(0)).min(s_len);
    s[start_idx..end_idx].to_string()
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-normalize-space>
pub(crate) fn normalize_space(s: &str) -> String {
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

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-lang>
fn lang_matches(context_lang: Option<&str>, target_lang: &str) -> bool {
    let Some(context_lang) = context_lang else {
        return false;
    };

    let context_lower = context_lang.to_ascii_lowercase();
    let target_lower = target_lang.to_ascii_lowercase();

    if context_lower == target_lower {
        return true;
    }

    // Check if context is target with additional suffix
    if context_lower.starts_with(&target_lower) {
        // Make sure the next character is a hyphen to avoid matching
        // e.g. "england" when target is "en"
        if let Some(next_char) = context_lower.chars().nth(target_lower.len()) {
            return next_char == '-';
        }
    }

    false
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
                None => Ok(Value::String(string_value(&context.context_node))),
            },
            CoreFunction::Concat(exprs) => {
                let strings: Result<Vec<_>, _> = exprs
                    .iter()
                    .map(|e| Ok(e.evaluate(context)?.string()))
                    .collect();
                Ok(Value::String(strings?.join("")))
            },
            CoreFunction::Id(expr) => {
                let args_str = expr.evaluate(context)?.string();
                let args_normalized = normalize_space(&args_str);
                let args = args_normalized.split(' ');

                let document = context.context_node.owner_doc();
                let mut result = Vec::new();
                for arg in args {
                    for element in document.get_elements_with_id(&Atom::from(arg)).iter() {
                        result.push(DomRoot::from_ref(element.upcast::<Node>()));
                    }
                }
                Ok(Value::Nodeset(result))
            },
            CoreFunction::LocalName(expr_opt) => {
                let node = match expr_opt {
                    Some(expr) => expr
                        .evaluate(context)
                        .and_then(try_extract_nodeset)?
                        .first()
                        .cloned(),
                    None => Some(context.context_node.clone()),
                };
                let name = node.and_then(|n| local_name(&n)).unwrap_or_default();
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
                let ns = node.and_then(|n| namespace_uri(&n)).unwrap_or_default();
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
                let name = node.and_then(|n| name(&n)).unwrap_or_default();
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
                Ok(Value::String(substring_before(&s1, &s2)))
            },
            CoreFunction::SubstringAfter(str1, str2) => {
                let s1 = str1.evaluate(context)?.string();
                let s2 = str2.evaluate(context)?.string();
                Ok(Value::String(substring_after(&s1, &s2)))
            },
            CoreFunction::Substring(str1, start, length_opt) => {
                let s = str1.evaluate(context)?.string();
                let start_idx = start.evaluate(context)?.number().round() as isize - 1;
                let len = match length_opt {
                    Some(len_expr) => Some(len_expr.evaluate(context)?.number().round() as isize),
                    None => None,
                };
                Ok(Value::String(substring(&s, start_idx, len)))
            },
            CoreFunction::StringLength(expr_opt) => {
                let s = match expr_opt {
                    Some(expr) => expr.evaluate(context)?.string(),
                    None => string_value(&context.context_node),
                };
                Ok(Value::Number(s.chars().count() as f64))
            },
            CoreFunction::NormalizeSpace(expr_opt) => {
                let s = match expr_opt {
                    Some(expr) => expr.evaluate(context)?.string(),
                    None => string_value(&context.context_node),
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
                    None => Value::String(string_value(&context.context_node)),
                };
                Ok(Value::Number(val.number()))
            },
            CoreFunction::Sum(expr) => {
                let nodes = expr.evaluate(context).and_then(try_extract_nodeset)?;
                let sum = nodes
                    .iter()
                    .map(|n| Value::String(string_value(n)).number())
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
            CoreFunction::Lang(expr) => {
                let context_lang = context.context_node.get_lang();
                let lang = expr.evaluate(context)?.string();
                Ok(Value::Boolean(lang_matches(context_lang.as_deref(), &lang)))
            },
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
#[cfg(test)]
mod tests {
    use super::{lang_matches, substring, substring_after, substring_before};

    #[test]
    fn test_substring_before() {
        assert_eq!(substring_before("hello world", "world"), "hello ");
        assert_eq!(substring_before("prefix:name", ":"), "prefix");
        assert_eq!(substring_before("no-separator", "xyz"), "");
        assert_eq!(substring_before("", "anything"), "");
        assert_eq!(substring_before("multiple:colons:here", ":"), "multiple");
        assert_eq!(substring_before("start-match-test", "start"), "");
    }

    #[test]
    fn test_substring_after() {
        assert_eq!(substring_after("hello world", "hello "), "world");
        assert_eq!(substring_after("prefix:name", ":"), "name");
        assert_eq!(substring_after("no-separator", "xyz"), "");
        assert_eq!(substring_after("", "anything"), "");
        assert_eq!(substring_after("multiple:colons:here", ":"), "colons:here");
        assert_eq!(substring_after("test-end-match", "match"), "");
    }

    #[test]
    fn test_substring() {
        assert_eq!(substring("hello world", 0, Some(5)), "hello");
        assert_eq!(substring("hello world", 6, Some(5)), "world");
        assert_eq!(substring("hello", 1, Some(3)), "ell");
        assert_eq!(substring("hello", -5, Some(2)), "he");
        assert_eq!(substring("hello", 0, None), "hello");
        assert_eq!(substring("hello", 2, Some(10)), "llo");
        assert_eq!(substring("hello", 5, Some(1)), "");
        assert_eq!(substring("", 0, Some(5)), "");
        assert_eq!(substring("hello", 0, Some(0)), "");
        assert_eq!(substring("hello", 0, Some(-5)), "");
    }

    #[test]
    fn test_lang_matches() {
        assert!(lang_matches(Some("en"), "en"));
        assert!(lang_matches(Some("EN"), "en"));
        assert!(lang_matches(Some("en"), "EN"));
        assert!(lang_matches(Some("en-US"), "en"));
        assert!(lang_matches(Some("en-GB"), "en"));

        assert!(!lang_matches(Some("eng"), "en"));
        assert!(!lang_matches(Some("fr"), "en"));
        assert!(!lang_matches(Some("fr-en"), "en"));
        assert!(!lang_matches(None, "en"));
    }
}
