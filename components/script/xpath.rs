/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bindings to the `xpath` crate

use std::cell::Ref;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use html5ever::{LocalName, Namespace, Prefix};
use script_bindings::callback::ExceptionHandling;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::root::Dom;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use style::Atom;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::XPathNSResolverBinding::XPathNSResolver;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::comment::Comment;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
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
    type JsError = Error;
    type NamespaceResolver = XPathWrapper<Rc<XPathNSResolver>>;
}

impl xpath::Node for XPathWrapper<DomRoot<Node>> {
    type ProcessingInstruction = XPathWrapper<DomRoot<ProcessingInstruction>>;
    type Document = XPathWrapper<DomRoot<Document>>;
    type Attribute = XPathWrapper<DomRoot<Attr>>;
    type Element = XPathWrapper<DomRoot<Element>>;

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

    fn preceding_nodes(&self, root: &Self) -> impl Iterator<Item = Self> {
        self.0.preceding_nodes(&root.0).map(XPathWrapper)
    }

    fn following_nodes(&self, root: &Self) -> impl Iterator<Item = Self> {
        self.0.following_nodes(&root.0).map(XPathWrapper)
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

    fn to_opaque(&self) -> impl Eq + Hash {
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

    fn lookup_namespace_uri(&self, uri: Option<&str>) -> Option<String> {
        self.0
            .LookupNamespaceURI(uri.map(DOMString::from))
            .map(String::from)
    }
}

impl xpath::Document for XPathWrapper<DomRoot<Document>> {
    type Node = XPathWrapper<DomRoot<Node>>;

    fn is_html_document(&self) -> bool {
        self.0.is_html_document()
    }

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

impl xpath::NamespaceResolver<Error> for XPathWrapper<Rc<XPathNSResolver>> {
    fn resolve_namespace_prefix(&self, prefix: Option<&str>) -> Result<Option<String>, Error> {
        self.0
            .LookupNamespaceURI__(
                prefix.map(DOMString::from),
                ExceptionHandling::Rethrow,
                CanGc::note(),
            )
            .map(|result| result.map(String::from))
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
