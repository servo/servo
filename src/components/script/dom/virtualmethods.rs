/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{AttrValue, StringAttrValue};
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementCast;
use dom::bindings::codegen::InheritTypes::HTMLBodyElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLIFrameElementCast;
use dom::bindings::codegen::InheritTypes::HTMLImageElementCast;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::codegen::InheritTypes::HTMLStyleElementCast;
use dom::bindings::js::JSRef;
use dom::element::Element;
use dom::element::{ElementTypeId, HTMLAnchorElementTypeId, HTMLBodyElementTypeId, HTMLImageElementTypeId};
use dom::element::{HTMLIFrameElementTypeId, HTMLObjectElementTypeId, HTMLStyleElementTypeId};
use dom::event::Event;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmlstyleelement::HTMLStyleElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};
use servo_util::str::DOMString;

/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common
/// behaviours. Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    /// Returns self as the superclass of the implementation for this trait,
    /// if any.
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods+>;

    /// Called when changing or adding attributes, after the attribute's value
    /// has been updated.
    fn after_set_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value),
            _ => (),
        }
    }

    /// Called when changing or removing attributes, before any modification
    /// has taken place.
    fn before_remove_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value),
            _ => (),
        }
    }

    /// Returns the right AttrValue variant for the attribute with name `name`
    /// on this element.
    fn parse_plain_attribute(&self, name: &str, value: DOMString) -> AttrValue {
        match self.super_type() {
            Some(ref s) => s.parse_plain_attribute(name, value),
            _ => StringAttrValue(value),
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
    fn child_inserted(&self, child: &JSRef<Node>) {
        match self.super_type() {
            Some(ref s) => s.child_inserted(child),
            _ => (),
        }
    }

    /// Called during event dispatch after the bubbling phase completes.
    fn handle_event(&self, event: &JSRef<Event>) {
        match self.super_type() {
            Some(s) => {
                s.handle_event(event);
            }
            _ => (),
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any
/// method call on the trait object will invoke the corresponding method on the
/// concrete type, propagating up the parent hierarchy unless otherwise
/// interrupted.
pub fn vtable_for<'a>(node: &'a JSRef<Node>) -> &'a VirtualMethods+ {
    match node.type_id() {
        ElementNodeTypeId(HTMLAnchorElementTypeId) => {
            let element: &JSRef<HTMLAnchorElement> = HTMLAnchorElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(HTMLBodyElementTypeId) => {
            let element: &JSRef<HTMLBodyElement> = HTMLBodyElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(HTMLImageElementTypeId) => {
            let element: &JSRef<HTMLImageElement> = HTMLImageElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(HTMLIFrameElementTypeId) => {
            let element: &JSRef<HTMLIFrameElement> = HTMLIFrameElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(HTMLObjectElementTypeId) => {
            let element: &JSRef<HTMLObjectElement> = HTMLObjectElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(HTMLStyleElementTypeId) => {
            let element: &JSRef<HTMLStyleElement> = HTMLStyleElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(ElementTypeId) => {
            let element: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        ElementNodeTypeId(_) => {
            let element: &JSRef<HTMLElement> = HTMLElementCast::to_ref(node).unwrap();
            element as &VirtualMethods+
        }
        _ => {
            node as &VirtualMethods+
        }
    }
}
