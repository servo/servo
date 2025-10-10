/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashSet;

use crate::Node;

/// The primary types of values that an XPath expression returns as a result.
#[derive(Debug)]
pub enum Value<N: Node> {
    Boolean(bool),
    /// A IEEE-754 double-precision floating point number.
    Number(f64),
    String(String),
    /// A collection of not-necessarily-unique nodes.
    Nodeset(Vec<N>),
}

pub(crate) fn parse_number_from_string(string: &str) -> f64 {
    // https://www.w3.org/TR/1999/REC-xpath-19991116/#function-number:
    // > a string that consists of optional whitespace followed by an optional minus sign followed
    // > by a Number followed by whitespace is converted to the IEEE 754 number that is nearest
    // > (according to the IEEE 754 round-to-nearest rule) to the mathematical value represented
    // > by the string; any other string is converted to NaN

    // The specification does not define what "whitespace" means exactly, we choose to trim only ascii whitespace,
    // as that seems to be what other browsers do.
    string.trim_ascii().parse().unwrap_or(f64::NAN)
}

/// Helper for `PartialEq<Value>` implementations
fn str_vals<N: Node>(nodes: &[N]) -> HashSet<String> {
    nodes.iter().map(|n| n.text_content()).collect()
}

/// Helper for `PartialEq<Value>` implementations
fn num_vals<N: Node>(nodes: &[N]) -> Vec<f64> {
    nodes
        .iter()
        .map(|node| parse_number_from_string(&node.text_content()))
        .collect()
}

impl<N: Node> PartialEq<Value<N>> for Value<N> {
    fn eq(&self, other: &Value<N>) -> bool {
        match (self, other) {
            (Value::Nodeset(left_nodes), Value::Nodeset(right_nodes)) => {
                let left_strings = str_vals(left_nodes);
                let right_strings = str_vals(right_nodes);
                !left_strings.is_disjoint(&right_strings)
            },
            (&Value::Nodeset(ref nodes), &Value::Number(val)) |
            (&Value::Number(val), &Value::Nodeset(ref nodes)) => {
                let numbers = num_vals(nodes);
                numbers.contains(&val)
            },
            (&Value::Nodeset(ref nodes), &Value::String(ref val)) |
            (&Value::String(ref val), &Value::Nodeset(ref nodes)) => {
                let strings = str_vals(nodes);
                strings.contains(val)
            },
            (&Value::Boolean(_), _) | (_, &Value::Boolean(_)) => self.boolean() == other.boolean(),
            (&Value::Number(_), _) | (_, &Value::Number(_)) => self.number() == other.number(),
            _ => self.string() == other.string(),
        }
    }
}

