/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::EventHandlerBinding::{EventHandlerNonNull, OnBeforeUnloadEventHandlerNonNull};
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding::{self, HTMLBodyElementMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use script_traits::ScriptMsg as ConstellationMsg;
use string_cache::Atom;
use time;
use url::Url;

/// How long we should wait before performing the initial reflow after `<body>` is parsed, in
/// nanoseconds.
const INITIAL_REFLOW_DELAY: u64 = 200_000_000;

#[dom_struct]
pub struct HTMLBodyElement {
    htmlelement: HTMLElement,
}

impl HTMLBodyElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document)
                     -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBodyElementBinding::Wrap)
    }

    /// https://drafts.csswg.org/cssom-view/#the-html-body-element
    pub fn is_the_html_body_element(&self) -> bool {
        let self_node = self.upcast::<Node>();
        let root_elem = self.upcast::<Element>().root_element();
        let root_node = root_elem.upcast::<Node>();
        root_node.is_parent_of(self_node) &&
            self_node.preceding_siblings().all(|n| !n.is::<HTMLBodyElement>())
    }

}

impl HTMLBodyElementMethods for HTMLBodyElement {
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
    make_url_setter!(SetBackground, "background");

    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!(ForwardToWindow);
}

pub trait HTMLBodyElementLayoutHelpers {
    fn get_background_color(&self) -> Option<RGBA>;
    fn get_color(&self) -> Option<RGBA>;
    fn get_background(&self) -> Option<Url>;
}

impl HTMLBodyElementLayoutHelpers for LayoutJS<HTMLBodyElement> {
    #[allow(unsafe_code)]
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("bgcolor"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("text"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_background(&self) -> Option<Url> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("background"))
                .and_then(AttrValue::as_url)
                .cloned()
        }
    }
}

impl VirtualMethods for HTMLBodyElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if !tree_in_doc {
            return
        }

        let window = window_from_node(self);
        let document = window.Document();
        document.set_reflow_timeout(time::precise_time_ns() + INITIAL_REFLOW_DELAY);
        let event = ConstellationMsg::HeadParsed;
        window.constellation_chan().send(event).unwrap();
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match *name {
            atom!("bgcolor") |
            atom!("text") => AttrValue::from_legacy_color(value.into()),
            atom!("background") => {
                AttrValue::from_url(document_from_node(self).url(), value.into())
            },
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        let do_super_mutate = match (attr.local_name(), mutation) {
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                let window = window_from_node(self);
                // https://html.spec.whatwg.org/multipage/
                // #event-handlers-on-elements,-document-objects,-and-window-objects:event-handlers-3
                match name {
                    &atom!("onfocus") | &atom!("onload") | &atom!("onscroll") | &atom!("onafterprint") |
                    &atom!("onbeforeprint") | &atom!("onbeforeunload") | &atom!("onhashchange") |
                    &atom!("onlanguagechange") | &atom!("onmessage") | &atom!("onoffline") | &atom!("ononline") |
                    &atom!("onpagehide") | &atom!("onpageshow") | &atom!("onpopstate") | &atom!("onstorage") |
                    &atom!("onresize") | &atom!("onunload") | &atom!("onerror")
                      => {
                          let evtarget = window.upcast::<EventTarget>(); // forwarded event
                          let source_line = 1; //TODO(#9604) obtain current JS execution line
                          evtarget.set_event_handler_uncompiled(window.get_url(),
                                                                source_line,
                                                                &name[2..],
                                                                DOMString::from((**attr.value()).to_owned()));
                          false
                    }
                    _ => true, // HTMLElement::attribute_mutated will take care of this.
                }
            },
            _ => true,
        };

        if do_super_mutate {
            self.super_type().unwrap().attribute_mutated(attr, mutation);
        }
    }
}
