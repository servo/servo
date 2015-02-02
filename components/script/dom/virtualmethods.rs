/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementCast;
use dom::bindings::codegen::InheritTypes::HTMLAreaElementCast;
use dom::bindings::codegen::InheritTypes::HTMLBodyElementCast;
use dom::bindings::codegen::InheritTypes::HTMLButtonElementCast;
use dom::bindings::codegen::InheritTypes::HTMLCanvasElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFieldSetElementCast;
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
use dom::bindings::js::JSRef;
use dom::document::Document;
use dom::element::Element;
use dom::element::ElementTypeId;
use dom::event::Event;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcanvaselement::HTMLCanvasElement;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::{Node, NodeHelpers, NodeTypeId, CloneChildrenFlag};

use util::str::DOMString;

use string_cache::Atom;

/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common
/// behaviours. Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    /// Returns self as the superclass of the implementation for this trait,
    /// if any.
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods>;

    /// Called when changing or adding attributes, after the attribute's value
    /// has been updated.
    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => (),
        }
    }

    /// Called when changing or removing attributes, before any modification
    /// has taken place.
    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => (),
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
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }
    }

    /// Called when a Node is removed from a tree, where 'tree_in_doc'
    /// indicates whether the tree is part of a Document.
    fn unbind_from_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.unbind_from_tree(tree_in_doc),
            _ => (),
        }
    }

    /// Called on the parent when a node is added to its child list.
    fn child_inserted(&self, child: JSRef<Node>) {
        match self.super_type() {
            Some(ref s) => s.child_inserted(child),
            _ => (),
        }
    }

    /// Called during event dispatch after the bubbling phase completes.
    fn handle_event(&self, event: JSRef<Event>) {
        match self.super_type() {
            Some(s) => {
                s.handle_event(event);
            }
            _ => (),
        }
    }

    /// https://dom.spec.whatwg.org/#concept-node-clone (step 5)
    fn cloning_steps(&self, copy: JSRef<Node>, maybe_doc: Option<JSRef<Document>>,
                     clone_children: CloneChildrenFlag) {
        match self.super_type() {
            Some(ref s) => s.cloning_steps(copy, maybe_doc, clone_children),
            _ => (),
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any
/// method call on the trait object will invoke the corresponding method on the
/// concrete type, propagating up the parent hierarchy unless otherwise
/// interrupted.
pub fn vtable_for<'a>(node: &'a JSRef<'a, Node>) -> &'a (VirtualMethods + 'a) {
    match node.type_id() {
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
            let element: &'a JSRef<'a, HTMLAnchorElement> = HTMLAnchorElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) => {
            let element: &'a JSRef<'a, HTMLAreaElement> = HTMLAreaElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) => {
            let element: &'a JSRef<'a, HTMLBodyElement> = HTMLBodyElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            let element: &'a JSRef<'a, HTMLButtonElement> = HTMLButtonElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)) => {
            let element: &'a JSRef<'a, HTMLCanvasElement> = HTMLCanvasElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
            let element: &'a JSRef<'a, HTMLFieldSetElement> = HTMLFieldSetElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
            let element: &'a JSRef<'a, HTMLImageElement> = HTMLImageElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)) => {
            let element: &'a JSRef<'a, HTMLIFrameElement> = HTMLIFrameElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
            let element: &'a JSRef<'a, HTMLInputElement> = HTMLInputElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
            let element: &'a JSRef<'a, HTMLLinkElement> = HTMLLinkElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            let element: &'a JSRef<'a, HTMLObjectElement> = HTMLObjectElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)) => {
            let element: &'a JSRef<'a, HTMLOptGroupElement> = HTMLOptGroupElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
            let element: &'a JSRef<'a, HTMLOptionElement> = HTMLOptionElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)) => {
            let element: &'a JSRef<'a, HTMLScriptElement> = HTMLScriptElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            let element: &'a JSRef<'a, HTMLSelectElement> = HTMLSelectElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)) => {
            let element: &'a JSRef<'a, HTMLStyleElement> = HTMLStyleElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)) => {
            let element: &'a JSRef<'a, HTMLTableElement> =
                HTMLTableElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_))) => {
            let element: &'a JSRef<'a, HTMLTableCellElement> =
                HTMLTableCellElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)) => {
            let element: &'a JSRef<'a, HTMLTableRowElement> =
                HTMLTableRowElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement)) => {
            let element: &'a JSRef<'a, HTMLTableSectionElement> =
                HTMLTableSectionElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            let element: &'a JSRef<'a, HTMLTextAreaElement> = HTMLTextAreaElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement)) => {
            let element: &'a JSRef<'a, HTMLTitleElement> =
                HTMLTitleElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(ElementTypeId::Element) => {
            let element: &'a JSRef<'a, Element> = ElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        NodeTypeId::Element(_) => {
            let element: &'a JSRef<'a, HTMLElement> = HTMLElementCast::to_borrowed_ref(node).unwrap();
            element as &'a (VirtualMethods + 'a)
        }
        _ => {
            node as &'a (VirtualMethods + 'a)
        }
    }
}
