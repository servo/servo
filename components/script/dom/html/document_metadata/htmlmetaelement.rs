/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use content_security_policy::{Policy, PolicyDisposition, PolicySource};
use dom_struct::dom_struct;
use embedder_traits::Theme;
use html5ever::{LocalName, Prefix, local_name};
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use paint_api::viewport_description::ViewportDescription;
use script_bindings::dom::UnrootedDom;
use servo_config::pref;
use style::str::HTML_SPACE_CHARACTERS;

use crate::dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlheadelement::HTMLHeadElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};

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
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLMetaElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLMetaElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }

    fn process_attributes(&self, cx: &mut JSContext) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);
            if name.eq_ignore_ascii_case("referrer") {
                self.apply_referrer();
            }
            if name.eq_ignore_ascii_case("viewport") {
                self.parse_and_send_viewport_if_necessary(cx);
            }
        // https://html.spec.whatwg.org/multipage/#attr-meta-http-equiv
        } else if !self.HttpEquiv().is_empty() {
            // TODO: Implement additional http-equiv candidates
            if self.HttpEquiv().eq_ignore_ascii_case("refresh") {
                self.declarative_refresh();
            } else if self
                .HttpEquiv()
                .eq_ignore_ascii_case("content-security-policy")
            {
                self.apply_csp_list();
            }
        }
    }

    fn process_referrer_attribute(&self) {
        let element = self.upcast::<Element>();
        if let Some(ref name) = element.get_name() {
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name.eq_ignore_ascii_case("referrer") {
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
            .get_attribute_string_value(&local_name!("content"))
            .filter(|value| !value.is_empty())
        {
            // Step 4. Let value be the value of element's content attribute, converted to ASCII
            // lowercase.
            // Step 5. If value is one of the values given in the first column of the following
            // table, then set value to the value given in the second column:
            // Step 6. If value is a referrer policy, then set element's node document's policy
            // container's referrer policy to policy.
            doc.set_referrer_policy(ReferrerPolicy::from_with_legacy(&content));
        }
    }

    /// <https://drafts.csswg.org/css-viewport/#parsing-algorithm>
    fn parse_and_send_viewport_if_necessary(&self, cx: &mut JSContext) {
        if !pref!(viewport_meta_enabled) {
            return;
        }

        // Skip processing if this isn't the top level frame
        if !self.owner_window().is_top_level() {
            return;
        }
        let element = self.upcast::<Element>();
        let Some(content) = element.get_attribute_string_value(&local_name!("content")) else {
            return;
        };

        if let Ok(viewport) = ViewportDescription::from_str(&content) {
            let initial_scale = viewport.initial_scale.get();
            let window = self.owner_window();
            window.paint_api().viewport(window.webview_id(), viewport);
            window
                .get_or_init_visual_viewport(cx)
                .update_scale(initial_scale);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-color-scheme>
    fn obtain_page_supported_color_schemes(&self, cx: &mut JSContext) {
        let doc = self.owner_document();
        // Step 1. Let candidate elements be the list of all meta elements
        // that meet the following criteria, in tree order:
        let new_theme = doc
            .upcast::<Node>()
            // Do not traverse shadow trees for optimization, which also implies:
            // > the element is in a document tree;
            .traverse_preorder_non_rooting(cx.no_gc(), ShadowIncluding::No)
            .filter_map(UnrootedDom::downcast::<HTMLMetaElement>)
            .filter_map(|meta| {
                let element = UnrootedDom::upcast::<Element>(meta);

                // > the element has a content attribute.
                element
                    .get_attribute_string_value(&local_name!("content"))
                    .filter(|_| {
                        // > the element has a name attribute,
                        // > whose value is an ASCII case-insensitive match for color-scheme; and
                        element.get_name().is_color_scheme()
                    })
            })
            // Step 2. For each element in candidate elements:
            .find_map(|content| {
                // Step 2.1. Let parsed be the result of parsing a list of
                // component values given the value of element's content attribute.
                // Step 2.2. If parsed is a valid CSS 'color-scheme' property value,
                // then return parsed.
                // TODO: Allow for more different themes than the ones that embedders can set
                if content.eq_ignore_ascii_case("dark") {
                    Some(Theme::Dark)
                } else if content.eq_ignore_ascii_case("light") {
                    Some(Theme::Light)
                } else {
                    // Step 3. Return null.
                    None
                }
            });

        doc.set_theme(new_theme);
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
            .get_attribute_string_value(&local_name!("content"))
        else {
            return;
        };
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
            self.owner_document().shared_declarative_refresh_steps(
                &content.as_bytes(),
                /* from_meta_element */ true,
            );
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
            self.process_attributes(cx);

            // Optimization: only if this meta element has a color scheme we should update.
            // Otherwise we would traverse the whole DOM for any meta element, which are
            // commonly used for information for crawlers.
            if self.upcast::<Element>().get_name().is_color_scheme() {
                // https://html.spec.whatwg.org/multipage/#meta-color-scheme
                // > If any meta elements are inserted into the document or removed from the document,
                // > or existing meta elements have their name or content attributes changed,
                // > user agents must re-run the above algorithm.
                //
                // When the element is inserted
                self.obtain_page_supported_color_schemes(cx);
            }
        }
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(cx, attr, mutation);
        }

        self.process_referrer_attribute();

        // Optimization: only if this meta element either did or does now specify a color-scheme.
        // Or if the content of a meta element is changed that specifies a color-scheme
        // Otherwise we would traverse the whole DOM for any meta element, which are
        // commonly used for information for crawlers.
        let affects_color_scheme = if *attr.local_name() == local_name!("name") {
            mutation.old_value(attr).is_color_scheme() || mutation.new_value(attr).is_color_scheme()
        } else {
            self.upcast::<Element>().get_name().is_color_scheme() &&
                *attr.local_name() == local_name!("content")
        };

        if affects_color_scheme {
            // https://html.spec.whatwg.org/multipage/#meta-color-scheme
            // > If any meta elements are inserted into the document or removed from the document,
            // > or existing meta elements have their name or content attributes changed,
            // > user agents must re-run the above algorithm.
            //
            // When the content attribute has changed
            self.obtain_page_supported_color_schemes(cx);
        }
    }

    fn unbind_from_tree(&self, cx: &mut js::context::JSContext, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(cx, context);
        }

        if context.tree_connected {
            self.process_referrer_attribute();

            // Optimization: only if this meta element has a color scheme we should update.
            // Otherwise we would traverse the whole DOM for any meta element, which are
            // commonly used for information for crawlers.
            if self.upcast::<Element>().get_name().is_color_scheme() {
                // https://html.spec.whatwg.org/multipage/#meta-color-scheme
                // > If any meta elements are inserted into the document or removed from the document,
                // > or existing meta elements have their name or content attributes changed,
                // > user agents must re-run the above algorithm.
                //
                // When the element is removed
                self.obtain_page_supported_color_schemes(cx);
            }
        }
    }
}

/// Trait to make it easier to make sure all callers lowercase to ASCII
/// before comparing to the `color-scheme` value.
/// Otherwise it is easy to miss one usage and compare case-sensitively.
trait IsColorSchemeValue {
    fn is_color_scheme(&self) -> bool;
}

impl<T: AsRef<str>> IsColorSchemeValue for Option<T> {
    fn is_color_scheme(&self) -> bool {
        self.as_ref()
            .is_some_and(|name| name.as_ref().eq_ignore_ascii_case("color-scheme"))
    }
}
