/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFormElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, AttributeHandlers, HTMLFormElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use servo_util::str::DOMString;
use std::ascii::OwnedStrAsciiExt;


#[jstraceable]
#[must_root]
pub struct HTMLFormElement {
    pub htmlelement: HTMLElement,
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFormElementTypeId))
    }
}

impl HTMLFormElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLFormElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFormElementBinding::Wrap)
    }
}

impl<'a> HTMLFormElementMethods for JSRef<'a, HTMLFormElement> {
    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-acceptcharset
    make_getter!(AcceptCharset, "accept-charset")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-action
    fn Action(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        let url = element.get_url_attribute("src");
        match url.as_slice() {
            "" => {
                let window = window_from_node(self).root();
                window.get_url().serialize()
            },
            _ => url
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-action
    make_setter!(SetAction, "action")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-autocomplete
    fn Autocomplete(self) -> DOMString {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let ac = elem.get_string_attribute("autocomplete").into_ascii_lower();
        // https://html.spec.whatwg.org/multipage/forms.html#attr-form-autocomplete
        match ac.as_slice() {
            "off" => ac,
            _ => "on".to_string()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-enctype
    fn Enctype(self) -> DOMString {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let enctype = elem.get_string_attribute("enctype").into_ascii_lower();
        // https://html.spec.whatwg.org/multipage/forms.html#attr-fs-enctype
        match enctype.as_slice() {
            "text/plain" | "multipart/form-data" => enctype,
            _ => "application/x-www-form-urlencoded".to_string()
        }
    }


    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-enctype
    make_setter!(SetEnctype, "enctype")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-encoding
    fn Encoding(self) -> DOMString {
        self.Enctype()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-encoding
    fn SetEncoding(self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-method
    fn Method(self) -> DOMString {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let method = elem.get_string_attribute("method").into_ascii_lower();
        // https://html.spec.whatwg.org/multipage/forms.html#attr-fs-method
        match method.as_slice() {
            "post" | "dialog" => method,
            _ => "get".to_string()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-method
    make_setter!(SetMethod, "method")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-name
    make_getter!(Name)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-name
    make_setter!(SetName, "name")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-novalidate
    make_bool_getter!(NoValidate)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-novalidate
    make_bool_setter!(SetNoValidate, "novalidate")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-target
    make_getter!(Target)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-target
    make_setter!(SetTarget, "target")
}

impl Reflectable for HTMLFormElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
