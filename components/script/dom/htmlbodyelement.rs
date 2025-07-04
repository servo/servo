/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::{EmbedderMsg, LoadStatus};
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::color::AbsoluteColor;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLBodyElementBinding::HTMLBodyElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, Node, NodeTraits};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLBodyElement {
    htmlelement: HTMLElement,
}

impl HTMLBodyElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLBodyElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLBodyElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLBodyElementMethods<crate::DomTypeHolder> for HTMLBodyElement {
    // https://html.spec.whatwg.org/multipage/#dom-body-bgcolor
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-body-bgcolor
    make_legacy_color_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#dom-body-text
    make_getter!(Text, "text");

    // https://html.spec.whatwg.org/multipage/#dom-body-text
    make_legacy_color_setter!(SetText, "text");

    // https://html.spec.whatwg.org/multipage/#dom-body-background
    make_getter!(Background, "background");

    // https://html.spec.whatwg.org/multipage/#dom-body-background
    fn SetBackground(&self, input: DOMString, can_gc: CanGc) {
        let value =
            AttrValue::from_resolved_url(&self.owner_document().base_url().get_arc(), input.into());
        self.upcast::<Element>()
            .set_attribute(&local_name!("background"), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!(ForwardToWindow);
}

pub(crate) trait HTMLBodyElementLayoutHelpers {
    fn get_background_color(self) -> Option<AbsoluteColor>;
    fn get_color(self) -> Option<AbsoluteColor>;
    fn get_background(self) -> Option<ServoUrl>;
}

impl HTMLBodyElementLayoutHelpers for LayoutDom<'_, HTMLBodyElement> {
    fn get_background_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    fn get_color(self) -> Option<AbsoluteColor> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("text"))
            .and_then(AttrValue::as_color)
            .cloned()
    }

    fn get_background(self) -> Option<ServoUrl> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("background"))
            .and_then(AttrValue::as_resolved_url)
            .cloned()
            .map(Into::into)
    }
}

impl VirtualMethods for HTMLBodyElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        if attr.local_name() == &local_name!("bgcolor") {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        if !context.tree_is_in_a_document_tree {
            return;
        }

        let window = self.owner_window();
        window.prevent_layout_until_load_event();
        if window.is_top_level() {
            window.send_to_embedder(EmbedderMsg::NotifyLoadStatusChanged(
                window.webview_id(),
                LoadStatus::HeadParsed,
            ));
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("bgcolor") | local_name!("text") => {
                AttrValue::from_legacy_color(value.into())
            },
            local_name!("background") => AttrValue::from_resolved_url(
                &self.owner_document().base_url().get_arc(),
                value.into(),
            ),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        let do_super_mutate = match (attr.local_name(), mutation) {
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                let window = self.owner_window();
                // https://html.spec.whatwg.org/multipage/
                // #event-handlers-on-elements,-document-objects,-and-window-objects:event-handlers-6
                match name {
                    &local_name!("onafterprint") |
                    &local_name!("onbeforeprint") |
                    &local_name!("onbeforeunload") |
                    &local_name!("onerror") |
                    &local_name!("onfocus") |
                    &local_name!("onhashchange") |
                    &local_name!("onload") |
                    &local_name!("onlanguagechange") |
                    &local_name!("onmessage") |
                    &local_name!("onmessageerror") |
                    &local_name!("onoffline") |
                    &local_name!("ononline") |
                    &local_name!("onpagehide") |
                    &local_name!("onpagereveal") |
                    &local_name!("onpageshow") |
                    &local_name!("onpageswap") |
                    &local_name!("onpopstate") |
                    &local_name!("onrejectionhandled") |
                    &local_name!("onresize") |
                    &local_name!("onscroll") |
                    &local_name!("onstorage") |
                    &local_name!("onunhandledrejection") |
                    &local_name!("onunload") => {
                        let source = &**attr.value();
                        let evtarget = window.upcast::<EventTarget>(); // forwarded event
                        let source_line = 1; //TODO(#9604) obtain current JS execution line
                        evtarget.set_event_handler_uncompiled(
                            window.get_url(),
                            source_line,
                            &name[2..],
                            source,
                        );
                        false
                    },
                    _ => true, // HTMLElement::attribute_mutated will take care of this.
                }
            },
            _ => true,
        };

        if do_super_mutate {
            self.super_type()
                .unwrap()
                .attribute_mutated(attr, mutation, can_gc);
        }
    }
}
