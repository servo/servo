/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementCast;
use dom::bindings::codegen::InheritTypes::HTMLAreaElementCast;
use dom::bindings::codegen::InheritTypes::HTMLAppletElementCast;
use dom::bindings::codegen::InheritTypes::HTMLBaseElementCast;
use dom::bindings::codegen::InheritTypes::HTMLBodyElementCast;
use dom::bindings::codegen::InheritTypes::HTMLButtonElementCast;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFieldSetElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFontElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFormElementCast;
use dom::bindings::codegen::InheritTypes::HTMLHeadElementCast;
use dom::bindings::codegen::InheritTypes::HTMLIFrameElementCast;
use dom::bindings::codegen::InheritTypes::HTMLImageElementCast;
use dom::bindings::codegen::InheritTypes::HTMLInputElementCast;
use dom::bindings::codegen::InheritTypes::HTMLLinkElementCast;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::codegen::InheritTypes::HTMLOptGroupElementCast;
use dom::bindings::codegen::InheritTypes::HTMLOptionElementCast;
use dom::bindings::codegen::InheritTypes::HTMLScriptElementCast;
use dom::bindings::codegen::InheritTypes::HTMLSelectElementCast;
use dom::bindings::codegen::InheritTypes::HTMLStyleElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTableElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTableCellElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTableRowElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTableSectionElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTextAreaElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTitleElementCast;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::event::Event;
use dom::htmlelement::HTMLElementTypeId;
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node, NodeHelpers};
use dom::node::NodeTypeId;

use util::str::DOMString;

use string_cache::Atom;

/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common
/// behaviours. Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    /// Returns self as the superclass of the implementation for this trait,
    /// if any.
    fn super_type(&self) -> Option<&VirtualMethods>;

    /// Called when changing or adding attributes, after the attribute's value
    /// has been updated.
    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }
    }

    /// Called when changing or removing attributes, before any modification
    /// has taken place.
    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }
    }

    /// Called when changing or removing attributes, after all modification
    /// has taken place.
    fn after_remove_attr(&self, name: &Atom) {
        if let Some(ref s) = self.super_type() {
            s.after_remove_attr(name);
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

    /// https://dom.spec.whatwg.org/#concept-node-clone (step 5)
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
pub fn vtable_for<'a>(node: &'a &'a Node) -> &'a (VirtualMethods + 'a) {
    match node.type_id() {
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
            let element = HTMLAnchorElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAppletElement)) => {
            HTMLAppletElementCast::to_borrowed_ref(node).unwrap() as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) => {
            let element = HTMLAreaElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBaseElement)) => {
            let element = HTMLBaseElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) => {
            let element = HTMLBodyElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            let element = HTMLButtonElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)) => {
            let element = HTMLCanvasElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
            let element = HTMLFieldSetElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)) => {
            let element = HTMLFontElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement)) => {
            let element = HTMLFormElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement)) => {
            let element = HTMLHeadElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
            let element = HTMLImageElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)) => {
            let element = HTMLIFrameElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
            let element = HTMLInputElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
            let element = HTMLLinkElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            let element = HTMLObjectElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)) => {
            let element = HTMLOptGroupElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
            let element = HTMLOptionElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)) => {
            let element = HTMLScriptElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            let element = HTMLSelectElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)) => {
            let element = HTMLStyleElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)) => {
            let element =
                HTMLTableElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_))) => {
            let element =
                HTMLTableCellElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)) => {
            let element =
                HTMLTableRowElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement)) => {
            let element =
                HTMLTableSectionElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            let element = HTMLTextAreaElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement)) => {
            let element =
                HTMLTitleElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::Element) => {
            let element = ElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(_) => {
            let element = HTMLElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        _ => {
            node as &'a (VirtualMethods + 'a)
        }
    }
}
