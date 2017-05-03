/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::inheritance::Castable;
use dom::bindings::inheritance::ElementTypeId;
use dom::bindings::inheritance::HTMLElementTypeId;
use dom::bindings::inheritance::NodeTypeId;
use dom::bindings::inheritance::SVGElementTypeId;
use dom::bindings::inheritance::SVGGraphicsElementTypeId;
use dom::bindings::str::DOMString;
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
use dom::htmldetailselement::HTMLDetailsElement;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::HTMLFontElement;
use dom::htmlformelement::HTMLFormElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhrelement::HTMLHRElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmllabelelement::HTMLLabelElement;
use dom::htmllielement::HTMLLIElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlmediaelement::HTMLMediaElement;
use dom::htmlmetaelement::HTMLMetaElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::htmloutputelement::HTMLOutputElement;
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
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node, UnbindContext};
use dom::svgsvgelement::SVGSVGElement;
use html5ever::LocalName;
use style::attr::AttrValue;

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
    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match self.super_type() {
            Some(ref s) => s.parse_plain_attribute(name, value),
            _ => AttrValue::String(value.into()),
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
    /// Implements removing steps:
    /// https://dom.spec.whatwg.org/#concept-node-remove-ext
    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(context);
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

    /// Called on an element when it is popped off the stack of open elements
    /// of a parser.
    fn pop(&self) {
        if let Some(ref s) = self.super_type() {
            s.pop();
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
            node.downcast::<HTMLAnchorElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAppletElement)) => {
            node.downcast::<HTMLAppletElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) => {
            node.downcast::<HTMLAreaElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBaseElement)) => {
            node.downcast::<HTMLBaseElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) => {
            node.downcast::<HTMLBodyElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            node.downcast::<HTMLButtonElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)) => {
            node.downcast::<HTMLCanvasElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDetailsElement)) => {
            node.downcast::<HTMLDetailsElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
            node.downcast::<HTMLFieldSetElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)) => {
            node.downcast::<HTMLFontElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement)) => {
            node.downcast::<HTMLFormElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement)) => {
            node.downcast::<HTMLHeadElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHRElement)) => {
            node.downcast::<HTMLHRElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
            node.downcast::<HTMLImageElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)) => {
            node.downcast::<HTMLIFrameElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
            node.downcast::<HTMLInputElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement)) => {
            node.downcast::<HTMLLabelElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLIElement)) => {
            node.downcast::<HTMLLIElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
            node.downcast::<HTMLLinkElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMediaElement(_))) => {
            node.downcast::<HTMLMediaElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMetaElement)) => {
            node.downcast::<HTMLMetaElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            node.downcast::<HTMLObjectElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)) => {
            node.downcast::<HTMLOptGroupElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
            node.downcast::<HTMLOptionElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)) => {
            node.downcast::<HTMLOutputElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)) => {
            node.downcast::<HTMLScriptElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            node.downcast::<HTMLSelectElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)) => {
            node.downcast::<HTMLStyleElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)) => {
            node.downcast::<HTMLTableElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement(_))) => {
            node.downcast::<HTMLTableCellElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)) => {
            node.downcast::<HTMLTableRowElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement)) => {
            node.downcast::<HTMLTableSectionElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTemplateElement)) => {
            node.downcast::<HTMLTemplateElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            node.downcast::<HTMLTextAreaElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement)) => {
            node.downcast::<HTMLTitleElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::SVGElement(SVGElementTypeId::SVGGraphicsElement(
                    SVGGraphicsElementTypeId::SVGSVGElement
                ))) => {
            node.downcast::<SVGSVGElement>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(ElementTypeId::Element) => {
            node.downcast::<Element>().unwrap() as &VirtualMethods
        }
        NodeTypeId::Element(_) => {
            node.downcast::<HTMLElement>().unwrap() as &VirtualMethods
        }
        _ => {
            node as &VirtualMethods
        }
    }
}
