/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::HashSet;
use std::{fmt, string};

use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::node::Node;

/// The primary types of values that an XPath expression returns as a result.
pub(crate) enum Value {
    Boolean(bool),
    /// A IEEE-754 double-precision floating point number
    Number(f64),
    String(String),
    /// A collection of not-necessarily-unique nodes
    Nodeset(Vec<DomRoot<Node>>),
}

impl fmt::Debug for Value {
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
fn str_vals(nodes: &[DomRoot<Node>]) -> HashSet<String> {
    nodes
        .iter()
        .map(|n| n.GetTextContent().unwrap_or_default().to_string())
        .collect()
}

/// Helper for `PartialEq<Value>` implementations
fn num_vals(nodes: &[DomRoot<Node>]) -> Vec<f64> {
    nodes
        .iter()
        .map(|n| Value::String(n.GetTextContent().unwrap_or_default().into()).number())
        .collect()
}

impl PartialEq<Value> for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nodeset(left_nodes), Value::Nodeset(right_nodes)) => {
                let left_strings = str_vals(left_nodes);
                let right_strings = str_vals(right_nodes);
                !left_strings.is_disjoint(&right_strings)
            },
            (&Value::Nodeset(ref nodes), &Value::Number(val)) |
            (&Value::Number(val), &Value::Nodeset(ref nodes)) => {
                let numbers = num_vals(nodes);
                numbers.iter().any(|n| *n == val)
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

impl Value {
    pub(crate) fn boolean(&self) -> bool {
        match *self {
            Value::Boolean(val) => val,
            Value::Number(n) => n != 0.0 && !n.is_nan(),
            Value::String(ref s) => !s.is_empty(),
            Value::Nodeset(ref nodeset) => !nodeset.is_empty(),
        }
    }

    pub(crate) fn into_boolean(self) -> bool {
        self.boolean()
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

    pub(crate) fn into_number(self) -> f64 {
        self.number()
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
                Some(n) => n.GetTextContent().unwrap_or_default().to_string(),
                None => "".to_owned(),
            },
        }
    }

    pub(crate) fn into_string(self) -> string::String {
        match self {
            Value::String(val) => val,
            other => other.string(),
        }
    }
}

macro_rules! from_impl {
    ($raw:ty, $variant:expr) => {
        impl From<$raw> for Value {
            fn from(other: $raw) -> Value {
                $variant(other)
            }
        }
    };
}

from_impl!(bool, Value::Boolean);
from_impl!(f64, Value::Number);
from_impl!(String, Value::String);
impl<'a> From<&'a str> for Value {
    fn from(other: &'a str) -> Value {
        Value::String(other.into())
    }
}
from_impl!(Vec<DomRoot<Node>>, Value::Nodeset);

macro_rules! partial_eq_impl {
    ($raw:ty, $variant:pat => $b:expr) => {
        impl PartialEq<$raw> for Value {
            fn eq(&self, other: &$raw) -> bool {
                match *self {
                    $variant => $b == other,
                    _ => false,
                }
            }
        }

        impl PartialEq<Value> for $raw {
            fn eq(&self, other: &Value) -> bool {
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
partial_eq_impl!(Vec<DomRoot<Node>>, Value::Nodeset(ref v) => v);

pub(crate) trait NodesetHelpers {
    /// Returns the node that occurs first in [document order]
    ///
    /// [document order]: https://www.w3.org/TR/xpath/#dt-document-order
    fn document_order_first(&self) -> Option<DomRoot<Node>>;
    fn document_order(&self) -> Vec<DomRoot<Node>>;
    fn document_order_unique(&self) -> Vec<DomRoot<Node>>;
}

impl NodesetHelpers for Vec<DomRoot<Node>> {
    fn document_order_first(&self) -> Option<DomRoot<Node>> {
        self.iter()
            .min_by(|a, b| {
                if a == b {
                    std::cmp::Ordering::Equal
                } else if a.is_before(b) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
            .cloned()
    }
    fn document_order(&self) -> Vec<DomRoot<Node>> {
        let mut nodes: Vec<DomRoot<Node>> = self.clone();
        if nodes.len() <= 1 {
            return nodes;
        }

        nodes.sort_by(|a, b| {
            if a == b {
                std::cmp::Ordering::Equal
            } else if a.is_before(b) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        nodes
    }
    fn document_order_unique(&self) -> Vec<DomRoot<Node>> {
        let mut seen = HashSet::new();
        let unique_nodes: Vec<DomRoot<Node>> = self
            .iter()
            .filter(|node| seen.insert(node.to_opaque()))
            .cloned()
            .collect();

        unique_nodes.document_order()
    }
}