impl<N: Node> Value<N> {
    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-boolean>
    pub(crate) fn boolean(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Number(number) => *number != 0.0 && !number.is_nan(),
            Value::String(string) => !string.is_empty(),
            Value::Nodeset(nodeset) => !nodeset.is_empty(),
        }
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-number>
    pub(crate) fn number(&self) -> f64 {
        match self {
            Value::Boolean(boolean) => {
                if *boolean {
                    1.0
                } else {
                    0.0
                }
            },
            Value::Number(number) => *number,
            Value::String(string) => parse_number_from_string(string),
            Value::Nodeset(_) => parse_number_from_string(&self.string()),
        }
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-string>
    pub(crate) fn string(&self) -> String {
        match self {
            Value::Boolean(value) => value.to_string(),
            Value::Number(number) => {
                if number.is_infinite() {
                    if number.is_sign_negative() {
                        "-Infinity".to_owned()
                    } else {
                        "Infinity".to_owned()
                    }
                } else if *number == 0.0 {
                    // catches -0.0 also
                    "0".into()
                } else {
                    number.to_string()
                }
            },
            Value::String(string) => string.to_owned(),
            Value::Nodeset(nodes) => nodes
                .document_order_first()
                .as_ref()
                .map(Node::text_content)
                .unwrap_or_default(),
        }
    }
}

macro_rules! from_impl {
    ($raw:ty, $variant:expr) => {
        impl<N: Node> From<$raw> for Value<N> {
            fn from(other: $raw) -> Self {
                $variant(other)
            }
        }
    };
}

from_impl!(bool, Value::Boolean);
from_impl!(f64, Value::Number);
from_impl!(String, Value::String);
impl<'a, N: Node> From<&'a str> for Value<N> {
    fn from(other: &'a str) -> Self {
        Value::String(other.into())
    }
}
from_impl!(Vec<N>, Value::Nodeset);

macro_rules! partial_eq_impl {
    ($raw:ty, $variant:pat => $b:expr) => {
        impl<N: Node> PartialEq<$raw> for Value<N> {
            fn eq(&self, other: &$raw) -> bool {
                match *self {
                    $variant => $b == other,
                    _ => false,
                }
            }
        }

        impl<N: Node> PartialEq<Value<N>> for $raw {
            fn eq(&self, other: &Value<N>) -> bool {
                match *other {
                    $variant => $b == self,
                    _ => false,
                }
            }
        }
    };
}

partial_eq_impl!(bool, Value::Boolean(ref v) => v);
partial_eq_impl!(f64, Value::Number(ref v) => v);
partial_eq_impl!(String, Value::String(ref v) => v);
partial_eq_impl!(&str, Value::String(ref v) => v);
partial_eq_impl!(Vec<N>, Value::Nodeset(ref v) => v);

pub trait NodesetHelpers<N: Node> {
    /// Returns the node that occurs first in [document order]
    ///
    /// [document order]: https://www.w3.org/TR/xpath/#dt-document-order
    fn document_order_first(&self) -> Option<N>;
    fn document_order(&self) -> Vec<N>;
    fn document_order_unique(&self) -> Vec<N>;
}

impl<N: Node> NodesetHelpers<N> for Vec<N> {
    fn document_order_first(&self) -> Option<N> {
        self.iter().min_by(|a, b| a.compare_tree_order(b)).cloned()
    }

    fn document_order(&self) -> Vec<N> {
        let mut nodes: Vec<N> = self.clone();
        if nodes.len() <= 1 {
            return nodes;
        }

        nodes.sort_by(|a, b| a.compare_tree_order(b));

        nodes
    }

    fn document_order_unique(&self) -> Vec<N> {
        let mut seen = HashSet::new();
        let unique_nodes: Vec<N> = self
            .iter()
            .filter(|node| seen.insert(node.to_opaque()))
            .cloned()
            .collect();

        unique_nodes.document_order()
    }
}

#[cfg(test)]
mod tests {
    use std::f64;

    use crate::dummy_implementation;

    type Value = super::Value<dummy_implementation::DummyNode>;

    #[test]
    fn string_value_to_number() {
        assert_eq!(Value::String("42.123".into()).number(), 42.123);
        assert_eq!(Value::String(" 42\n".into()).number(), 42.);
        assert!(Value::String("totally-invalid".into()).number().is_nan());

        // U+2004 is non-ascii whitespace, which should be rejected
        assert!(Value::String("\u{2004}42".into()).number().is_nan());
    }

    #[test]
    fn number_value_to_string() {
        assert_eq!(Value::Number(f64::NAN).string(), "NaN");
        assert_eq!(Value::Number(0.).string(), "0");
        assert_eq!(Value::Number(-0.).string(), "0");
        assert_eq!(Value::Number(f64::INFINITY).string(), "Infinity");
        assert_eq!(Value::Number(f64::NEG_INFINITY).string(), "-Infinity");
        assert_eq!(Value::Number(42.0).string(), "42");
        assert_eq!(Value::Number(-42.0).string(), "-42");
        assert_eq!(Value::Number(0.75).string(), "0.75");
        assert_eq!(Value::Number(-0.75).string(), "-0.75");
    }

    #[test]
    fn boolean_value_to_string() {
        assert_eq!(Value::Boolean(false).string(), "false");
        assert_eq!(Value::Boolean(true).string(), "true");
    }
}
