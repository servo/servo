/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::HTMLAnchorElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLIFrameElementCast;
use dom::bindings::codegen::InheritTypes::HTMLImageElementCast;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::js::JS;
use dom::element::Element;
use dom::element::{HTMLAnchorElementTypeId, HTMLImageElementTypeId};
use dom::element::{HTMLIframeElementTypeId, HTMLObjectElementTypeId};
use dom::event::Event;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

use std::cast;

/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common behaviours.
/// Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods>;

    /// Called when changing or adding attributes, after the attribute's value has been updated.
    fn after_set_attr(&mut self, abstract_self: &JS<Element>, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().after_set_attr(abstract_self, name, value);
        }
    }

    /// Called when changing or removing attributes, before any modification has taken place.
    fn before_remove_attr(&mut self, abstract_self: &JS<Element>, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().before_remove_attr(abstract_self, name, value);
        }
    }

    /// Called when a Node is appended to a tree that is part of a Document.
    fn bind_to_tree(&mut self, abstract_self: &JS<Node>) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().bind_to_tree(abstract_self);
        }
    }

    /// Called when a Node is removed from a tree that is part of a Document.
    fn unbind_from_tree(&mut self, abstract_self: &JS<Node>) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().unbind_from_tree(abstract_self);
        }
   }

    /// Called during event dispatch after the bubbling phase completes.
    fn handle_event(&mut self, abstract_self: &JS<Node>, event: &JS<Event>) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().handle_event(abstract_self, event);
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any method call
/// on the trait object will invoke the corresponding method on the concrete type,
/// propagating up the parent hierarchy unless otherwise interrupted.
pub fn vtable_for<'a>(node: &'a mut JS<Node>) -> &'a mut VirtualMethods {
    unsafe {
        match node.get().type_id {
            ElementNodeTypeId(HTMLAnchorElementTypeId) => {
                let mut elem: JS<HTMLAnchorElement> = HTMLAnchorElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let mut elem: JS<HTMLImageElement> = HTMLImageElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                let mut elem: JS<HTMLIFrameElement> = HTMLIFrameElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLObjectElementTypeId) => {
                let mut elem: JS<HTMLObjectElement> = HTMLObjectElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(_) => {
                let mut elem: JS<HTMLElement> = HTMLElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            _ => {
                cast::transmute_mut_region(node.get_mut()) as &mut VirtualMethods
            }
        }
    }
}
