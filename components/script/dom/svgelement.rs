/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use script_bindings::str::DOMString;
use stylo_dom::ElementState;

use crate::dom::bindings::codegen::Bindings::SVGElementBinding::SVGElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use crate::dom::document::Document;
use crate::dom::element::Element;
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
}

impl VirtualMethods for SVGElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<Element>() as &dyn VirtualMethods)
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

    // FIXME: The nonce should be stored in an internal slot instead of an
    // attribute (https://html.spec.whatwg.org/multipage/#cryptographicnonce)
    // https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce
    make_getter!(Nonce, "nonce");

    // https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce
    make_setter!(SetNonce, "nonce");

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
