/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::{Borrow, ToOwned};
use std::cell::Cell;
use std::default::Default;
use std::str::FromStr;

use base::id::WebViewId;
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use mime::Mime;
use net_traits::mime_classifier::{MediaType, MimeClassifier};
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CorsSettings, Destination, Initiator, InsecureRequestsPolicy, Referrer, RequestBuilder,
    RequestId,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, NetworkError, ReferrerPolicy, ResourceFetchTiming,
    ResourceTimingType,
};
use script_bindings::root::Dom;
use servo_arc::Arc;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::attr::AttrValue;
use style::stylesheets::Stylesheet;
use stylo_atoms::Atom;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenList_Binding::DOMTokenListMethods;
use crate::dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::document::Document;
use crate::dom::documentorshadowroot::StylesheetSource;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{
    AttributeMutation, Element, ElementCreator, cors_setting_for_element,
    referrer_policy_for_element, reflect_cross_origin_attribute, reflect_referrer_policy_attribute,
    set_cross_origin_attribute,
};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::medialist::MediaList;
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::stylesheet::StyleSheet as DOMStyleSheet;
use crate::dom::types::{EventTarget, GlobalScope};
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::create_a_potential_cors_request;
use crate::links::LinkRelations;
use crate::network_listener::{PreInvoke, ResourceTimingListener, submit_timing};
use crate::script_runtime::CanGc;
use crate::stylesheet_loader::{StylesheetContextSource, StylesheetLoader, StylesheetOwner};

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct RequestGenerationId(u32);

impl RequestGenerationId {
    fn increment(self) -> RequestGenerationId {
        RequestGenerationId(self.0 + 1)
    }
}

/// <https://html.spec.whatwg.org/multipage/#link-processing-options>
struct LinkProcessingOptions {
    href: String,
    destination: Option<Destination>,
    integrity: String,
    link_type: String,
    cryptographic_nonce_metadata: String,
    cross_origin: Option<CorsSettings>,
    referrer_policy: ReferrerPolicy,
    policy_container: PolicyContainer,
    source_set: Option<()>,
    base_url: ServoUrl,
    origin: ImmutableOrigin,
    insecure_requests_policy: InsecureRequestsPolicy,
    has_trustworthy_ancestor_origin: bool,
    // Some fields that we don't need yet are missing
}

#[dom_struct]
pub(crate) struct HTMLLinkElement {
    htmlelement: HTMLElement,
    /// The relations as specified by the "rel" attribute
    rel_list: MutNullableDom<DOMTokenList>,

    /// The link relations as they are used in practice.
    ///
    /// The reason this is seperate from [HTMLLinkElement::rel_list] is that
    /// a literal list is a bit unwieldy and that there are corner cases to consider
    /// (Like `rev="made"` implying an author relationship that is not represented in rel_list)
    #[no_trace]
    relations: Cell<LinkRelations>,

