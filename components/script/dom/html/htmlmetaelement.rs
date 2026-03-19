/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use content_security_policy::{Policy, PolicyDisposition, PolicySource};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use paint_api::viewport_description::ViewportDescription;
use servo_config::pref;
use style::str::HTML_SPACE_CHARACTERS;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlheadelement::HTMLHeadElement;
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLMetaElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLMetaElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    fn process_attributes(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);
            if name == "referrer" {
                self.apply_referrer();
            }
            if name == "viewport" {
                self.parse_and_send_viewport_if_necessary();
            }
        // https://html.spec.whatwg.org/multipage/#attr-meta-http-equiv
        } else if !self.HttpEquiv().is_empty() {
            // TODO: Implement additional http-equiv candidates
            match self.HttpEquiv().to_ascii_lowercase().as_str() {
                "refresh" => self.declarative_refresh(),
                "content-security-policy" => self.apply_csp_list(),
                _ => {},
            }
        }
    }

    fn process_referrer_attribute(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    fn apply_referrer(&self) {
        let doc = self.owner_document();
        // From spec: For historical reasons, unlike other standard metadata names, the processing model for referrer
        // is not responsive to element removals, and does not use tree order. Only the most-recently-inserted or
        // most-recently-modified meta element in this state has an effect.
        // Step 1. If element is not in a document tree, then return.
        let meta_node = self.upcast::<Node>();
        if !meta_node.is_in_a_document_tree() {
            return;
        }

        // Step 2. If element does not have a name attribute whose value is an ASCII
        // case-insensitive match for "referrer", then return.
        if self.upcast::<Element>().get_name() != Some(atom!("referrer")) {
            return;
        }

        // Step 3. If element does not have a content attribute, or that attribute's value is the
        // empty string, then return.
        if let Some(content) = self
            .upcast::<Element>()
            .get_attribute(&local_name!("content"))
            .filter(|attr| !attr.value().is_empty())
        {
            // Step 4. Let value be the value of element's content attribute, converted to ASCII
            // lowercase.
            // Step 5. If value is one of the values given in the first column of the following
            // table, then set value to the value given in the second column:
            // Step 6. If value is a referrer policy, then set element's node document's policy
            // container's referrer policy to policy.
            doc.set_referrer_policy(ReferrerPolicy::from_with_legacy(&content.value()));
        }
    }

    /// <https://drafts.csswg.org/css-viewport/#parsing-algorithm>
    fn parse_and_send_viewport_if_necessary(&self) {
        if !pref!(viewport_meta_enabled) {
            return;
        }

        // Skip processing if this isn't the top level frame
        if !self.owner_window().is_top_level() {
            return;
        }
        let element = self.upcast::<Element>();
        let Some(content) = element.get_attribute(&local_name!("content")) else {
            return;
        };

        if let Ok(viewport) = ViewportDescription::from_str(&content.value()) {
            self.owner_window()
                .paint_api()
                .viewport(self.owner_window().webview_id(), viewport);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-meta-http-equiv-content-security-policy>
    fn apply_csp_list(&self) {
        // Step 1. If the meta element is not a child of a head element, return.
        if self
            .upcast::<Node>()
            .GetParentElement()
            .is_none_or(|parent| !parent.is::<HTMLHeadElement>())
        {
            return;
        };
        // Step 2. If the meta element has no content attribute, or if that attribute's value is the empty string, then return.
        let Some(content) = self
            .upcast::<Element>()
            .get_attribute(&local_name!("content"))
        else {
            return;
        };
        let content = content.value();
        if content.is_empty() {
            return;
        }
        // Step 3. Let policy be the result of executing Content Security Policy's
        // parse a serialized Content Security Policy algorithm
        // on the meta element's content attribute's value,
        // with a source of "meta", and a disposition of "enforce".
        let mut policy = Policy::parse(&content, PolicySource::Meta, PolicyDisposition::Enforce);
        // Step 4. Remove all occurrences of the report-uri, frame-ancestors,
        // and sandbox directives from policy.
        policy.directive_set.retain(|directive| {
            !matches!(
                directive.name.as_str(),
                "report-uri" | "frame-ancestors" | "sandbox"
            )
        });
        // Step 5. Enforce the policy policy.
        self.owner_document().enforce_csp_policy(policy);
    }

    /// <https://html.spec.whatwg.org/multipage/#shared-declarative-refresh-steps>
    fn declarative_refresh(&self) {
        if !self.upcast::<Node>().is_in_a_document_tree() {
            return;
        }

        // Step 2. Let input be the value of the element's content attribute.
        let content = self.Content();
        // Step 1. If the meta element has no content attribute, or if that attribute's value is the empty string, then return.
        if !content.is_empty() {
            // Step 3. Run the shared declarative refresh steps with the meta element's node document, input, and the meta element.
            self.owner_document()
                .shared_declarative_refresh_steps(&content.as_bytes());
        }
    }
}

impl HTMLMetaElementMethods<crate::DomTypeHolder> for HTMLMetaElement {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-httpequiv
    make_getter!(HttpEquiv, "http-equiv");
    // https://html.spec.whatwg.org/multipage/#dom-meta-httpequiv
    make_atomic_setter!(SetHttpEquiv, "http-equiv");

    // https://html.spec.whatwg.org/multipage/#dom-meta-scheme
    make_getter!(Scheme, "scheme");
    // https://html.spec.whatwg.org/multipage/#dom-meta-scheme
    make_setter!(SetScheme, "scheme");
}

impl VirtualMethods for HTMLMetaElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
        }

        if context.tree_connected {
            self.process_attributes();
        }
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(cx, attr, mutation);
        }

        self.process_referrer_attribute();
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context, can_gc);
        }

        if context.tree_connected {
            self.process_referrer_attribute();
        }
    }
}
