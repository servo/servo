/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod ast;
mod context;
mod eval;
mod eval_function;
mod parser;
mod value;

use std::fmt::Debug;
use std::hash::Hash;

pub use ast::Expression;
use ast::QName;
use context::EvaluationCtx;
use markup5ever::{LocalName, Namespace, Prefix};
use parser::{OwnedParserError, parse as parse_impl};
pub use value::{NodesetHelpers, Value};

pub trait Dom {
    type Node: Node;
    /// An exception that can occur during JS evaluation.
    type JsError: Debug;
    type NamespaceResolver: NamespaceResolver<Self::JsError>;
}

/// A handle to a DOM node exposing all functionality needed by xpath.
pub trait Node: Eq + Clone + Debug {
    type ProcessingInstruction: ProcessingInstruction;
    type Document: Document<Node = Self>;
    type Attribute: Attribute<Node = Self>;
    type Element: Element<Node = Self>;

    fn is_comment(&self) -> bool;
    fn is_text(&self) -> bool;
    /// Equivalent to [`textContent`](https://dom.spec.whatwg.org/#dom-node-textcontent) attribute.
    fn text_content(&self) -> String;
    /// <https://html.spec.whatwg.org/multipage/#language>
    fn language(&self) -> Option<String>;
    fn parent(&self) -> Option<Self>;
    fn children(&self) -> impl Iterator<Item = Self>;
    /// <https://dom.spec.whatwg.org/#concept-tree-order>
    fn compare_tree_order(&self, other: &Self) -> std::cmp::Ordering;
    /// A non-shadow-including preorder traversal.
    fn traverse_preorder(&self) -> impl Iterator<Item = Self>;
    fn inclusive_ancestors(&self) -> impl Iterator<Item = Self>;
    fn preceding_nodes(&self, root: &Self) -> impl Iterator<Item = Self>;
    fn following_nodes(&self, root: &Self) -> impl Iterator<Item = Self>;
    fn preceding_siblings(&self) -> impl Iterator<Item = Self>;
    fn following_siblings(&self) -> impl Iterator<Item = Self>;
    fn owner_document(&self) -> Self::Document;
    fn to_opaque(&self) -> impl Eq + Hash;
    fn as_processing_instruction(&self) -> Option<Self::ProcessingInstruction>;
    fn as_attribute(&self) -> Option<Self::Attribute>;
    fn as_element(&self) -> Option<Self::Element>;
    fn lookup_namespace_uri(&self, uri: Option<&str>) -> Option<String>;
}

pub trait NamespaceResolver<E>: Clone {
    fn resolve_namespace_prefix(&self, prefix: Option<&str>) -> Result<Option<String>, E>;
}

pub trait ProcessingInstruction {
    fn target(&self) -> String;
}

pub trait Document {
    type Node: Node<Document = Self>;

    fn is_html_document(&self) -> bool;
    fn get_elements_with_id(&self, id: &str)
    -> impl Iterator<Item = <Self::Node as Node>::Element>;
}

pub trait Element {
    type Node: Node<Element = Self>;
    type Attribute: Attribute<Node = Self::Node>;

    fn as_node(&self) -> Self::Node;
    fn prefix(&self) -> Option<Prefix>;
    fn namespace(&self) -> Namespace;
    fn local_name(&self) -> LocalName;
    fn attributes(&self) -> impl Iterator<Item = Self::Attribute>;
}

pub trait Attribute {
    type Node: Node<Attribute = Self>;

    fn as_node(&self) -> Self::Node;
    fn prefix(&self) -> Option<Prefix>;
    fn namespace(&self) -> Namespace;
    fn local_name(&self) -> LocalName;
}

/// Parse an XPath expression from a string
pub fn parse<E>(xpath: &str) -> Result<Expression, Error<E>> {
    match parse_impl(xpath) {
        Ok(expression) => {
            log::debug!("Parsed XPath: {expression:?}");
            Ok(expression)
        },
        Err(error) => {
            log::debug!("Unable to parse XPath: {error}");
            Err(Error::Parsing(error))
        },
    }
}

/// Evaluate an already-parsed XPath expression
pub fn evaluate_parsed_xpath<D: Dom>(
    expr: &Expression,
    context_node: D::Node,
    resolver: Option<D::NamespaceResolver>,
) -> Result<Value<D::Node>, Error<D::JsError>> {
    let context = EvaluationCtx::<D>::new(context_node, resolver);
    match expr.evaluate(&context) {
        Ok(value) => {
            log::debug!("Evaluated XPath: {value:?}");
            Ok(value)
        },
        Err(error) => {
            log::debug!("Unable to evaluate XPath: {error:?}");
            Err(error)
        },
    }
}

