/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::InheritTypes::ElementTypeId;
use dom::bindings::codegen::InheritTypes::HTMLElementTypeId;
use dom::bindings::codegen::InheritTypes::NodeTypeId;
use dom::bindings::conversions::Castable;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::event::Event;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlappletelement::HTMLAppletElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlbaseelement::HTMLBaseElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::HTMLFontElement;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node};
use string_cache::Atom;
use util::str::DOMString;


/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common
/// behaviours. Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    /// Returns self as the superclass of the implementation for this trait,
    /// if any.
    fn super_type(&self) -> Option<&VirtualMethods>;

    /// Called when attributes of a node are mutated.
    /// https://dom.spec.whatwg.org/#attribute-is-set
    /// https://dom.spec.whatwg.org/#attribute-is-removed
    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }
    }

    /// Returns the right AttrValue variant for the attribute with name `name`
    /// on this element.
    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match self.super_type() {
            Some(ref s) => s.parse_plain_attribute(name, value),
            _ => AttrValue::String(value),
        }
    }

    /// Called when a Node is appended to a tree, where 'tree_in_doc' indicates
    /// whether the tree is part of a Document.
    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }
    }

    /// Called when a Node is removed from a tree, where 'tree_in_doc'
    /// indicates whether the tree is part of a Document.
    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }
    }

    /// Called on the parent when its children are changed.
    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
    }

    /// Called during event dispatch after the bubbling phase completes.
    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }
    }

    /// https://dom.spec.whatwg.org/#concept-node-adopt-ext
    fn adopting_steps(&self, old_doc: &Document) {
        if let Some(ref s) = self.super_type() {
            s.adopting_steps(old_doc);
        }
    }

    /// https://dom.spec.whatwg.org/#concept-node-clone-ext
    fn cloning_steps(&self, copy: &Node, maybe_doc: Option<&Document>,
                     clone_children: CloneChildrenFlag) {
        if let Some(ref s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children);
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any
/// method call on the trait object will invoke the corresponding method on the
/// concrete type, propagating up the parent hierarchy unless otherwise
/// interrupted.
pub fn vtable_for(node: &Node) -> &VirtualMethods {
    match node.type_id() {
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
            let element = node.downcast::<HTMLAnchorElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAppletElement)) => {
            node.downcast::<HTMLAppletElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) => {
            let element = node.downcast::<HTMLAreaElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBaseElement)) => {
            let element = node.downcast::<HTMLBaseElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) => {
            let element = node.downcast::<HTMLBodyElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            let element = node.downcast::<HTMLButtonElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)) => {
            let element = node.downcast::<HTMLCanvasElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
            let element = node.downcast::<HTMLFieldSetElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)) => {
            let element = node.downcast::<HTMLFontElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement)) => {
            let element = node.downcast::<HTMLFormElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement)) => {
            let element = node.downcast::<HTMLHeadElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
            let element = node.downcast::<HTMLImageElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)) => {
            let element = node.downcast::<HTMLIFrameElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
            let element = node.downcast::<HTMLInputElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
            let element = node.downcast::<HTMLLinkElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMetaElement)) => {
            let element = node.downcast::<HTMLMetaElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            let element = node.downcast::<HTMLObjectElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)) => {
            let element = node.downcast::<HTMLOptGroupElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
            let element = node.downcast::<HTMLOptionElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)) => {
            let element = node.downcast::<HTMLScriptElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            let element = node.downcast::<HTMLSelectElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)) => {
            let element = node.downcast::<HTMLStyleElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)) => {
            let element =
                node.downcast::<HTMLTableElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_))) => {
            let element =
                node.downcast::<HTMLTableCellElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)) => {
            let element =
                node.downcast::<HTMLTableRowElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement)) => {
            let element =
                node.downcast::<HTMLTableSectionElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTemplateElement)) => {
            node.downcast::<HTMLTemplateElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            let element = node.downcast::<HTMLTextAreaElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement)) => {
            let element =
                node.downcast::<HTMLTitleElement>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::Element) => {
            let element = node.downcast::<Element>().unwrap();
            element as &VirtualMethods
        }
        NodeTypeId::Element(_) => {
            let element = node.downcast::<HTMLElement>().unwrap();
            element as &VirtualMethods
        }
        _ => {
            node as &VirtualMethods
        }
    }
}