    #[conditional_malloc_size_of]
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
    /// <https://html.spec.whatwg.org/multipage/#explicitly-enabled>
    is_explicitly_enabled: Cell<bool>,
    /// Whether the previous type matched with the destination
    previous_type_matched: Cell<bool>,
    /// Whether the previous media environment matched with the media query
    previous_media_environment_matched: Cell<bool>,
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
            relations: Cell::new(LinkRelations::empty()),
            parser_inserted: Cell::new(creator.is_parser_created()),
            stylesheet: DomRefCell::new(None),
            cssom_stylesheet: MutNullableDom::new(None),
            pending_loads: Cell::new(0),
            any_failed_load: Cell::new(false),
            request_generation_id: Cell::new(RequestGenerationId(0)),
            is_explicitly_enabled: Cell::new(false),
            previous_type_matched: Cell::new(true),
            previous_media_environment_matched: Cell::new(true),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
        can_gc: CanGc,
    ) -> DomRoot<HTMLLinkElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLLinkElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_request_generation_id(&self) -> RequestGenerationId {
        self.request_generation_id.get()
    }

    // FIXME(emilio): These methods are duplicated with
    // HTMLStyleElement::set_stylesheet.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_stylesheet(&self, s: Arc<Stylesheet>) {
        let stylesheets_owner = self.stylesheet_list_owner();
        if let Some(ref s) = *self.stylesheet.borrow() {
            stylesheets_owner
                .remove_stylesheet(StylesheetSource::Element(Dom::from_ref(self.upcast())), s)
        }
        *self.stylesheet.borrow_mut() = Some(s.clone());
        self.clean_stylesheet_ownership();
        stylesheets_owner
            .add_stylesheet(StylesheetSource::Element(Dom::from_ref(self.upcast())), s);
    }

    pub(crate) fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub(crate) fn get_cssom_stylesheet(&self, can_gc: CanGc) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(
                    &self.owner_window(),
                    Some(self.upcast::<Element>()),
                    "text/css".into(),
                    None, // todo handle location
                    None, // todo handle title
                    sheet,
                    false, // is_constructed
                    can_gc,
                )
            })
        })
    }

    pub(crate) fn is_alternate(&self) -> bool {
        self.relations.get().contains(LinkRelations::ALTERNATE)
    }

    pub(crate) fn is_effectively_disabled(&self) -> bool {
        (self.is_alternate() && !self.is_explicitly_enabled.get()) ||
            self.upcast::<Element>()
                .has_attribute(&local_name!("disabled"))
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

impl VirtualMethods for HTMLLinkElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        let local_name = attr.local_name();
        let is_removal = mutation.is_removal();
        if *local_name == local_name!("disabled") {
            self.handle_disabled_attribute_change(!is_removal);
            return;
        }

        if !self.upcast::<Node>().is_connected() {
            return;
        }
        match *local_name {
            local_name!("rel") | local_name!("rev") => {
                self.relations
                    .set(LinkRelations::for_element(self.upcast()));
            },
            local_name!("href") => {
                if is_removal {
                    return;
                }
                // https://html.spec.whatwg.org/multipage/#link-type-stylesheet
                // When the href attribute of the link element of an external resource link
                // that is already browsing-context connected is changed.
                if self.relations.get().contains(LinkRelations::STYLESHEET) {
                    self.handle_stylesheet_url(&attr.value());
                }

                if self.relations.get().contains(LinkRelations::ICON) {
                    let sizes = get_attr(self.upcast(), &local_name!("sizes"));
                    self.handle_favicon_url(&attr.value(), &sizes);
                }

                // https://html.spec.whatwg.org/multipage/#link-type-prefetch
                // When the href attribute of the link element of an external resource link
                // that is already browsing-context connected is changed.
                if self.relations.get().contains(LinkRelations::PREFETCH) {
                    self.fetch_and_process_prefetch_link(&attr.value());
                }

                // https://html.spec.whatwg.org/multipage/#link-type-preload
                // When the href attribute of the link element of an external resource link
                // that is already browsing-context connected is changed.
                if self.relations.get().contains(LinkRelations::PRELOAD) {
                    self.handle_preload_url();
                }
            },
            local_name!("sizes") if self.relations.get().contains(LinkRelations::ICON) => {
                if let Some(ref href) = get_attr(self.upcast(), &local_name!("href")) {
                    self.handle_favicon_url(href, &Some(attr.value().to_string()));
                }
            },
            local_name!("crossorigin") => {
                // https://html.spec.whatwg.org/multipage/#link-type-prefetch
                // When the crossorigin attribute of the link element of an external resource link
                // that is already browsing-context connected is set, changed, or removed.
                if self.relations.get().contains(LinkRelations::PREFETCH) {
                    self.fetch_and_process_prefetch_link(&attr.value());
                }

                // https://html.spec.whatwg.org/multipage/#link-type-stylesheet
                // When the crossorigin attribute of the link element of an external resource link
                // that is already browsing-context connected is set, changed, or removed.
                if self.relations.get().contains(LinkRelations::STYLESHEET) {
                    self.handle_stylesheet_url(&attr.value());
                }
            },
            local_name!("as") => {
                // https://html.spec.whatwg.org/multipage/#link-type-preload
                // When the as attribute of the link element of an external resource link
                // that is already browsing-context connected is changed.
                if self.relations.get().contains(LinkRelations::PRELOAD) {
                    if let AttributeMutation::Set(Some(_)) = mutation {
                        self.handle_preload_url();
                    }
                }
            },
            local_name!("type") => {
                // https://html.spec.whatwg.org/multipage/#link-type-stylesheet
                // When the type attribute of the link element of an external resource link that
                // is already browsing-context connected is set or changed to a value that does
                // not or no longer matches the Content-Type metadata of the previous obtained
                // external resource, if any.
                //
                // TODO: Match Content-Type metadata to check if it needs to be updated
                if self.relations.get().contains(LinkRelations::STYLESHEET) {
                    self.handle_stylesheet_url(&attr.value());
                }

                // https://html.spec.whatwg.org/multipage/#link-type-preload
                // When the type attribute of the link element of an external resource link that
                // is already browsing-context connected, but was previously not obtained due to
                // the type attribute specifying an unsupported type for the request destination,
                // is set, removed, or changed.
                if self.relations.get().contains(LinkRelations::PRELOAD) &&
                    !self.previous_type_matched.get()
                {
                    self.handle_preload_url();
                }
            },
            local_name!("media") => {
                // https://html.spec.whatwg.org/multipage/#link-type-preload
                // When the media attribute of the link element of an external resource link that
                // is already browsing-context connected, but was previously not obtained due to
                // the media attribute not matching the environment, is changed or removed.
                if self.relations.get().contains(LinkRelations::PRELOAD) &&
                    !self.previous_media_environment_matched.get()
                {
                    match mutation {
                        AttributeMutation::Removed | AttributeMutation::Set(Some(_)) => {
                            self.handle_preload_url()
                        },
                        _ => {},
                    };
                }

                let matches_media_environment =
                    self.upcast::<Element>().matches_environment(&attr.value());
                self.previous_media_environment_matched
                    .set(matches_media_environment);
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

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        self.relations
            .set(LinkRelations::for_element(self.upcast()));

        if context.tree_connected {
            let element = self.upcast();

            if let Some(href) = get_attr(element, &local_name!("href")) {
                let relations = self.relations.get();
                if relations.contains(LinkRelations::STYLESHEET) {
                    self.handle_stylesheet_url(&href);
                }

                if relations.contains(LinkRelations::ICON) {
                    let sizes = get_attr(self.upcast(), &local_name!("sizes"));
                    self.handle_favicon_url(&href, &sizes);
                }

                if relations.contains(LinkRelations::PREFETCH) {
                    self.fetch_and_process_prefetch_link(&href);
                }

                if relations.contains(LinkRelations::PRELOAD) {
                    self.handle_preload_url();
                }
            }
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context, can_gc);
        }

        if let Some(s) = self.stylesheet.borrow_mut().take() {
            self.clean_stylesheet_ownership();
            self.stylesheet_list_owner()
                .remove_stylesheet(StylesheetSource::Element(Dom::from_ref(self.upcast())), &s);
        }
    }
}

