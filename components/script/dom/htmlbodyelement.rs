/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLBodyElementBinding::{self, HTMLBodyElementMethods};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLElementCast};
use dom::bindings::js::Root;
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::element::{AttributeMutation, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node, document_from_node};
use dom::virtualmethods::VirtualMethods;
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::Msg as ConstellationMsg;

use cssparser::RGBA;
use url::{Url, UrlParser};
use util::str::{self, DOMString};

use std::borrow::ToOwned;
use std::cell::Cell;
use std::rc::Rc;
use time;

/// How long we should wait before performing the initial reflow after `<body>` is parsed, in
/// nanoseconds.
const INITIAL_REFLOW_DELAY: u64 = 200_000_000;

#[dom_struct]
pub struct HTMLBodyElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
    background: DOMRefCell<Option<Url>>
}

impl HTMLBodyElementDerived for EventTarget {
    fn is_htmlbodyelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLBodyElement)))
    }
}

impl HTMLBodyElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document)
                     -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLBodyElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
            background: DOMRefCell::new(None)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBodyElementBinding::Wrap)
    }
}

impl HTMLBodyElementMethods for HTMLBodyElement {
    // https://html.spec.whatwg.org/multipage#dom-body-bgcolor
    make_getter!(BgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage#dom-body-bgcolor
    make_setter!(SetBgColor, "bgcolor");

    // https://html.spec.whatwg.org/multipage/#the-body-element
    fn GetOnunload(&self) -> Option<Rc<EventHandlerNonNull>> {
        let win = window_from_node(self);
        win.r().GetOnunload()
    }

    // https://html.spec.whatwg.org/multipage/#the-body-element
    fn SetOnunload(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        let win = window_from_node(self);
        win.r().SetOnunload(listener)
    }
}


impl HTMLBodyElement {
    pub fn get_background_color(&self) -> Option<RGBA> {
        self.background_color.get()
    }

    #[allow(unsafe_code)]
    pub fn get_background(&self) -> Option<Url> {
        unsafe {
            self.background.borrow_for_layout().clone()
        }
    }
}

impl VirtualMethods for HTMLBodyElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(element as &VirtualMethods)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if !tree_in_doc {
            return
        }

        let window = window_from_node(self);
        let document = window.r().Document();
        document.r().set_reflow_timeout(time::precise_time_ns() + INITIAL_REFLOW_DELAY);
        let ConstellationChan(ref chan) = window.r().constellation_chan();
        let event = ConstellationMsg::HeadParsed;
        chan.send(event).unwrap();
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match (attr.local_name(), mutation) {
            (&atom!(bgcolor), _) => {
                self.background_color.set(mutation.new_value(attr).and_then(|value| {
                    str::parse_legacy_color(&value).ok()
                }));
            },
            (&atom!(background), _) => {
                *self.background.borrow_mut() = mutation.new_value(attr).and_then(|value| {
                    let base = document_from_node(self).url();
                    UrlParser::new().base_url(&base).parse(&value).ok()
                });
            },
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                let window = window_from_node(self);
                let (cx, url, reflector) = (window.get_cx(),
                                            window.get_url(),
                                            window.reflector().get_jsobject());
                let evtarget = match name {
                    &atom!(onfocus) | &atom!(onload) | &atom!(onscroll) | &atom!(onafterprint) |
                    &atom!(onbeforeprint) | &atom!(onbeforeunload) | &atom!(onhashchange) |
                    &atom!(onlanguagechange) | &atom!(onmessage) | &atom!(onoffline) | &atom!(ononline) |
                    &atom!(onpagehide) | &atom!(onpageshow) | &atom!(onpopstate) | &atom!(onstorage) |
                    &atom!(onresize) | &atom!(onunload) | &atom!(onerror)
                      => EventTargetCast::from_ref(window.r()), // forwarded event
                    _ => EventTargetCast::from_ref(self),
                };
                evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                      &name[2..],
                                                      (**attr.value()).to_owned());
            },
            _ => {}
        }
    }
}
