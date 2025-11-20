/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bindings to the `xpath` crate

use std::cell::Ref;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::rc::Rc;

use html5ever::{LocalName, Namespace, Prefix};
use script_bindings::callback::ExceptionHandling;
use script_bindings::codegen::GenericBindings::AttrBinding::AttrMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use script_bindings::root::Dom;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use style::Atom;
use style::dom::OpaqueNode;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::comment::Comment;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits, PrecedingNodeIterator, ShadowIncluding};
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::text::Text;

pub(crate) type Value = xpath::Value<XPathWrapper<DomRoot<Node>>>;

/// Wrapper type that allows us to define xpath traits on the relevant types,
/// since they're not defined in `script`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct XPathWrapper<T>(pub T);

pub(crate) struct XPathImplementation;

impl xpath::Dom for XPathImplementation {
    type Node = XPathWrapper<DomRoot<Node>>;
    type NamespaceResolver = XPathWrapper<Rc<XPathNSResolver>>;
}

impl xpath::Node for XPathWrapper<DomRoot<Node>> {
    type ProcessingInstruction = XPathWrapper<DomRoot<ProcessingInstruction>>;
    type Document = XPathWrapper<DomRoot<Document>>;
    type Attribute = XPathWrapper<DomRoot<Attr>>;
    type Element = XPathWrapper<DomRoot<Element>>;
    /// A opaque handle to a node with the sole purpose of comparing one node with another.
    type Opaque = OpaqueNode;

    fn is_comment(&self) -> bool {
        self.0.is::<Comment>()
    }

    fn is_text(&self) -> bool {
        self.0.is::<Text>()
    }

    fn text_content(&self) -> String {
        self.0.GetTextContent().unwrap_or_default().into()
    }

    fn language(&self) -> Option<String> {
        self.0.get_lang()
    }

    fn parent(&self) -> Option<Self> {
        // The parent of an attribute node is its owner, see
        // https://www.w3.org/TR/1999/REC-xpath-19991116/#attribute-nodes
        if let Some(attribute) = self.0.downcast::<Attr>() {
            return attribute
                .GetOwnerElement()
                .map(DomRoot::upcast)
                .map(XPathWrapper);
        }

        self.0.GetParentNode().map(XPathWrapper)
    }

    fn children(&self) -> impl Iterator<Item = Self> {
        self.0.children().map(XPathWrapper)
    }

    fn compare_tree_order(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if self.0.is_before(&other.0) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    fn traverse_preorder(&self) -> impl Iterator<Item = Self> {
        self.0
            .traverse_preorder(ShadowIncluding::No)
            .map(XPathWrapper)
    }

    fn inclusive_ancestors(&self) -> impl Iterator<Item = Self> {
        self.0
            .inclusive_ancestors(ShadowIncluding::No)
            .map(XPathWrapper)
    }

    fn preceding_nodes(&self) -> impl Iterator<Item = Self> {
        PrecedingNodeIteratorWithoutAncestors::new(&self.0).map(XPathWrapper)
    }

    fn following_nodes(&self) -> impl Iterator<Item = Self> {
        let owner_document = self.0.owner_document();
        let next_non_descendant_node = self
            .0
            .following_nodes(owner_document.upcast())
            .next_skipping_children();
        let following_nodes = next_non_descendant_node
            .clone()
            .map(|node| node.following_nodes(owner_document.upcast()))
            .into_iter()
            .flatten();
        next_non_descendant_node
            .into_iter()
            .chain(following_nodes)
            .map(XPathWrapper)
    }

    fn preceding_siblings(&self) -> impl Iterator<Item = Self> {
        self.0.preceding_siblings().map(XPathWrapper)
    }

    fn following_siblings(&self) -> impl Iterator<Item = Self> {
        self.0.following_siblings().map(XPathWrapper)
    }

    fn owner_document(&self) -> Self::Document {
        XPathWrapper(self.0.owner_document())
    }

    fn to_opaque(&self) -> Self::Opaque {
        self.0.to_opaque()
    }

    fn as_processing_instruction(&self) -> Option<Self::ProcessingInstruction> {
        self.0
            .downcast::<ProcessingInstruction>()
            .map(DomRoot::from_ref)
            .map(XPathWrapper)
    }

    fn as_attribute(&self) -> Option<Self::Attribute> {
        self.0
            .downcast::<Attr>()
            .map(DomRoot::from_ref)
            .map(XPathWrapper)
    }

    fn as_element(&self) -> Option<Self::Element> {
        self.0
            .downcast::<Element>()
            .map(DomRoot::from_ref)
            .map(XPathWrapper)
    }

    fn get_root_node(&self) -> Self {
        XPathWrapper(self.0.GetRootNode(&GetRootNodeOptions::empty()))
    }
}

impl xpath::Document for XPathWrapper<DomRoot<Document>> {
    type Node = XPathWrapper<DomRoot<Node>>;

    fn get_elements_with_id(
        &self,
        id: &str,
    ) -> impl Iterator<Item = XPathWrapper<DomRoot<Element>>> {
        struct ElementIterator<'a> {
            elements: Ref<'a, [Dom<Element>]>,
            position: usize,
        }

        impl<'a> Iterator for ElementIterator<'a> {
            type Item = XPathWrapper<DomRoot<Element>>;