impl HTMLLinkElement {
    fn compute_destination_for_attribute(&self) -> Destination {
        let element = self.upcast::<Element>();
        element
            .get_attribute(&ns!(), &local_name!("as"))
            .map(|attr| translate_a_preload_destination(&attr.value()))
            .unwrap_or(Destination::None)
    }

    /// <https://html.spec.whatwg.org/multipage/#create-link-options-from-element>
    fn processing_options(&self) -> LinkProcessingOptions {
        let element = self.upcast::<Element>();

        // Step 1. Let document be el's node document.
        let document = self.upcast::<Node>().owner_doc();

        // Step 2. Let options be a new link processing options
        let destination = self.compute_destination_for_attribute();

        let mut options = LinkProcessingOptions {
            href: String::new(),
            destination: Some(destination),
            integrity: String::new(),
            link_type: String::new(),
            cryptographic_nonce_metadata: self.upcast::<Element>().nonce_value(),
            cross_origin: cors_setting_for_element(element),
            referrer_policy: referrer_policy_for_element(element),
            policy_container: document.policy_container().to_owned(),
            source_set: None, // FIXME
            origin: document.borrow().origin().immutable().to_owned(),
            base_url: document.borrow().base_url(),
            insecure_requests_policy: document.insecure_requests_policy(),
            has_trustworthy_ancestor_origin: document.has_trustworthy_ancestor_or_current_origin(),
        };

        // Step 3. If el has an href attribute, then set options's href to the value of el's href attribute.
        if let Some(href_attribute) = element.get_attribute(&ns!(), &local_name!("href")) {
            options.href = (**href_attribute.value()).to_owned();
        }

        // Step 4. If el has an integrity attribute, then set options's integrity
        //         to the value of el's integrity content attribute.
        if let Some(integrity_attribute) = element.get_attribute(&ns!(), &local_name!("integrity"))
        {
            options.integrity = (**integrity_attribute.value()).to_owned();
        }

        // Step 5. If el has a type attribute, then set options's type to the value of el's type attribute.
        if let Some(type_attribute) = element.get_attribute(&ns!(), &local_name!("type")) {
            options.link_type = (**type_attribute.value()).to_owned();
        }

        // Step 6. Assert: options's href is not the empty string, or options's source set is not null.
        assert!(!options.href.is_empty() || options.source_set.is_some());

        // Step 7. Return options.
        options
    }