#[derive(Clone, Debug)]
pub enum Error<JsError> {
    NotANodeset,
    /// It is not clear where variables used in XPath expression should come from.
    /// Firefox throws "NS_ERROR_ILLEGAL_VALUE" when using them, chrome seems to return
    /// an empty result. We also error out.
    ///
    /// See <https://github.com/whatwg/dom/issues/67>
    CannotUseVariables,
    InvalidQName {
        qname: QName,
    },
    Internal {
        msg: String,
    },
    /// A JS exception that needs to be propagated to the caller.
    JsException(JsError),
    Parsing(OwnedParserError),
}

/// <https://www.w3.org/TR/xml/#NT-NameStartChar>
fn is_valid_start(c: char) -> bool {
    matches!(c, ':' |
        'A'..='Z' |
        '_' |
        'a'..='z' |
        '\u{C0}'..='\u{D6}' |
        '\u{D8}'..='\u{F6}' |
        '\u{F8}'..='\u{2FF}' |
        '\u{370}'..='\u{37D}' |
        '\u{37F}'..='\u{1FFF}' |
        '\u{200C}'..='\u{200D}' |
        '\u{2070}'..='\u{218F}' |
        '\u{2C00}'..='\u{2FEF}' |
        '\u{3001}'..='\u{D7FF}' |
        '\u{F900}'..='\u{FDCF}' |
        '\u{FDF0}'..='\u{FFFD}' |
        '\u{10000}'..='\u{EFFFF}')
}

/// <https://www.w3.org/TR/xml/#NT-NameChar>
fn is_valid_continuation(c: char) -> bool {
    is_valid_start(c) ||
        matches!(c,
            '-' |
            '.' |
            '0'..='9' |
            '\u{B7}' |
            '\u{300}'..='\u{36F}' |
            '\u{203F}'..='\u{2040}')
}

#[cfg(test)]
/// Provides a dummy DOM to be used for tests.
mod dummy_implementation {
    use std::{cmp, iter};

    use markup5ever::{LocalName, ns};

    use super::*;

    // FIXME: Expand this as more features are required
    #[derive(Clone, Eq, Debug, PartialEq)]
    pub(crate) struct DummyNode;
    pub(crate) struct DummyProcessingInstruction;
    pub(crate) struct DummyDocument;
    pub(crate) struct DummyAttribute;
    pub(crate) struct DummyElement;

    impl Node for DummyNode {
        type ProcessingInstruction = DummyProcessingInstruction;
        type Document = DummyDocument;
        type Attribute = DummyAttribute;
        type Element = DummyElement;

        fn is_comment(&self) -> bool {
            false
        }
        fn is_text(&self) -> bool {
            false
        }
        fn text_content(&self) -> String {
            String::new()
        }
        fn language(&self) -> Option<String> {
            None
        }
        fn parent(&self) -> Option<Self> {
            None
        }
        fn children(&self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn compare_tree_order(&self, _: &Self) -> cmp::Ordering {
            cmp::Ordering::Greater
        }
        fn traverse_preorder(&self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn inclusive_ancestors(&self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn preceding_nodes(&self, _: &Self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn following_nodes(&self, _: &Self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn preceding_siblings(&self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn following_siblings(&self) -> impl Iterator<Item = Self> {
            iter::empty()
        }
        fn owner_document(&self) -> Self::Document {
            DummyDocument
        }
        fn to_opaque(&self) -> impl Eq + Hash {
            0
        }
        fn as_processing_instruction(&self) -> Option<Self::ProcessingInstruction> {
            None
        }
        fn as_attribute(&self) -> Option<Self::Attribute> {
            None
        }
        fn as_element(&self) -> Option<Self::Element> {
            None
        }
        fn lookup_namespace_uri(&self, _: Option<&str>) -> Option<String> {
            None
        }
    }

    impl ProcessingInstruction for DummyProcessingInstruction {
        fn target(&self) -> String {
            String::new()
        }
    }

    impl Document for DummyDocument {
        type Node = DummyNode;

        fn is_html_document(&self) -> bool {
            true
        }
        fn get_elements_with_id(
            &self,
            _: &str,
        ) -> impl Iterator<Item = <Self::Node as Node>::Element> {
            iter::empty()
        }
    }

    impl Element for DummyElement {
        type Node = DummyNode;
        type Attribute = DummyAttribute;

        fn as_node(&self) -> Self::Node {
            DummyNode
        }
        fn prefix(&self) -> Option<Prefix> {
            None
        }
        fn namespace(&self) -> Namespace {
            ns!()
        }
        fn local_name(&self) -> LocalName {
            LocalName::from("")
        }
        fn attributes(&self) -> impl Iterator<Item = Self::Attribute> {
            iter::empty()
        }
    }

    impl Attribute for DummyAttribute {
        type Node = DummyNode;

        fn as_node(&self) -> Self::Node {
            DummyNode
        }
        fn prefix(&self) -> Option<Prefix> {
            None
        }
        fn namespace(&self) -> Namespace {
            ns!()
        }
        fn local_name(&self) -> LocalName {
            LocalName::from("")
        }
    }
}
