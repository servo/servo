/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::XPathResultBinding::{
    XPathResultConstants, XPathResultMethods,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
use crate::xpath::{NodesetHelpers, Value};

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, JSTraceable, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub(crate) enum XPathResultType {
    Any = XPathResultConstants::ANY_TYPE,
    Number = XPathResultConstants::NUMBER_TYPE,
    String = XPathResultConstants::STRING_TYPE,
    Boolean = XPathResultConstants::BOOLEAN_TYPE,
    UnorderedNodeIterator = XPathResultConstants::UNORDERED_NODE_ITERATOR_TYPE,
    OrderedNodeIterator = XPathResultConstants::ORDERED_NODE_ITERATOR_TYPE,
    UnorderedNodeSnapshot = XPathResultConstants::UNORDERED_NODE_SNAPSHOT_TYPE,
    OrderedNodeSnapshot = XPathResultConstants::ORDERED_NODE_SNAPSHOT_TYPE,
    AnyUnorderedNode = XPathResultConstants::ANY_UNORDERED_NODE_TYPE,
    FirstOrderedNode = XPathResultConstants::FIRST_ORDERED_NODE_TYPE,
}

impl TryFrom<u16> for XPathResultType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            XPathResultConstants::ANY_TYPE => Ok(Self::Any),
            XPathResultConstants::NUMBER_TYPE => Ok(Self::Number),
            XPathResultConstants::STRING_TYPE => Ok(Self::String),
            XPathResultConstants::BOOLEAN_TYPE => Ok(Self::Boolean),
            XPathResultConstants::UNORDERED_NODE_ITERATOR_TYPE => Ok(Self::UnorderedNodeIterator),
            XPathResultConstants::ORDERED_NODE_ITERATOR_TYPE => Ok(Self::OrderedNodeIterator),
            XPathResultConstants::UNORDERED_NODE_SNAPSHOT_TYPE => Ok(Self::UnorderedNodeSnapshot),
            XPathResultConstants::ORDERED_NODE_SNAPSHOT_TYPE => Ok(Self::OrderedNodeSnapshot),
            XPathResultConstants::ANY_UNORDERED_NODE_TYPE => Ok(Self::AnyUnorderedNode),
            XPathResultConstants::FIRST_ORDERED_NODE_TYPE => Ok(Self::FirstOrderedNode),
            _ => Err(()),
        }
    }
}

#[derive(Debug, JSTraceable, MallocSizeOf)]
pub(crate) enum XPathResultValue {
    Boolean(bool),
    /// A IEEE-754 double-precision floating point number
    Number(f64),
    String(DOMString),
    /// A collection of unique nodes
    Nodeset(Vec<DomRoot<Node>>),
}

impl From<Value> for XPathResultValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(b) => XPathResultValue::Boolean(b),
            Value::Number(n) => XPathResultValue::Number(n),
            Value::String(s) => XPathResultValue::String(s.into()),
            Value::Nodeset(nodes) => {
                // Put the evaluation result into (unique) document order. This also re-roots them
                // so that we are sure we can hold them for the lifetime of this XPathResult.
                let rooted_nodes = nodes.document_order_unique();
                XPathResultValue::Nodeset(rooted_nodes)
            },
        }
    }
}

#[dom_struct]
pub(crate) struct XPathResult {
    reflector_: Reflector,
    window: Dom<Window>,
    result_type: XPathResultType,
    value: XPathResultValue,
    iterator_invalid: Cell<bool>,
    iterator_pos: Cell<usize>,
}

impl XPathResult {
    fn new_inherited(
        window: &Window,
        result_type: XPathResultType,
        value: XPathResultValue,
    ) -> XPathResult {
        // TODO(vlindhol): if the wanted result type is AnyUnorderedNode | FirstOrderedNode,
        // we could drop all nodes except one to save memory.
        let inferred_result_type = if result_type == XPathResultType::Any {
            match value {
                XPathResultValue::Boolean(_) => XPathResultType::Boolean,
                XPathResultValue::Number(_) => XPathResultType::Number,
                XPathResultValue::String(_) => XPathResultType::String,
                XPathResultValue::Nodeset(_) => XPathResultType::UnorderedNodeIterator,
            }
        } else {
            result_type
        };

        XPathResult {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            result_type: inferred_result_type,
            iterator_invalid: Cell::new(false),
            iterator_pos: Cell::new(0),
            value,
        }
    }

