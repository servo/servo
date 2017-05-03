/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::EventHandlerBinding::{EventHandlerNonNull, OnBeforeUnloadEventHandlerNonNull};
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding::{self, HTMLBodyElementMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::LocalName;
use script_traits::ScriptMsg as ConstellationMsg;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use time;

/// How long we should wait before performing the initial reflow after `<body>` is parsed, in
/// nanoseconds.
const INITIAL_REFLOW_DELAY: u64 = 200_000_000;

#[dom_struct]
pub struct HTMLBodyElement {
    htmlelement: HTMLElement,
}

impl HTMLBodyElement {
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document)
                     -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLBodyElement> {
        Node::reflect_node(box HTMLBodyElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLBodyElementBinding::Wrap)
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
    fn get_background(&self) -> Option<ServoUrl>;
}

impl HTMLBodyElementLayoutHelpers for LayoutJS<HTMLBodyElement> {
    #[allow(unsafe_code)]
    fn get_background_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("bgcolor"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_color(&self) -> Option<RGBA> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("text"))
                .and_then(AttrValue::as_color)
                .cloned()
        }
    }

    #[allow(unsafe_code)]
    fn get_background(&self) -> Option<ServoUrl> {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("background"))
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
        window.upcast::<GlobalScope>().constellation_chan().send(event).unwrap();
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("bgcolor") |
            local_name!("text") => AttrValue::from_legacy_color(value.into()),
            local_name!("background") => {
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
                    &local_name!("onfocus") | &local_name!("onload") | &local_name!("onscroll") |
                    &local_name!("onafterprint") | &local_name!("onbeforeprint") |
                    &local_name!("onbeforeunload") | &local_name!("onhashchange") |
                    &local_name!("onlanguagechange") | &local_name!("onmessage") |
                    &local_name!("onoffline") | &local_name!("ononline") |
                    &local_name!("onpagehide") | &local_name!("onpageshow") |
                    &local_name!("onpopstate") | &local_name!("onstorage") |
                    &local_name!("onresize") | &local_name!("onunload") | &local_name!("onerror")
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