            fn next(&mut self) -> Option<Self::Item> {
                let element = self.elements.get(self.position)?;
                self.position += 1;
                Some(element.as_rooted().into())
            }
        }

        ElementIterator {
            elements: self.0.get_elements_with_id(&Atom::from(id)),
            position: 0,
        }
    }
}

impl xpath::Element for XPathWrapper<DomRoot<Element>> {
    type Node = XPathWrapper<DomRoot<Node>>;
    type Attribute = XPathWrapper<DomRoot<Attr>>;

    fn as_node(&self) -> Self::Node {
        DomRoot::from_ref(self.0.upcast::<Node>()).into()
    }

    fn attributes(&self) -> impl Iterator<Item = Self::Attribute> {
        struct AttributeIterator<'a> {
            attributes: Ref<'a, [Dom<Attr>]>,
            position: usize,
        }

        impl<'a> Iterator for AttributeIterator<'a> {
            type Item = XPathWrapper<DomRoot<Attr>>;

            fn next(&mut self) -> Option<Self::Item> {
                let attribute = self.attributes.get(self.position)?;
                self.position += 1;
                Some(attribute.as_rooted().into())
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let exact_length = self.attributes.len() - self.position;
                (exact_length, Some(exact_length))
            }
        }

        AttributeIterator {
            attributes: self.0.attrs(),
            position: 0,
        }
    }

    fn prefix(&self) -> Option<Prefix> {
        self.0.prefix().clone()
    }

    fn namespace(&self) -> Namespace {
        self.0.namespace().clone()
    }

    fn local_name(&self) -> LocalName {
        self.0.local_name().clone()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.0.is_html_element() && self.0.owner_document().is_html_document()
    }
}

impl xpath::Attribute for XPathWrapper<DomRoot<Attr>> {
    type Node = XPathWrapper<DomRoot<Node>>;

    fn as_node(&self) -> Self::Node {
        XPathWrapper(DomRoot::from_ref(self.0.upcast::<Node>()))
    }

    fn prefix(&self) -> Option<Prefix> {
        self.0.prefix().cloned()
    }

    fn namespace(&self) -> Namespace {
        self.0.namespace().clone()
    }

    fn local_name(&self) -> LocalName {
        self.0.local_name().clone()
    }
}

impl xpath::NamespaceResolver for XPathWrapper<Rc<XPathNSResolver>> {
    fn resolve_namespace_prefix(&self, prefix: &str) -> Option<String> {
        self.0
            .LookupNamespaceURI__(
                Some(DOMString::from(prefix)),
                ExceptionHandling::Report,
                CanGc::note(),
            )
            .ok()
            .flatten()
            .map(String::from)
    }
}

impl xpath::ProcessingInstruction for XPathWrapper<DomRoot<ProcessingInstruction>> {
    fn target(&self) -> String {
        self.0.target().to_owned().into()
    }
}

impl<T> From<T> for XPathWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> XPathWrapper<T> {
    pub(crate) fn into_inner(self) -> T {
        self.0
    }
}

pub(crate) fn parse_expression(
    expression: &str,
    resolver: Option<Rc<XPathNSResolver>>,
    is_in_html_document: bool,
) -> Fallible<xpath::Expression> {
    xpath::parse(expression, resolver.map(XPathWrapper), is_in_html_document).map_err(|error| {
        match error {
            xpath::ParserError::FailedToResolveNamespacePrefix => Error::Namespace(None),
            _ => Error::Syntax(Some(format!("Failed to parse XPath expression: {error:?}"))),
        }
    })
}

enum PrecedingNodeIteratorWithoutAncestors {
    Done,
    NotDone {
        current: DomRoot<Node>,
        /// When we're currently walking over the subtree of a node in reverse tree order
        /// then this is the iterator for doing that.
        subtree_iterator: Option<PrecedingNodeIterator>,
    },
}

/// Returns the previous element (in tree order) that is not an ancestor of `node`.
fn previous_non_ancestor_node(node: &Node) -> Option<DomRoot<Node>> {
    let mut current = DomRoot::from_ref(node);
    loop {
        if let Some(previous_sibling) = current.GetPreviousSibling() {
            return Some(previous_sibling);
        }

        current = current.GetParentNode()?;
    }
}

impl PrecedingNodeIteratorWithoutAncestors {
    fn new(node: &Node) -> Self {
        let Some(current) = previous_non_ancestor_node(node) else {
            return Self::Done;
        };

        Self::NotDone {
            subtree_iterator: current
                .descending_last_children()
                .last()
                .map(|node| node.preceding_nodes(&current)),
            current,
        }
    }
}

impl Iterator for PrecedingNodeIteratorWithoutAncestors {
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self::NotDone {
            current,
            subtree_iterator,
        } = self
        else {
            return None;
        };

        if let Some(next_node) = subtree_iterator
            .as_mut()
            .and_then(|iterator| iterator.next())
        {
            return Some(next_node);
        }

        // Our current subtree is exhausted. Return the root of the subtree and move on to the next one
        // in inverse tree order.
        let Some(next_subtree) = previous_non_ancestor_node(current) else {
            *self = Self::Done;
            return None;
        };

        *current = next_subtree;
        *subtree_iterator = current
            .descending_last_children()
            .last()
            .map(|node| node.preceding_nodes(current));

        self.next()
    }
}
