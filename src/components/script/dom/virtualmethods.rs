/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLIFrameElementCast;
use dom::bindings::codegen::InheritTypes::HTMLImageElementCast;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::codegen::InheritTypes::HTMLStyleElementCast;
use dom::bindings::js::JSRef;
use dom::element::Element;
use dom::element::{ElementTypeId, HTMLImageElementTypeId};
use dom::element::{HTMLIFrameElementTypeId, HTMLObjectElementTypeId, HTMLStyleElementTypeId};
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
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:>;

    /// Called when changing or adding attributes, after the attribute's value
    /// has been updated.
    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name, value),
            _ => (),
        }
    }

    /// Called when changing or removing attributes, before any modification
    /// has taken place.
    fn before_remove_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.before_remove_attr(name, value),
            _ => (),
        }
    }

    /// Called when a Node is appended to a tree that is part of a Document.
    fn bind_to_tree(&mut self) {
        match self.super_type() {
            Some(ref mut s) => s.bind_to_tree(),
            _ => (),
        }
    }

    /// Called when a Node is removed from a tree that is part of a Document.
    fn unbind_from_tree(&mut self) {
        match self.super_type() {
            Some(ref mut s) => s.unbind_from_tree(),
            _ => (),
        }
    }

    /// Called on the parent when a node is added to its child list.
    fn child_inserted(&mut self, child: &JSRef<Node>) {
        match self.super_type() {
            Some(ref mut s) => s.child_inserted(child),
            _ => (),
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any
/// method call on the trait object will invoke the corresponding method on the
/// concrete type, propagating up the parent hierarchy unless otherwise
/// interrupted.
pub fn vtable_for<'a>(node: &'a mut JSRef<Node>) -> &'a mut VirtualMethods: {
    match node.type_id() {
        ElementNodeTypeId(HTMLImageElementTypeId) => {
            let element: &mut JSRef<HTMLImageElement> = HTMLImageElementCast::to_mut_ref(node).unwrap();
            element as &mut VirtualMethods:
        }
        ElementNodeTypeId(HTMLIFrameElementTypeId) => {
            let element: &mut JSRef<HTMLIFrameElement> = HTMLIFrameElementCast::to_mut_ref(node).unwrap();
            element as &mut VirtualMethods:
        }
        ElementNodeTypeId(HTMLObjectElementTypeId) => {
            let element: &mut JSRef<HTMLObjectElement> = HTMLObjectElementCast::to_mut_ref(node).unwrap();
            element as &mut VirtualMethods:
        }
        ElementNodeTypeId(HTMLStyleElementTypeId) => {
            let element: &mut JSRef<HTMLStyleElement> = HTMLStyleElementCast::to_mut_ref(node).unwrap();
            element as &mut VirtualMethods:
        }
        ElementNodeTypeId(ElementTypeId) => {
            let element: &mut JSRef<Element> = ElementCast::to_mut_ref(node).unwrap();
            element as &mut VirtualMethods:
        }
        ElementNodeTypeId(_) => {
            let element: &mut JSRef<HTMLElement> = HTMLElementCast::to_mut_ref(node).unwrap();
            element as &mut VirtualMethods:
        }
        _ => {
            node as &mut VirtualMethods:
        }
    }
}
