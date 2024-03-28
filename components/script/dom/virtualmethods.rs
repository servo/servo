/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::LocalName;
use style::attr::AttrValue;

use super::htmltablecolelement::HTMLTableColElement;
use crate::dom::attr::Attr;
use crate::dom::bindings::inheritance::{
    Castable, ElementTypeId, HTMLElementTypeId, HTMLMediaElementTypeId, NodeTypeId,
    SVGElementTypeId, SVGGraphicsElementTypeId,
};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlbaseelement::HTMLBaseElement;
use crate::dom::htmlbodyelement::HTMLBodyElement;
use crate::dom::htmlbuttonelement::HTMLButtonElement;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::htmldetailselement::HTMLDetailsElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlfontelement::HTMLFontElement;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::htmlhrelement::HTMLHRElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::htmllabelelement::HTMLLabelElement;
use crate::dom::htmllielement::HTMLLIElement;
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::htmlmetaelement::HTMLMetaElement;
use crate::dom::htmlobjectelement::HTMLObjectElement;
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmloutputelement::HTMLOutputElement;
use crate::dom::htmlpreelement::HTMLPreElement;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::htmlstyleelement::HTMLStyleElement;
use crate::dom::htmltablecellelement::HTMLTableCellElement;
use crate::dom::htmltableelement::HTMLTableElement;
use crate::dom::htmltablerowelement::HTMLTableRowElement;
use crate::dom::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::htmltemplateelement::HTMLTemplateElement;
use crate::dom::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::htmltitleelement::HTMLTitleElement;
use crate::dom::htmlvideoelement::HTMLVideoElement;
use crate::dom::node::{BindContext, ChildrenMutation, CloneChildrenFlag, Node, UnbindContext};
use crate::dom::svgelement::SVGElement;
use crate::dom::svgsvgelement::SVGSVGElement;

/// Trait to allow DOM nodes to opt-in to overriding (or adding to) common
/// behaviours. Replicates the effect of C++ virtual methods.
pub trait VirtualMethods {
    /// Returns self as the superclass of the implementation for this trait,
    /// if any.
    fn super_type(&self) -> Option<&dyn VirtualMethods>;

    /// Called when attributes of a node are mutated.
    /// <https://dom.spec.whatwg.org/#attribute-is-set>
    /// <https://dom.spec.whatwg.org/#attribute-is-removed>
    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }
    }

    /// Returns `true` if given attribute `attr` affects style of the
    /// given element.
    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        match self.super_type() {
            Some(s) => s.attribute_affects_presentational_hints(attr),
            None => false,
        }
    }

    /// Returns the right AttrValue variant for the attribute with name `name`
    /// on this element.
    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match self.super_type() {
            Some(s) => s.parse_plain_attribute(name, value),
            _ => AttrValue::String(value.into()),
        }
    }

    /// Called when a Node is appended to a tree, where 'tree_connected' indicates
    /// whether the tree is part of a Document.
    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }
    }

    /// Called when a Node is removed from a tree, where 'tree_connected'
    /// indicates whether the tree is part of a Document.
    /// Implements removing steps:
    /// <https://dom.spec.whatwg.org/#concept-node-remove-ext>
    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context);
        }
    }

    /// Called on the parent when its children are changed.
    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }
    }

    /// Called during event dispatch after the bubbling phase completes.
    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-adopt-ext>
    fn adopting_steps(&self, old_doc: &Document) {
        if let Some(s) = self.super_type() {
            s.adopting_steps(old_doc);
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-clone-ext>
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children);
        }
    }

    /// Called on an element when it is popped off the stack of open elements
    /// of a parser.
    fn pop(&self) {
        if let Some(s) = self.super_type() {
            s.pop();
        }
    }
}

/// Obtain a VirtualMethods instance for a given Node-derived object. Any
/// method call on the trait object will invoke the corresponding method on the
/// concrete type, propagating up the parent hierarchy unless otherwise
/// interrupted.
pub fn vtable_for(node: &Node) -> &dyn VirtualMethods {
    match node.type_id() {
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
            node.downcast::<HTMLAnchorElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) => {
            node.downcast::<HTMLAreaElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBaseElement)) => {
            node.downcast::<HTMLBaseElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement)) => {
            node.downcast::<HTMLBodyElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            node.downcast::<HTMLButtonElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement)) => {
            node.downcast::<HTMLCanvasElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDetailsElement)) => {
            node.downcast::<HTMLDetailsElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
            node.downcast::<HTMLFieldSetElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)) => {
            node.downcast::<HTMLFontElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement)) => {
            node.downcast::<HTMLFormElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement)) => {
            node.downcast::<HTMLHeadElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHRElement)) => {
            node.downcast::<HTMLHRElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
            node.downcast::<HTMLImageElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)) => {
            node.downcast::<HTMLIFrameElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
            node.downcast::<HTMLInputElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement)) => {
            node.downcast::<HTMLLabelElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLIElement)) => {
            node.downcast::<HTMLLIElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
            node.downcast::<HTMLLinkElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMediaElement(
            media_el,
        ))) => match media_el {
            HTMLMediaElementTypeId::HTMLVideoElement => {
                node.downcast::<HTMLVideoElement>().unwrap() as &dyn VirtualMethods
            },
            _ => node.downcast::<HTMLMediaElement>().unwrap() as &dyn VirtualMethods,
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMetaElement)) => {
            node.downcast::<HTMLMetaElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            node.downcast::<HTMLObjectElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)) => {
            node.downcast::<HTMLOptGroupElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
            node.downcast::<HTMLOptionElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)) => {
            node.downcast::<HTMLOutputElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLPreElement)) => {
            node.downcast::<HTMLPreElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)) => {
            node.downcast::<HTMLScriptElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            node.downcast::<HTMLSelectElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSourceElement)) => {
            node.downcast::<HTMLSourceElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLStyleElement)) => {
            node.downcast::<HTMLStyleElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement)) => {
            node.downcast::<HTMLTableElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(
            HTMLElementTypeId::HTMLTableCellElement,
        )) => node.downcast::<HTMLTableCellElement>().unwrap() as &dyn VirtualMethods,
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableColElement)) => {
            node.downcast::<HTMLTableColElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement)) => {
            node.downcast::<HTMLTableRowElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(
            HTMLElementTypeId::HTMLTableSectionElement,
        )) => node.downcast::<HTMLTableSectionElement>().unwrap() as &dyn VirtualMethods,
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTemplateElement)) => {
            node.downcast::<HTMLTemplateElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            node.downcast::<HTMLTextAreaElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement)) => {
            node.downcast::<HTMLTitleElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::SVGElement(SVGElementTypeId::SVGGraphicsElement(
            SVGGraphicsElementTypeId::SVGSVGElement,
        ))) => node.downcast::<SVGSVGElement>().unwrap() as &dyn VirtualMethods,
        NodeTypeId::Element(ElementTypeId::SVGElement(SVGElementTypeId::SVGElement)) => {
            node.downcast::<SVGElement>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(ElementTypeId::Element) => {
            node.downcast::<Element>().unwrap() as &dyn VirtualMethods
        },
        NodeTypeId::Element(_) => node.downcast::<HTMLElement>().unwrap() as &dyn VirtualMethods,
        _ => node as &dyn VirtualMethods,
    }
}
