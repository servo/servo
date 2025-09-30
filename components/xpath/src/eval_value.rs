/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashSet;
use std::{fmt, string};

use crate::Node;

/// The primary types of values that an XPath expression returns as a result.
pub enum Value<N: Node> {
    Boolean(bool),
    /// A IEEE-754 double-precision floating point number
    Number(f64),
    String(String),
    /// A collection of not-necessarily-unique nodes
    Nodeset(Vec<N>),
}

impl<N: Node> fmt::Debug for Value<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Value::Boolean(val) => write!(f, "{}", val),
            Value::Number(val) => write!(f, "{}", val),
            Value::String(ref val) => write!(f, "{}", val),
            Value::Nodeset(ref val) => write!(f, "Nodeset({:?})", val),
        }
    }
}

pub(crate) fn str_to_num(s: &str) -> f64 {
    s.trim().parse().unwrap_or(f64::NAN)
}

/// Helper for `PartialEq<Value>` implementations
fn str_vals<N: Node>(nodes: &[N]) -> HashSet<String> {
    nodes.iter().map(|n| n.text_content()).collect()
}

/// Helper for `PartialEq<Value>` implementations
fn num_vals<N: Node>(nodes: &[N]) -> Vec<f64> {
    nodes
        .iter()
        .map(|node| str_to_num(&node.text_content()))
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
    pub(crate) fn boolean(&self) -> bool {
        match *self {
            Value::Boolean(val) => val,
            Value::Number(n) => n != 0.0 && !n.is_nan(),
            Value::String(ref s) => !s.is_empty(),
            Value::Nodeset(ref nodeset) => !nodeset.is_empty(),
        }
    }

    pub(crate) fn number(&self) -> f64 {
        match *self {
            Value::Boolean(val) => {
                if val {
                    1.0
                } else {
                    0.0
                }
            },
            Value::Number(val) => val,
            Value::String(ref s) => str_to_num(s),
            Value::Nodeset(..) => str_to_num(&self.string()),
        }
    }

    pub(crate) fn string(&self) -> string::String {
        match *self {
            Value::Boolean(v) => v.to_string(),
            Value::Number(n) => {
                if n.is_infinite() {
                    if n.signum() < 0.0 {
                        "-Infinity".to_owned()
                    } else {
                        "Infinity".to_owned()
                    }
                } else if n == 0.0 {
                    // catches -0.0 also
                    0.0.to_string()
                } else {
                    n.to_string()
                }
            },
            Value::String(ref val) => val.clone(),
            Value::Nodeset(ref nodes) => match nodes.document_order_first() {
                Some(n) => n.text_content(),
                None => "".to_owned(),
            },
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
