/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::element::{HTMLAnchorElementTypeId, HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::event::AbstractEvent;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::node::{Node, AbstractNode, ElementNodeTypeId};
use servo_util::str::DOMString;

use std::unstable::raw::Box;

/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common behaviours.
/// Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods>;

    /// Called during SetAttribute, before any modification has taken place.
    fn before_set_attr(&mut self, abstract_self: AbstractNode, name: DOMString, old_value: Option<DOMString>, new_value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().before_set_attr(abstract_self, name, old_value, new_value);
        }
    }

    /// Called during SetAttribute, after the attribute's value has been updated.
    fn after_set_attr(&mut self, abstract_self: AbstractNode, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().after_set_attr(abstract_self, name, value);
        }
    }

    /// Called during RemoveAttribute, before any modification has taken place.
    fn before_remove_attr(&mut self, abstract_self: AbstractNode, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().before_remove_attr(abstract_self, name, value);
        }
    }

    /// Called during RemoveAttribute, after the attribute has been removed.
    fn after_remove_attr(&mut self, abstract_self: AbstractNode, name: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().after_remove_attr(abstract_self, name);
        }
    }

    /// Called when a Node is appended to a tree that is part of a Document.
    fn bind_to_tree(&mut self, abstract_self: AbstractNode) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().bind_to_tree(abstract_self);
        }
    }

    /// Called when a Node is removed from a tree that is part of a Document.
    fn unbind_from_tree(&mut self, abstract_self: AbstractNode) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().unbind_from_tree(abstract_self);
        }
   }

    /// Called during event dispatch after the bubbling phase completes.
    fn handle_event(&mut self, abstract_self: AbstractNode, event: AbstractEvent) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().handle_event(abstract_self, event);
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any method call
/// on the trait object will invoke the corresponding method on the concrete type,
/// propagating up the parent hierarchy unless otherwise interrupted.
pub fn vtable_for(node: AbstractNode) -> &mut VirtualMethods {
    unsafe {
        match node.type_id() {
            ElementNodeTypeId(HTMLAnchorElementTypeId) => {
                let elem = node.raw_object() as *mut Box<HTMLAnchorElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                let elem = node.raw_object() as *mut Box<HTMLIFrameElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let elem = node.raw_object() as *mut Box<HTMLImageElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            ElementNodeTypeId(_) => {
                let elem = node.raw_object() as *mut Box<HTMLElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            _ => {
                let elem = node.raw_object() as *mut Box<Node>;
                &mut (*elem).data as &mut VirtualMethods
            }
        }
    }
}