    /// NB: Blindly trusts `result_type` and constructs an object regardless of the contents
    /// of `value`. The exception is `XPathResultType::Any`, for which we look at the value
    /// to determine the type.
    pub(crate) fn new(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        result_type: XPathResultType,
        value: XPathResultValue,
    ) -> DomRoot<XPathResult> {
        reflect_dom_object_with_proto(
            Box::new(XPathResult::new_inherited(window, result_type, value)),
            window,
            proto,
            can_gc,
        )
    }
}

impl XPathResultMethods<crate::DomTypeHolder> for XPathResult {
    /// <https://dom.spec.whatwg.org/#dom-xpathresult-resulttype>
    fn ResultType(&self) -> u16 {
        self.result_type as u16
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-numbervalue>
    fn GetNumberValue(&self) -> Fallible<f64> {
        match (&self.value, self.result_type) {
            (XPathResultValue::Number(n), XPathResultType::Number) => Ok(*n),
            _ => Err(Error::Type(
                "Can't get number value for non-number XPathResult".to_string(),
            )),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-stringvalue>
    fn GetStringValue(&self) -> Fallible<DOMString> {
        match (&self.value, self.result_type) {
            (XPathResultValue::String(s), XPathResultType::String) => Ok(s.clone()),
            _ => Err(Error::Type(
                "Can't get string value for non-string XPathResult".to_string(),
            )),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-booleanvalue>
    fn GetBooleanValue(&self) -> Fallible<bool> {
        match (&self.value, self.result_type) {
            (XPathResultValue::Boolean(b), XPathResultType::Boolean) => Ok(*b),
            _ => Err(Error::Type(
                "Can't get boolean value for non-boolean XPathResult".to_string(),
            )),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-iteratenext>
    fn IterateNext(&self) -> Fallible<Option<DomRoot<Node>>> {
        // TODO(vlindhol): actually set `iterator_invalid` somewhere
        if self.iterator_invalid.get() {
            return Err(Error::Range(
                "Invalidated iterator for XPathResult, the DOM has mutated.".to_string(),
            ));
        }

        match (&self.value, self.result_type) {
            (
                XPathResultValue::Nodeset(nodes),
                XPathResultType::OrderedNodeIterator | XPathResultType::UnorderedNodeIterator,
            ) => {
                let pos = self.iterator_pos.get();
                if pos >= nodes.len() {
                    Ok(None)
                } else {
                    let node = nodes[pos].clone();
                    self.iterator_pos.set(pos + 1);
                    Ok(Some(node))
                }
            },
            _ => Err(Error::Type(
                "Can't iterate on XPathResult that is not a node-set".to_string(),
            )),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-invaliditeratorstate>
    fn GetInvalidIteratorState(&self) -> Fallible<bool> {
        let is_iterator_invalid = self.iterator_invalid.get();
        if is_iterator_invalid ||
            matches!(
                self.result_type,
                XPathResultType::OrderedNodeIterator | XPathResultType::UnorderedNodeIterator
            )
        {
            Ok(is_iterator_invalid)
        } else {
            Err(Error::Type(
                "Can't iterate on XPathResult that is not a node-set".to_string(),
            ))
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-snapshotlength>
    fn GetSnapshotLength(&self) -> Fallible<u32> {
        match (&self.value, self.result_type) {
            (
                XPathResultValue::Nodeset(nodes),
                XPathResultType::OrderedNodeSnapshot | XPathResultType::UnorderedNodeSnapshot,
            ) => Ok(nodes.len() as u32),
            _ => Err(Error::Type(
                "Can't get snapshot length of XPathResult that is not a snapshot".to_string(),
            )),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-snapshotitem>
    fn SnapshotItem(&self, index: u32) -> Fallible<Option<DomRoot<Node>>> {
        match (&self.value, self.result_type) {
            (
                XPathResultValue::Nodeset(nodes),
                XPathResultType::OrderedNodeSnapshot | XPathResultType::UnorderedNodeSnapshot,
            ) => Ok(nodes.get(index as usize).cloned()),
            _ => Err(Error::Type(
                "Can't get snapshot item of XPathResult that is not a snapshot".to_string(),
            )),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-xpathresult-singlenodevalue>
    fn GetSingleNodeValue(&self) -> Fallible<Option<DomRoot<Node>>> {
        match (&self.value, self.result_type) {
            (
                XPathResultValue::Nodeset(nodes),
                XPathResultType::AnyUnorderedNode | XPathResultType::FirstOrderedNode,
            ) => Ok(nodes.first().cloned()),
            _ => Err(Error::Type(
                "Getting single value requires result type 'any unordered node' or 'first ordered node'".to_string(),
            )),
        }
    }
}
