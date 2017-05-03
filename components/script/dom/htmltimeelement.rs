/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTimeElementBinding;
use dom::bindings::codegen::Bindings::HTMLTimeElementBinding::HTMLTimeElementMethods;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::LocalName;

#[dom_struct]
pub struct HTMLTimeElement {
    htmlelement: HTMLElement,
}

impl HTMLTimeElement {
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> HTMLTimeElement {
        HTMLTimeElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTimeElement> {
        Node::reflect_node(box HTMLTimeElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLTimeElementBinding::Wrap)
    }
}

impl HTMLTimeElementMethods for HTMLTimeElement {
    // https://html.spec.whatwg.org/multipage/#dom-time-datetime
    make_getter!(DateTime, "datetime");

    // https://html.spec.whatwg.org/multipage/#dom-time-datetime
    make_setter!(SetDateTime, "datetime");
}
