/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use script_bindings::str::DOMString;
use stylo_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::SVGElementBinding::SVGElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct SVGElement {
    element: Element,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
}

impl SVGElement {
    fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGElement {
        SVGElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGElement {
        SVGElement {
            element: Element::new_inherited_with_state(state, tag_name, ns!(svg), prefix, document),
            style_decl: Default::default(),
        }
    }

    pub(crate) fn new(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<SVGElement> {
        Node::reflect_node_with_proto(
            Box::new(SVGElement::new_inherited(tag_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}

impl VirtualMethods for SVGElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.as_element() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        let element = self.as_element();
        if let (&local_name!("nonce"), mutation) = (attr.local_name(), mutation) {
            match mutation {
                AttributeMutation::Set(_) => {
                    let nonce = &**attr.value();
                    element.update_nonce_internal_slot(nonce.to_owned());
                },
                AttributeMutation::Removed => {
                    element.update_nonce_internal_slot(String::new());
                },
            }
        }
    }
}

impl SVGElementMethods<crate::DomTypeHolder> for SVGElement {
    // https://html.spec.whatwg.org/multipage/#the-style-attribute
    fn Style(&self) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = self.owner_window();
            CSSStyleDeclaration::new(
                &global,
                CSSStyleOwner::Element(Dom::from_ref(self.upcast())),
                None,
                CSSModificationAccess::ReadWrite,
                CanGc::note(),
            )
        })
    }

    // <https://html.spec.whatwg.org/multipage/#globaleventhandlers>
    global_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce
    fn Nonce(&self) -> DOMString {
        self.as_element().nonce_value().into()
    }

    // https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce
    fn SetNonce(&self, value: DOMString) {
        self.as_element()
            .update_nonce_internal_slot(value.to_string())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-autofocus
    fn Autofocus(&self) -> bool {
        self.element.has_attribute(&local_name!("autofocus"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-autofocus
    fn SetAutofocus(&self, autofocus: bool, can_gc: CanGc) {
        self.element
            .set_bool_attribute(&local_name!("autofocus"), autofocus, can_gc);
    }
}
