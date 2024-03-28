/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::Cell;
use std::default::Default;

use cssparser::{Parser as CssParser, ParserInput};
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use servo_arc::Arc;
use servo_atoms::Atom;
use style::attr::AttrValue;
use style::media_queries::MediaList;
use style::parser::ParserContext as CssParserContext;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::{CssRuleType, Origin, Stylesheet, UrlExtraData};
use style_traits::ParsingMode;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenList_Binding::DOMTokenListMethods;
use crate::dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{
    cors_setting_for_element, reflect_cross_origin_attribute, reflect_referrer_policy_attribute,
    set_cross_origin_attribute, AttributeMutation, Element, ElementCreator,
};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{
    document_from_node, stylesheets_owner_from_node, window_from_node, BindContext, Node,
    UnbindContext,
};
use crate::dom::stylesheet::StyleSheet as DOMStyleSheet;
use crate::dom::virtualmethods::VirtualMethods;
use crate::stylesheet_loader::{StylesheetContextSource, StylesheetLoader, StylesheetOwner};

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub struct RequestGenerationId(u32);

impl RequestGenerationId {
    fn increment(self) -> RequestGenerationId {
        RequestGenerationId(self.0 + 1)
    }
}

#[dom_struct]
pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableDom<DOMTokenList>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    stylesheet: DomRefCell<Option<Arc<Stylesheet>>>,
    cssom_stylesheet: MutNullableDom<CSSStyleSheet>,

    /// <https://html.spec.whatwg.org/multipage/#a-style-sheet-that-is-blocking-scripts>
    parser_inserted: Cell<bool>,
    /// The number of loads that this link element has triggered (could be more
    /// than one because of imports) and have not yet finished.
    pending_loads: Cell<u32>,
    /// Whether any of the loads have failed.
    any_failed_load: Cell<bool>,
    /// A monotonically increasing counter that keeps track of which stylesheet to apply.
    request_generation_id: Cell<RequestGenerationId>,
}

impl HTMLLinkElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        creator: ElementCreator,
    ) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            rel_list: Default::default(),
            parser_inserted: Cell::new(creator.is_parser_created()),
            stylesheet: DomRefCell::new(None),
            cssom_stylesheet: MutNullableDom::new(None),
            pending_loads: Cell::new(0),
            any_failed_load: Cell::new(false),
            request_generation_id: Cell::new(RequestGenerationId(0)),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
    ) -> DomRoot<HTMLLinkElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLLinkElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
        )
    }

    pub fn get_request_generation_id(&self) -> RequestGenerationId {
        self.request_generation_id.get()
    }

    // FIXME(emilio): These methods are duplicated with
    // HTMLStyleElement::set_stylesheet.
    #[allow(crown::unrooted_must_root)]
    pub fn set_stylesheet(&self, s: Arc<Stylesheet>) {
        let stylesheets_owner = stylesheets_owner_from_node(self);
        if let Some(ref s) = *self.stylesheet.borrow() {
            stylesheets_owner.remove_stylesheet(self.upcast(), s)
        }
        *self.stylesheet.borrow_mut() = Some(s.clone());
        self.clean_stylesheet_ownership();
        stylesheets_owner.add_stylesheet(self.upcast(), s);
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(
                    &window_from_node(self),
                    self.upcast::<Element>(),
                    "text/css".into(),
                    None, // todo handle location
                    None, // todo handle title
                    sheet,
                )
            })
        })
    }

    pub fn is_alternate(&self) -> bool {
        let rel = get_attr(self.upcast(), &local_name!("rel"));
        match rel {
            Some(ref value) => value
                .split(HTML_SPACE_CHARACTERS)
                .any(|s| s.eq_ignore_ascii_case("alternate")),
            None => false,
        }
    }

    fn clean_stylesheet_ownership(&self) {
        if let Some(cssom_stylesheet) = self.cssom_stylesheet.get() {
            cssom_stylesheet.set_owner(None);
        }
        self.cssom_stylesheet.set(None);
    }
}

fn get_attr(element: &Element, local_name: &LocalName) -> Option<String> {
    let elem = element.get_attribute(&ns!(), local_name);
    elem.map(|e| {
        let value = e.value();
        (**value).to_owned()
    })
}

