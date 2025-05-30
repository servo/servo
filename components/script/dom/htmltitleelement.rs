/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, ChildrenMutation, Node, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLTitleElement {
    htmlelement: HTMLElement,
    popped: Cell<bool>,
}

impl HTMLTitleElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            popped: Cell::new(false),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLTitleElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLTitleElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    fn notify_title_changed(&self) {
        let node = self.upcast::<Node>();
        if node.is_in_a_document_tree() {
            let owner_doc = node.owner_doc();
            owner_doc.update_title_element();
            owner_doc.title_changed();
        }
    }

    pub(crate) fn text(&self) -> DOMString {
        self.upcast::<Node>().child_text_content()
    }
}

impl HTMLTitleElementMethods<crate::DomTypeHolder> for HTMLTitleElement {
    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn Text(&self) -> DOMString {
        self.text()
    }

    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn SetText(&self, value: DOMString, can_gc: CanGc) {
        self.upcast::<Node>().SetTextContent(Some(value), can_gc);
        self.notify_title_changed();
    }
}

impl VirtualMethods for HTMLTitleElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }

        // Notify of title changes only after the initial full parsing
        // of the element.
        if self.popped.get() {
            self.notify_title_changed();
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }
        if context.tree_is_in_a_document_tree {
            let owner_doc = self.upcast::<Node>().owner_doc();
            owner_doc.update_title_element();
            owner_doc.title_changed();
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context, can_gc);
        }
        if context.tree_is_in_a_document_tree {
            let owner_doc = self.upcast::<Node>().owner_doc();
            owner_doc.update_title_element();
            owner_doc.title_changed();
        }
    }

    fn pop(&self) {
        if let Some(s) = self.super_type() {
            s.pop();
        }

        self.popped.set(true);

        // Initial notification of title change, once the full text
        // is available.
        self.notify_title_changed();
    }
}
