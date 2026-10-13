/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::ElementCreator;
use crate::dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{BindContext, ChildrenMutation, Node};

#[dom_struct]
pub(crate) struct HTMLTitleElement {
    htmlelement: HTMLElement,
    /// Whether this element is on the HTML parsers stack of open elements.
    ///
    /// While this is the case  we don't bother incrementally
    /// updating the document title.
    is_currently_being_parsed: Cell<bool>,
}

impl HTMLTitleElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        is_parser_created: bool,
    ) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            is_currently_being_parsed: Cell::new(is_parser_created),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
    ) -> DomRoot<HTMLTitleElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLTitleElement::new_inherited(
                local_name,
                prefix,
                document,
                creator.is_parser_created(),
            )),
            document,
            proto,
        )
    }

    fn notify_title_changed(&self) {
        let node = self.upcast::<Node>();
        if node.is_in_a_document_tree() {
            node.owner_doc().title_changed();
        }
    }
}

impl HTMLTitleElementMethods<crate::DomTypeHolder> for HTMLTitleElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-title-text>
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().child_text_content()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-title-text>
    fn SetText(&self, cx: &mut JSContext, value: DOMString) {
        self.upcast::<Node>()
            .set_text_content_for_element(cx, Some(value))
    }
}

impl VirtualMethods for HTMLTitleElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(cx, mutation);
        }

        // Notify of title changes only after the initial full parsing
        // of the element.
        if !self.is_currently_being_parsed.get() {
            self.notify_title_changed();
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
        }
        let node = self.upcast::<Node>();
        if context.tree_is_in_a_document_tree {
            node.owner_doc().title_changed();
        }
    }

    fn pop(&self, cx: &mut js::context::JSContext) {
        if let Some(s) = self.super_type() {
            s.pop(cx);
        }

        self.is_currently_being_parsed.set(false);

        // Initial notification of title change, once the full text
        // is available.
        self.notify_title_changed();
    }
}