fn string_is_stylesheet(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => value
            .split(HTML_SPACE_CHARACTERS)
            .any(|s| s.eq_ignore_ascii_case("stylesheet")),
        None => false,
    }
}

/// Favicon spec usage in accordance with CEF implementation:
/// only url of icon is required/used
/// <https://html.spec.whatwg.org/multipage/#rel-icon>
fn is_favicon(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => value
            .split(HTML_SPACE_CHARACTERS)
            .any(|s| s.eq_ignore_ascii_case("icon") || s.eq_ignore_ascii_case("apple-touch-icon")),
        None => false,
    }
}

impl VirtualMethods for HTMLLinkElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if !self.upcast::<Node>().is_connected() || mutation.is_removal() {
            return;
        }

        let rel = get_attr(self.upcast(), &local_name!("rel"));
        match *attr.local_name() {
            local_name!("href") => {
                if string_is_stylesheet(&rel) {
                    self.handle_stylesheet_url(&attr.value());
                } else if is_favicon(&rel) {
                    let sizes = get_attr(self.upcast(), &local_name!("sizes"));
                    self.handle_favicon_url(rel.as_ref().unwrap(), &attr.value(), &sizes);
                }
            },
            local_name!("sizes") => {
                if is_favicon(&rel) {
                    if let Some(ref href) = get_attr(self.upcast(), &local_name!("href")) {
                        self.handle_favicon_url(
                            rel.as_ref().unwrap(),
                            href,
                            &Some(attr.value().to_string()),
                        );
                    }
                }
            },
            _ => {},
        }
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

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if context.tree_connected {
            let element = self.upcast();

            let rel = get_attr(element, &local_name!("rel"));
            let href = get_attr(element, &local_name!("href"));
            let sizes = get_attr(self.upcast(), &local_name!("sizes"));

            match href {
                Some(ref href) if string_is_stylesheet(&rel) => {
                    self.handle_stylesheet_url(href);
                },
                Some(ref href) if is_favicon(&rel) => {
                    self.handle_favicon_url(rel.as_ref().unwrap(), href, &sizes);
                },
                _ => {},
            }
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if let Some(s) = self.stylesheet.borrow_mut().take() {
            self.clean_stylesheet_ownership();
            stylesheets_owner_from_node(self).remove_stylesheet(self.upcast(), &s);
        }
    }
}

impl HTMLLinkElement {
    /// <https://html.spec.whatwg.org/multipage/#concept-link-obtain>
    fn handle_stylesheet_url(&self, href: &str) {
        let document = document_from_node(self);
        if document.browsing_context().is_none() {
            return;
        }

        // Step 1.
        if href.is_empty() {
            return;
        }

        // Step 2.
        let link_url = match document.base_url().join(href) {
            Ok(url) => url,
            Err(e) => {
                debug!("Parsing url {} failed: {}", href, e);
                return;
            },
        };

        let element = self.upcast::<Element>();

        // Step 3
        let cors_setting = cors_setting_for_element(element);

        let mq_attribute = element.get_attribute(&ns!(), &local_name!("media"));
        let value = mq_attribute.as_ref().map(|a| a.value());
        let mq_str = match value {
            Some(ref value) => &***value,
            None => "",
        };

        let mut input = ParserInput::new(mq_str);
        let mut css_parser = CssParser::new(&mut input);
        let document_url_data = &UrlExtraData(document.url().get_arc());
        let window = document.window();
        // FIXME(emilio): This looks somewhat fishy, since we use the context
        // only to parse the media query list, CssRuleType::Media doesn't make
        // much sense.
        let context = CssParserContext::new(
            Origin::Author,
            document_url_data,
            Some(CssRuleType::Media),
            ParsingMode::DEFAULT,
            document.quirks_mode(),
            /* namespaces = */ Default::default(),
            window.css_error_reporter(),
            None,
        );
        let media = MediaList::parse(&context, &mut css_parser);

        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.as_ref().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        self.request_generation_id
            .set(self.request_generation_id.get().increment());

        // TODO: #8085 - Don't load external stylesheets if the node's mq
        // doesn't match.
        let loader = StylesheetLoader::for_element(self.upcast());
        loader.load(
            StylesheetContextSource::LinkElement { media: Some(media) },
            link_url,
            cors_setting,
            integrity_metadata.to_owned(),
        );
    }

