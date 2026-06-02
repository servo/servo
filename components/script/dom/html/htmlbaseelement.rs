/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use servo_url::ServoUrl;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLBaseElementBinding::HTMLBaseElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::security::csp::CspReporting;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLBaseElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#frozen-base-url>
    #[no_trace]
    frozen_base_url: DomRefCell<Option<ServoUrl>>,
}

impl HTMLBaseElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLBaseElement {
        HTMLBaseElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            frozen_base_url: Default::default(),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLBaseElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLBaseElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn clear_frozen_base_url(&self) {
        *self.frozen_base_url.borrow_mut() = None;
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-frozen-base-url>
    pub(crate) fn set_frozen_base_url(&self) {
        // Step 1. Let document be element's node document.
        let document = self.owner_document();
        // Step 2. Let urlRecord be the result of parsing the value of element's href content attribute
        // with document's fallback base URL, and document's character encoding. (Thus, the base element isn't affected by itself.)
        let attr = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("href"));
        let Some(href_value) = attr.as_ref().map(|attr| attr.value()) else {
            unreachable!("Must always have a href set when setting frozen base URL");
        };
        let document_fallback_url = document.fallback_base_url();
        let url_record = document_fallback_url.join(&href_value).ok();
        // Step 3. If any of the following are true:
        if
        // urlRecord is failure;
        url_record.as_ref().is_none_or(|url_record|
            // urlRecord's scheme is "data" or "javascript"; or
            url_record.scheme() == "data" || url_record.scheme() == "javascript"
            // running Is base allowed for Document? on urlRecord and document returns "Blocked",
            || !document
                .get_csp_list()
                .is_base_allowed_for_document(
                    document.window().upcast::<GlobalScope>(),
                    &url_record.clone().into_url(),
                    &document.origin().immutable().clone().into_url_origin(),
                ))
        {
            // then set element's frozen base URL to document's fallback base URL and return.
            *self.frozen_base_url.borrow_mut() = Some(document_fallback_url);
            return;
        }
        // Step 4. Set element's frozen base URL to urlRecord.
        *self.frozen_base_url.borrow_mut() = url_record;
        // Step 5. Respond to base URL changes given document.
        // TODO
    }

    /// <https://html.spec.whatwg.org/multipage/#frozen-base-url>
    pub(crate) fn frozen_base_url(&self) -> ServoUrl {
        self.frozen_base_url
            .borrow()
            .clone()
            .expect("Must only retrieve frozen base URL for valid base elements")
    }
}

impl HTMLBaseElementMethods<crate::DomTypeHolder> for HTMLBaseElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-base-href>
    fn Href(&self) -> DOMString {
        // Step 1. Let document be element's node document.
        let document = self.owner_document();

        // Step 2. Let url be the value of the href attribute of this element, if it has one, and the empty string otherwise.
        let attr = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("href"));
        let value = attr.as_ref().map(|attr| attr.value());
        let url = value.as_ref().map_or("", |value| &**value);

        // Step 3. Let urlRecord be the result of parsing url with document's fallback base URL,
        // and document's character encoding. (Thus, the base element isn't affected by other base elements or itself.)
        let url_record = document.fallback_base_url().join(url);

        match url_record {
            Err(_) => {
                // Step 4. If urlRecord is failure, return url.
                url.into()
            },
            Ok(url_record) => {
                // Step 5. Return the serialization of urlRecord.
                url_record.into_string().into()
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-base-href
    make_setter!(SetHref, "href");
}

impl VirtualMethods for HTMLBaseElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        // https://html.spec.whatwg.org/multipage/#frozen-base-url
        if *attr.local_name() == local_name!("href") {
            // > The base element is the first base element in tree order with an href content attribute in its Document,
            // > and its href content attribute is changed.
            if self.frozen_base_url.borrow().is_some() && !mutation.is_removal() {
                self.set_frozen_base_url();
            } else {
                // > The base element becomes the first base element in tree order with an href content attribute in its Document.
                let document = self.owner_document();
                document.refresh_base_element();
            }
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);
        // https://html.spec.whatwg.org/multipage/#frozen-base-url
        // > The base element becomes the first base element in tree order with an href content attribute in its Document.
        let document = self.owner_document();
        document.refresh_base_element();
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);
        // https://html.spec.whatwg.org/multipage/#frozen-base-url
        // > The base element becomes the first base element in tree order with an href content attribute in its Document.
        let document = self.owner_document();
        document.refresh_base_element();
    }
}