    /// The `fetch and process the linked resource` algorithm for [`rel="prefetch"`](https://html.spec.whatwg.org/multipage/#link-type-prefetch)
    fn fetch_and_process_prefetch_link(&self, href: &str) {
        // Step 1. If el's href attribute's value is the empty string, then return.
        if href.is_empty() {
            return;
        }

        // Step 2. Let options be the result of creating link options from el.
        let mut options = self.processing_options();

        // Step 3. Set options's destination to the empty string.
        options.destination = Some(Destination::None);

        // Step 4. Let request be the result of creating a link request given options.
        let url = options.base_url.clone();
        let Some(request) = options.create_link_request(self.owner_window().webview_id()) else {
            // Step 5. If request is null, then return.
            return;
        };

        // Step 6. Set request's initiator to "prefetch".
        let request = request.initiator(Initiator::Prefetch);

        // (Step 7, firing load/error events is handled in the FetchResponseListener impl for PrefetchContext)

        // Step 8. The user agent should fetch request, with processResponseConsumeBody set to processPrefetchResponse.
        let document = self.upcast::<Node>().owner_doc();
        let fetch_context = PrefetchContext {
            url,
            link: Trusted::new(self),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        };

        document.fetch_background(request, fetch_context);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-link-obtain>
    fn handle_stylesheet_url(&self, href: &str) {
        let document = self.owner_document();
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

        if !element.matches_environment(mq_str) {
            return;
        }

        let media = MediaList::parse_media_list(mq_str, document.window());

        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.as_ref().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        self.request_generation_id
            .set(self.request_generation_id.get().increment());

        let loader = StylesheetLoader::for_element(self.upcast());
        loader.load(
            StylesheetContextSource::LinkElement { media: Some(media) },
            link_url,
            cors_setting,
            integrity_metadata.to_owned(),
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-link-disabled>
    fn handle_disabled_attribute_change(&self, disabled: bool) {
        if !disabled {
            self.is_explicitly_enabled.set(true);
        }
        if let Some(stylesheet) = self.get_stylesheet() {
            if stylesheet.set_disabled(disabled) {
                self.stylesheet_list_owner().invalidate_stylesheets();
            }
        }
    }

    fn handle_favicon_url(&self, href: &str, _sizes: &Option<String>) {
        let document = self.owner_document();
        match document.base_url().join(href) {
            Ok(url) => {
                let window = document.window();
                if window.is_top_level() {
                    let msg = EmbedderMsg::NewFavicon(document.webview_id(), url.clone());
                    window.send_to_embedder(msg);
                }
            },
            Err(e) => debug!("Parsing url {} failed: {}", href, e),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#link-type-preload:fetch-and-process-the-linked-resource-2>
    fn handle_preload_url(&self) {
        // Step 1. Update the source set for el.
        // TODO
        // Step 2. Let options be the result of creating link options from el.
        let options = self.processing_options();
        // Step 3. Preload options, with the following steps given a response response:
        // Step 3.1 If response is a network error, fire an event named error at el.
        // Otherwise, fire an event named load at el.
        self.preload(options);
    }

    /// <https://html.spec.whatwg.org/multipage/#preload>
    fn preload(&self, options: LinkProcessingOptions) {
        // Step 1. If options's type doesn't match options's destination, then return.
        let type_matches_destination: bool =
            HTMLLinkElement::type_matches_destination(&options.link_type, options.destination);
        self.previous_type_matched.set(type_matches_destination);
        if !type_matches_destination {
            return;
        }
        // Step 2. If options's destination is "image" and options's source set is not null,
        // then set options's href to the result of selecting an image source from options's source set.
        // TODO
        // Step 3. Let request be the result of creating a link request given options.
        let url = options.base_url.clone();
        let Some(request) = options.create_link_request(self.owner_window().webview_id()) else {
            // Step 4. If request is null, then return.
            return;
        };
        let document = self.upcast::<Node>().owner_doc();
        // Step 5. Let unsafeEndTime be 0.
        // TODO
        // Step 6. Let entry be a new preload entry whose integrity metadata is options's integrity.
        // TODO
        // Step 7. Let key be the result of creating a preload key given request.
        // TODO
        // Step 8. If options's document is "pending", then set request's initiator type to "early hint".
        // TODO
        // Step 9. Let controller be null.
        // Step 10. Let reportTiming given a Document document be to report timing for controller
        // given document's relevant global object.
        // Step 11. Set controller to the result of fetching request, with processResponseConsumeBody
        // set to the following steps given a response response and null, failure, or a byte sequence bodyBytes:
        let fetch_context = PreloadContext {
            url,
            link: Trusted::new(self),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        };
        document.fetch_background(request.clone(), fetch_context);
    }

    /// <https://html.spec.whatwg.org/multipage/#match-preload-type>
    fn type_matches_destination(mime_type: &str, destination: Option<Destination>) -> bool {
        // Step 1. If type is an empty string, then return true.
        if mime_type.is_empty() {
            return true;
        }
        // Step 2. If destination is "fetch", then return true.
        //
        // Fetch is handled as an empty string destination in the spec:
        // https://fetch.spec.whatwg.org/#concept-potential-destination-translate
        let Some(destination) = destination else {
            return false;
        };
        if destination == Destination::None {
            return true;
        }
        // Step 3. Let mimeTypeRecord be the result of parsing type.
        let Ok(mime_type_record) = Mime::from_str(mime_type) else {
            // Step 4. If mimeTypeRecord is failure, then return false.
            return false;
        };
        // Step 5. If mimeTypeRecord is not supported by the user agent, then return false.
        //
        // We currently don't check if we actually support the mime type. Only if we can classify
        // it according to the spec.
        let Some(mime_type) = MimeClassifier::get_media_type(&mime_type_record) else {
            return false;
        };
        // Step 6. If any of the following are true:
        if
        // destination is "audio" or "video", and mimeTypeRecord is an audio or video MIME type;
        ((destination == Destination::Audio || destination == Destination::Video) &&
            mime_type == MediaType::AudioVideo)
            // destination is a script-like destination and mimeTypeRecord is a JavaScript MIME type;
            || (destination.is_script_like() && mime_type == MediaType::JavaScript)
            // destination is "image" and mimeTypeRecord is an image MIME type;
            || (destination == Destination::Image && mime_type == MediaType::Image)
            // destination is "font" and mimeTypeRecord is a font MIME type;
            || (destination == Destination::Font && mime_type == MediaType::Font)
            // destination is "json" and mimeTypeRecord is a JSON MIME type;
            || (destination == Destination::Json && mime_type == MediaType::Json)
            // destination is "style" and mimeTypeRecord's essence is text/css; or
            || (destination == Destination::Style && mime_type_record == mime::TEXT_CSS)
            // destination is "track" and mimeTypeRecord's essence is text/vtt,
            || (destination == Destination::Track && mime_type_record.essence_str() == "text/vtt")
        {
            // then return true.
            return true;
        }
        // Step 7. Return false.
        false
    }

    fn fire_event_after_response(&self, response: Result<ResourceFetchTiming, NetworkError>) {
        if response.is_err() {
            self.upcast::<EventTarget>()
                .fire_event(atom!("error"), CanGc::note());
        } else {
            // TODO(35035): Figure out why we need to queue a task for the load event. Otherwise
            // the performance timing data hasn't been saved yet, which fails several preload
            // WPT tests that assume that performance timing information is available when
            // the load event is fired.
            let this = Trusted::new(self);
            self.owner_global()
                .task_manager()
                .performance_timeline_task_source()
                .queue(task!(preload_load_event: move || {
                    let this = this.root();
                    this
                        .upcast::<EventTarget>()
                        .fire_event(atom!("load"), CanGc::note());
                }));
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

    fn referrer_policy(&self) -> ReferrerPolicy {
        if self.RelList(CanGc::note()).Contains("noreferrer".into()) {
            return ReferrerPolicy::NoReferrer;
        }

        ReferrerPolicy::EmptyString
    }

    fn set_origin_clean(&self, origin_clean: bool) {
        if let Some(stylesheet) = self.get_cssom_stylesheet(CanGc::note()) {
            stylesheet.set_origin_clean(origin_clean);
        }
    }
}

impl HTMLLinkElementMethods<crate::DomTypeHolder> for HTMLLinkElement {
    // https://html.spec.whatwg.org/multipage/#dom-link-href
    make_url_getter!(Href, "href");

    // https://html.spec.whatwg.org/multipage/#dom-link-href
    make_url_setter!(SetHref, "href");

    // https://html.spec.whatwg.org/multipage/#dom-link-rel
    make_getter!(Rel, "rel");

    // https://html.spec.whatwg.org/multipage/#dom-link-rel
    fn SetRel(&self, rel: DOMString, can_gc: CanGc) {
        self.upcast::<Element>()
            .set_tokenlist_attribute(&local_name!("rel"), rel, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-as
    make_enumerated_getter!(
        As,
        "as",
        "fetch" | "audio" | "audioworklet" | "document" | "embed" | "font" | "frame"
            | "iframe" | "image" | "json" | "manifest" | "object" | "paintworklet"
            | "report" | "script" | "serviceworker" | "sharedworker" | "style" | "track"
            | "video" | "webidentity" | "worker" | "xslt",
        missing => "",
        invalid => ""
    );

    // https://html.spec.whatwg.org/multipage/#dom-link-as
    make_setter!(SetAs, "as");

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

    // https://html.spec.whatwg.org/multipage/#dom-link-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-link-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-link-rellist
    fn RelList(&self, can_gc: CanGc) -> DomRoot<DOMTokenList> {
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
                can_gc,
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
    fn SetCrossOrigin(&self, value: Option<DOMString>, can_gc: CanGc) {
        set_cross_origin_attribute(self.upcast::<Element>(), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-referrerpolicy
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    // https://drafts.csswg.org/cssom/#dom-linkstyle-sheet
    fn GetSheet(&self, can_gc: CanGc) -> Option<DomRoot<DOMStyleSheet>> {
        self.get_cssom_stylesheet(can_gc).map(DomRoot::upcast)
    }
}

impl LinkProcessingOptions {
    /// <https://html.spec.whatwg.org/multipage/#create-a-link-request>
    fn create_link_request(self, webview_id: WebViewId) -> Option<RequestBuilder> {
        // Step 1. Assert: options's href is not the empty string.
        assert!(!self.href.is_empty());

        // Step 2. If options's destination is null, then return null.
        let destination = self.destination?;

        // Step 3. Let url be the result of encoding-parsing a URL given options's href, relative to options's base URL.
        // TODO: The spec passes a base url which is incompatible with the
        //       "encoding-parse a URL" algorithm.
        let Ok(url) = self.base_url.join(&self.href) else {
            // Step 4. If url is failure, then return null.
            return None;
        };

        // Step 5. Let request be the result of creating a potential-CORS request given
        //         url, options's destination, and options's crossorigin.
        // Step 6. Set request's policy container to options's policy container.
        // Step 7. Set request's integrity metadata to options's integrity.
        // Step 8. Set request's cryptographic nonce metadata to options's cryptographic nonce metadata.
        // Step 9. Set request's referrer policy to options's referrer policy.
        // FIXME: Step 10. Set request's client to options's environment.
        // FIXME: Step 11. Set request's priority to options's fetch priority.
        // FIXME: Use correct referrer
        let builder = create_a_potential_cors_request(
            Some(webview_id),
            url,
            destination,
            self.cross_origin,
            None,
            Referrer::NoReferrer,
            self.insecure_requests_policy,
            self.has_trustworthy_ancestor_origin,
            self.policy_container,
        )
        .initiator(Initiator::Link)
        .origin(self.origin)
        .integrity_metadata(self.integrity)
        .cryptographic_nonce_metadata(self.cryptographic_nonce_metadata)
        .referrer_policy(self.referrer_policy);

        // Step 12. Return request.
        Some(builder)
    }
}

/// <https://html.spec.whatwg.org/multipage/#translate-a-preload-destination>
fn translate_a_preload_destination(potential_destination: &str) -> Destination {
    match potential_destination {
        "fetch" => Destination::None,
        "font" => Destination::Font,
        "image" => Destination::Image,
        "script" => Destination::Script,
        "track" => Destination::Track,
        _ => Destination::None,
    }
}

struct PrefetchContext {
    /// The `<link>` element that caused this prefetch operation
    link: Trusted<HTMLLinkElement>,

    resource_timing: ResourceFetchTiming,

    /// The url being prefetched
    url: ServoUrl,
}

impl FetchResponseListener for PrefetchContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        _ = chunk;
    }

    // Step 7 of `fetch and process the linked resource` in https://html.spec.whatwg.org/multipage/#link-type-prefetch
    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        if response.is_err() {
            // Step 1. If response is a network error, fire an event named error at el.
            self.link
                .root()
                .upcast::<EventTarget>()
                .fire_event(atom!("error"), CanGc::note());
        } else {
            // Step 2. Otherwise, fire an event named load at el.
            self.link
                .root()
                .upcast::<EventTarget>()
                .fire_event(atom!("load"), CanGc::note());
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None);
    }
}

impl ResourceTimingListener for PrefetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName("prefetch".to_string()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.link.root().upcast::<Node>().owner_doc().global()
    }
}

impl PreInvoke for PrefetchContext {
    fn should_invoke(&self) -> bool {
        // Prefetch requests are never aborted.
        true
    }
}

struct PreloadContext {
    /// The `<link>` element that caused this preload operation
    link: Trusted<HTMLLinkElement>,

    resource_timing: ResourceFetchTiming,

    /// The url being preloaded
    url: ServoUrl,
}

impl FetchResponseListener for PreloadContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: RequestId,
        fetch_metadata: Result<FetchMetadata, NetworkError>,
    ) {
        _ = fetch_metadata;
    }

    fn process_response_chunk(&mut self, _: RequestId, chunk: Vec<u8>) {
        _ = chunk;
    }

    /// Step 3.1 of <https://html.spec.whatwg.org/multipage/#link-type-preload:fetch-and-process-the-linked-resource-2>
    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        self.link.root().fire_event_after_response(response);
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        submit_timing(self, CanGc::note())
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None);
    }
}

impl ResourceTimingListener for PreloadContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName(self.url.clone().into_string()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.link.root().upcast::<Node>().owner_doc().global()
    }
}

impl PreInvoke for PreloadContext {
    fn should_invoke(&self) -> bool {
        // Preload requests are never aborted.
        true
    }
}