    fn handle_favicon_url(&self, _rel: &str, href: &str, _sizes: &Option<String>) {
        let document = document_from_node(self);
        match document.base_url().join(href) {
            Ok(url) => {
                let window = document.window();
                if window.is_top_level() {
                    let msg = EmbedderMsg::NewFavicon(url.clone());
                    window.send_to_embedder(msg);
                }
            },
            Err(e) => debug!("Parsing url {} failed: {}", href, e),
        }
    }
}

impl StylesheetOwner for HTMLLinkElement {
    fn increment_pending_loads_count(&self) {
        self.pending_loads.set(self.pending_loads.get() + 1)
    }

    fn load_finished(&self, succeeded: bool) -> Option<bool> {
        assert!(self.pending_loads.get() > 0, "What finished?");
        if !succeeded {
            self.any_failed_load.set(true);
        }

        self.pending_loads.set(self.pending_loads.get() - 1);
        if self.pending_loads.get() != 0 {
            return None;
        }

        let any_failed = self.any_failed_load.get();
        self.any_failed_load.set(false);
        Some(any_failed)
    }

    fn parser_inserted(&self) -> bool {
        self.parser_inserted.get()
    }

    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        if self.RelList().Contains("noreferrer".into()) {
            return Some(ReferrerPolicy::NoReferrer);
        }

        None
    }

    fn set_origin_clean(&self, origin_clean: bool) {
        if let Some(stylesheet) = self.get_cssom_stylesheet() {
            stylesheet.set_origin_clean(origin_clean);
        }
    }
}

impl HTMLLinkElementMethods for HTMLLinkElement {
    // https://html.spec.whatwg.org/multipage/#dom-link-href
    make_url_getter!(Href, "href");

    // https://html.spec.whatwg.org/multipage/#dom-link-href
    make_url_setter!(SetHref, "href");

    // https://html.spec.whatwg.org/multipage/#dom-link-rel
    make_getter!(Rel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-link-rel
    fn SetRel(&self, rel: DOMString) {
        self.upcast::<Element>()
            .set_tokenlist_attribute(&local_name!("rel"), rel);
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-media
    make_getter!(Media, "media");

    // https://html.spec.whatwg.org/multipage/#dom-link-media
    make_setter!(SetMedia, "media");

    // https://html.spec.whatwg.org/multipage/#dom-link-integrity
    make_getter!(Integrity, "integrity");

    // https://html.spec.whatwg.org/multipage/#dom-link-integrity
    make_setter!(SetIntegrity, "integrity");

    // https://html.spec.whatwg.org/multipage/#dom-link-hreflang
    make_getter!(Hreflang, "hreflang");

    // https://html.spec.whatwg.org/multipage/#dom-link-hreflang
    make_setter!(SetHreflang, "hreflang");

    // https://html.spec.whatwg.org/multipage/#dom-link-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-link-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-link-rellist
    fn RelList(&self) -> DomRoot<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(
                self.upcast(),
                &local_name!("rel"),
                Some(vec![
                    Atom::from("alternate"),
                    Atom::from("apple-touch-icon"),
                    Atom::from("apple-touch-icon-precomposed"),
                    Atom::from("canonical"),
                    Atom::from("dns-prefetch"),
                    Atom::from("icon"),
                    Atom::from("import"),
                    Atom::from("manifest"),
                    Atom::from("modulepreload"),
                    Atom::from("next"),
                    Atom::from("preconnect"),
                    Atom::from("prefetch"),
                    Atom::from("preload"),
                    Atom::from("prerender"),
                    Atom::from("stylesheet"),
                ]),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-charset
    make_getter!(Charset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-link-charset
    make_setter!(SetCharset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-link-rev
    make_getter!(Rev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-link-rev
    make_setter!(SetRev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-link-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#dom-link-target
    make_setter!(SetTarget, "target");

    // https://html.spec.whatwg.org/multipage/#dom-link-crossorigin
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-crossorigin
    fn SetCrossOrigin(&self, value: Option<DOMString>) {
        set_cross_origin_attribute(self.upcast::<Element>(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-referrerpolicy
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    // https://drafts.csswg.org/cssom/#dom-linkstyle-sheet
    fn GetSheet(&self) -> Option<DomRoot<DOMStyleSheet>> {
        self.get_cssom_stylesheet().map(DomRoot::upcast)
    }
}
