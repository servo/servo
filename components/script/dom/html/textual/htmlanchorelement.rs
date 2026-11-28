/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::context::{JSContext, NoGC};
use js::rust::HandleObject;
use layout_api::AccessibilityDamage;
use net_traits::blob_url_store::UrlWithBlobClaim;
use script_bindings::cell::DomRefCell;
use style::attr::AttrValue;
use stylo_atoms::Atom;
use stylo_dom::ElementState;

use crate::dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element, reflect_referrer_policy_attribute};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::links::htmlhyperlinkelementutils::{
    HyperlinkElement, HyperlinkElementTraits,
};
use crate::dom::html::links::relations::LinkRelations;
use crate::dom::node::Node;
use crate::dom::node::virtualmethods::VirtualMethods;

#[dom_struct]
pub(crate) struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableDom<DOMTokenList>,
    #[no_trace]
    relations: Cell<LinkRelations>,
    #[no_trace]
    url: DomRefCell<Option<UrlWithBlobClaim>>,
}

impl HTMLAnchorElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
            relations: Cell::new(LinkRelations::empty()),
            url: DomRefCell::new(None),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLAnchorElement> {
        Node::reflect_weak_referenceable_node_with_proto(
            cx,
            Rc::new(HTMLAnchorElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HyperlinkElement for HTMLAnchorElement {
    fn get_url(&self) -> &DomRefCell<Option<UrlWithBlobClaim>> {
        &self.url
    }

    fn relations(&self) -> LinkRelations {
        self.relations.get()
    }
}

impl VirtualMethods for HTMLAnchorElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);

        self.attribute_mutated_for_hyperlinks(cx.no_gc(), attr, mutation);

        // https://html.spec.whatwg.org/multipage/#introduction-2
        // > Similarly, for a and area elements with an href attribute and a rel attribute,
        // > links must be created for the keywords of the rel attribute
        // > as defined for those keywords in the link types section.
        match *attr.local_name() {
            local_name!("href") => {
                self.upcast::<Element>().set_state(
                    cx.no_gc(),
                    ElementState::UNVISITED,
                    !mutation.is_removal(),
                );

                // Adding or removing the `href` attribute can cause the node's accessibility role
                // to change, and we also need to ensure the URL property is accurate.
                self.upcast::<Node>()
                    .add_pending_accessibility_damage(cx.no_gc(), AccessibilityDamage::Node);
            },
            local_name!("rel") | local_name!("rev") => {
                self.relations
                    .set(LinkRelations::for_element(self.upcast()));
            },
            _ => {},
        }
    }
}

impl HTMLAnchorElementMethods<crate::DomTypeHolder> for HTMLAnchorElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-a-text>
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().GetTextContent().unwrap()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-a-text>
    fn SetText(&self, cx: &mut JSContext, value: DOMString) {
        self.upcast::<Node>()
            .set_text_content_for_element(cx, Some(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    make_getter!(Rel, "rel");

    /// <https://html.spec.whatwg.org/multipage/#dom-a-rel>
    fn SetRel(&self, cx: &mut JSContext, rel: DOMString) {
        self.upcast::<Element>()
            .set_tokenlist_attribute(cx, &local_name!("rel"), rel);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-a-rellist>
    fn RelList(&self, cx: &mut JSContext) -> DomRoot<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(
                cx,
                self.upcast(),
                &local_name!("rel"),
                Some(vec![
                    Atom::from("noopener"),
                    Atom::from("noreferrer"),
                    Atom::from("opener"),
                ]),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-hreflang
    make_getter!(Hreflang, "hreflang");

    // https://html.spec.whatwg.org/multipage/#dom-a-hreflang
    make_setter!(SetHreflang, "hreflang");

    // https://html.spec.whatwg.org/multipage/#dom-a-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-a-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_getter!(Coords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_setter!(SetCoords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-charset
    make_getter!(Charset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-a-charset
    make_setter!(SetCharset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-rev
    make_getter!(Rev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-a-rev
    make_setter!(SetRev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-a-shape
    make_getter!(Shape, "shape");

    // https://html.spec.whatwg.org/multipage/#dom-a-shape
    make_setter!(SetShape, "shape");

    // https://html.spec.whatwg.org/multipage/#attr-hyperlink-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#attr-hyperlink-target
    make_setter!(SetTarget, "target");

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-href>
    fn Href(&self, no_gc: &NoGC) -> USVString {
        self.get_href(no_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    make_url_setter!(SetHref, "href");

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-origin>
    fn Origin(&self, no_gc: &NoGC) -> USVString {
        self.get_origin(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol>
    fn Protocol(&self, no_gc: &NoGC) -> USVString {
        self.get_protocol(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol>
    fn SetProtocol(&self, cx: &mut JSContext, value: USVString) {
        self.set_protocol(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-password>
    fn Password(&self, no_gc: &NoGC) -> USVString {
        self.get_password(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-password>
    fn SetPassword(&self, cx: &mut JSContext, value: USVString) {
        self.set_password(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash>
    fn Hash(&self, no_gc: &NoGC) -> USVString {
        self.get_hash(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash>
    fn SetHash(&self, cx: &mut JSContext, value: USVString) {
        self.set_hash(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-host>
    fn Host(&self, no_gc: &NoGC) -> USVString {
        self.get_host(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-host>
    fn SetHost(&self, cx: &mut JSContext, value: USVString) {
        self.set_host(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname>
    fn Hostname(&self, no_gc: &NoGC) -> USVString {
        self.get_hostname(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname>
    fn SetHostname(&self, cx: &mut JSContext, value: USVString) {
        self.set_hostname(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-port>
    fn Port(&self, no_gc: &NoGC) -> USVString {
        self.get_port(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-port>
    fn SetPort(&self, cx: &mut JSContext, value: USVString) {
        self.set_port(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname>
    fn Pathname(&self, no_gc: &NoGC) -> USVString {
        self.get_pathname(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname>
    fn SetPathname(&self, cx: &mut JSContext, value: USVString) {
        self.set_pathname(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-search>
    fn Search(&self, no_gc: &NoGC) -> USVString {
        self.get_search(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-search>
    fn SetSearch(&self, cx: &mut JSContext, value: USVString) {
        self.set_search(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-username>
    fn Username(&self, no_gc: &NoGC) -> USVString {
        self.get_username(no_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hyperlink-username>
    fn SetUsername(&self, cx: &mut JSContext, value: USVString) {
        self.set_username(cx, value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-a-referrerpolicy>
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");
}
