/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashSet;
use std::mem;

use crate::Node;

/// The primary types of values that an XPath expression returns as a result.
#[derive(Debug)]
pub enum Value<N: Node> {
    Boolean(bool),
    /// A IEEE-754 double-precision floating point number.
    Number(f64),
    String(String),
    NodeSet(NodeSet<N>),
}

#[derive(Debug)]
pub struct NodeSet<N: Node> {
    nodes: Vec<N>,
    is_sorted: bool,
}

impl<N: Node> Default for NodeSet<N> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            is_sorted: false,
        }
    }
}

impl<N: Node> NodeSet<N> {
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn push(&mut self, node: N) {
        self.is_sorted = false;
        self.nodes.push(node);
    }

    pub(crate) fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = N>,
    {
        self.nodes.extend(iter);
    }

    /// Whether this set is known to be sorted in tree order.
    ///
    /// This method is pessimistic and will never look at the elements in the set.
    /// As such, it *may* return `false` even if the set happens to be sorted.
    pub(crate) fn is_sorted(&self) -> bool {
        self.is_sorted || self.nodes.len() < 2
    }

    /// Assume that this set is sorted, without actually sorting it.
    pub(crate) fn assume_sorted(&mut self) {
        debug_assert!(
            self.nodes
                .is_sorted_by(|a, b| a.compare_tree_order(b).is_le())
        );
        self.is_sorted = true;
    }

    pub(crate) fn sort(&mut self) {
        if self.is_sorted() {
            return;
        }

        // Using sort_unstable_by here is fine because duplicates won't appear in the final
        // result anyways.
        self.nodes.sort_unstable_by(|a, b| a.compare_tree_order(b));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &N> {
        self.nodes.iter()
    }

    /// Return the first node in tree order that appears within this set.
    ///
    /// This method will not sort the set itself.
    pub(crate) fn first(&self) -> Option<N> {
        if self.is_sorted() {
            return self.nodes.first().cloned();
        }

        self.iter().min_by(|a, b| a.compare_tree_order(b)).cloned()
    }

    pub(crate) fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.nodes = mem::take(&mut self.nodes)
            .into_iter()
            .filter_map(|node| {
                let opaque = node.to_opaque();
                seen.insert(opaque).then_some(node)
            })
            .collect();
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    pub(crate) fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&N) -> bool,
    {
        self.nodes.retain(f)
    }

    pub(crate) fn reverse(&mut self) {
        self.nodes = mem::take(&mut self.nodes).into_iter().rev().collect();
    }
}

impl<N: Node> IntoIterator for NodeSet<N> {
    type IntoIter = <Vec<N> as IntoIterator>::IntoIter;
    type Item = <Vec<N> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<N: Node> FromIterator<N> for NodeSet<N> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Self {
            nodes: iter.into_iter().collect(),
            is_sorted: false,
        }
    }
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
fn num_vals<N: Node>(nodes: &NodeSet<N>) -> Vec<f64> {
    nodes
        .iter()
        .map(|node| parse_number_from_string(&node.text_content()))
        .collect()
}

impl<N: Node> PartialEq<Value<N>> for Value<N> {
    fn eq(&self, other: &Value<N>) -> bool {
        match (self, other) {
            (Value::NodeSet(left_nodes), Value::NodeSet(right_nodes)) => {
                let left_strings: HashSet<String> =
                    left_nodes.iter().map(|node| node.text_content()).collect();
                let right_strings: HashSet<String> =
                    right_nodes.iter().map(|node| node.text_content()).collect();
                !left_strings.is_disjoint(&right_strings)
            },
            (&Value::NodeSet(ref nodes), &Value::Number(val)) |
            (&Value::Number(val), &Value::NodeSet(ref nodes)) => {
                let numbers = num_vals(nodes);
                numbers.contains(&val)
            },
            (&Value::NodeSet(ref nodes), &Value::String(ref string)) |
            (&Value::String(ref string), &Value::NodeSet(ref nodes)) => nodes
                .iter()
                .map(|node| node.text_content())
                .any(|text_content| &text_content == string),
            (&Value::Boolean(_), _) | (_, &Value::Boolean(_)) => {
                self.convert_to_boolean() == other.convert_to_boolean()
            },
            (&Value::Number(_), _) | (_, &Value::Number(_)) => {
                self.convert_to_number() == other.convert_to_number()
            },
            _ => self.convert_to_string() == other.convert_to_string(),
        }
    }
}

impl<N: Node> Value<N> {
    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-boolean>
    pub fn convert_to_boolean(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            Value::Number(number) => *number != 0.0 && !number.is_nan(),
            Value::String(string) => !string.is_empty(),
            Value::NodeSet(nodeset) => !nodeset.is_empty(),
        }
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-number>
    pub fn convert_to_number(&self) -> f64 {
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
            Value::NodeSet(_) => parse_number_from_string(&self.convert_to_string()),
        }
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#function-string>
    pub fn convert_to_string(&self) -> String {
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
            Value::NodeSet(nodes) => nodes
                .first()
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

#[cfg(test)]
mod tests {
    use std::f64;

    use crate::dummy_implementation;

    type Value = super::Value<dummy_implementation::DummyNode>;

    #[test]
    fn string_value_to_number() {
        assert_eq!(Value::String("42.123".into()).convert_to_number(), 42.123);
        assert_eq!(Value::String(" 42\n".into()).convert_to_number(), 42.);
        assert!(
            Value::String("totally-invalid".into())
                .convert_to_number()
                .is_nan()
        );

        // U+2004 is non-ascii whitespace, which should be rejected
        assert!(
            Value::String("\u{2004}42".into())
                .convert_to_number()
                .is_nan()
        );
    }

    #[test]
    fn number_value_to_string() {
        assert_eq!(Value::Number(f64::NAN).convert_to_string(), "NaN");
        assert_eq!(Value::Number(0.).convert_to_string(), "0");
        assert_eq!(Value::Number(-0.).convert_to_string(), "0");
        assert_eq!(Value::Number(f64::INFINITY).convert_to_string(), "Infinity");
        assert_eq!(
            Value::Number(f64::NEG_INFINITY).convert_to_string(),
            "-Infinity"
        );
        assert_eq!(Value::Number(42.0).convert_to_string(), "42");
        assert_eq!(Value::Number(-42.0).convert_to_string(), "-42");
        assert_eq!(Value::Number(0.75).convert_to_string(), "0.75");
        assert_eq!(Value::Number(-0.75).convert_to_string(), "-0.75");
    }

    #[test]
    fn boolean_value_to_string() {
        assert_eq!(Value::Boolean(false).convert_to_string(), "false");
        assert_eq!(Value::Boolean(true).convert_to_string(), "true");
    }
}
