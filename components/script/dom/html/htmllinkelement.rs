/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::{Borrow, ToOwned};
use std::cell::Cell;
use std::default::Default;

use base::generic_channel::GenericSharedMemory;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use net_traits::image_cache::{
    Image, ImageCache, ImageCacheResponseCallback, ImageCacheResult, ImageLoadListener,
    ImageOrMetadataAvailable, ImageResponse, PendingImageId,
};
use net_traits::request::{Destination, Initiator, RequestBuilder, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseMsg, NetworkError, ReferrerPolicy, ResourceFetchTiming,
};
use pixels::PixelFormat;
use script_bindings::root::Dom;
use servo_arc::Arc;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::stylesheets::Stylesheet;
use stylo_atoms::Atom;
use webrender_api::units::DeviceIntSize;

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
use crate::dom::css::cssstylesheet::CSSStyleSheet;
use crate::dom::css::stylesheet::StyleSheet as DOMStyleSheet;
use crate::dom::document::Document;
use crate::dom::documentorshadowroot::StylesheetSource;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{
    AttributeMutation, Element, ElementCreator, cors_setting_for_element,
    referrer_policy_for_element, reflect_cross_origin_attribute, reflect_referrer_policy_attribute,
    set_cross_origin_attribute,
};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::medialist::MediaList;
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::processingoptions::{
    LinkFetchContext, LinkFetchContextType, LinkProcessingOptions,
};
use crate::dom::types::{EventTarget, GlobalScope};
use crate::dom::virtualmethods::VirtualMethods;
use crate::links::LinkRelations;
use crate::network_listener::{FetchResponseListener, ResourceTimingListener, submit_timing};
use crate::script_runtime::CanGc;
use crate::stylesheet_loader::{ElementStylesheetLoader, StylesheetContextSource, StylesheetOwner};

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct RequestGenerationId(u32);

impl RequestGenerationId {
    fn increment(self) -> RequestGenerationId {
        RequestGenerationId(self.0 + 1)
    }
}

#[dom_struct]
pub(crate) struct HTMLLinkElement {
    htmlelement: HTMLElement,
    /// The relations as specified by the "rel" attribute
    rel_list: MutNullableDom<DOMTokenList>,

    /// The link relations as they are used in practice.
    ///
    /// The reason this is separate from [HTMLLinkElement::rel_list] is that
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
    /// Line number this element was created on
    line_number: u64,
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
            line_number: creator.return_line_number(),
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
        stylesheets_owner.add_owned_stylesheet(self.upcast(), s);
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
                    None, // constructor_document
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
            cssom_stylesheet.set_owner_node(None);
        }
        self.cssom_stylesheet.set(None);
    }

    pub(crate) fn line_number(&self) -> u32 {
        self.line_number as u32
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
                    self.handle_favicon_url(&attr.value());
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
                self.handle_favicon_url(&attr.value());
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
                    if let AttributeMutation::Set(Some(_), _) = mutation {
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
                        AttributeMutation::Removed | AttributeMutation::Set(Some(_), _) => {
                            self.handle_preload_url()
                        },
                        _ => {},
                    };
                }

                let matches_media_environment =
                    MediaList::matches_environment(&self.owner_document(), &attr.value());
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
                    self.handle_favicon_url(&href);
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
    fn compute_destination_for_attribute(&self) -> Option<Destination> {
        // Let destination be the result of translating the keyword
        // representing the state of el's as attribute.
        let element = self.upcast::<Element>();
        element
            .get_attribute(&ns!(), &local_name!("as"))
            .and_then(|attr| LinkProcessingOptions::translate_a_preload_destination(&attr.value()))
    }

    /// <https://html.spec.whatwg.org/multipage/#create-link-options-from-element>
    fn processing_options(&self) -> LinkProcessingOptions {
        let element = self.upcast::<Element>();

        // Step 1. Let document be el's node document.
        let document = self.upcast::<Node>().owner_doc();

        // Step 2. Let options be a new link processing options
        let mut options = LinkProcessingOptions {
            href: String::new(),
            destination: Destination::None,
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

    /// <https://html.spec.whatwg.org/multipage/#default-fetch-and-process-the-linked-resource>
    ///
    /// This method does not implement Step 7 (fetching the request) and instead returns the [RequestBuilder],
    /// as the fetch context that should be used depends on the link type.
    fn default_fetch_and_process_the_linked_resource(&self) -> Option<RequestBuilder> {
        // Step 1. Let options be the result of creating link options from el.
        let options = self.processing_options();

        // Step 2. Let request be the result of creating a link request given options.
        let Some(request) = options.create_link_request(self.owner_window().webview_id()) else {
            // Step 3. If request is null, then return.
            return None;
        };
        // Step 4. Set request's synchronous flag.
        let mut request = request.synchronous(true);

        // Step 5. Run the linked resource fetch setup steps, given el and request. If the result is false, then return.
        if !self.linked_resource_fetch_setup(&mut request) {
            return None;
        }

        // TODO Step 6. Set request's initiator type to "css" if el's rel attribute
        // contains the keyword stylesheet; "link" otherwise.

        // Step 7. Fetch request with processResponseConsumeBody set to the following steps given response response and null,
        // failure, or a byte sequence bodyBytes: [..]
        Some(request)
    }

    /// <https://html.spec.whatwg.org/multipage/#linked-resource-fetch-setup-steps>
    fn linked_resource_fetch_setup(&self, request: &mut RequestBuilder) -> bool {
        if self.relations.get().contains(LinkRelations::ICON) {
            // Step 1. Set request's destination to "image".
            request.destination = Destination::Image;

            // Step 2. Return true.
            return true;
        }

        true
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
        options.destination = Destination::None;

        // Step 4. Let request be the result of creating a link request given options.
        let Some(request) = options.create_link_request(self.owner_window().webview_id()) else {
            // Step 5. If request is null, then return.
            return;
        };
        let url = request.url.clone();

        // Step 6. Set request's initiator to "prefetch".
        let request = request.initiator(Initiator::Prefetch);

        // (Step 7, firing load/error events is handled in the FetchResponseListener impl for LinkFetchContext)

        // Step 8. The user agent should fetch request, with processResponseConsumeBody set to processPrefetchResponse.
        let document = self.upcast::<Node>().owner_doc();
        let fetch_context = LinkFetchContext {
            url,
            link: Some(Trusted::new(self)),
            document: Trusted::new(&document),
            global: Trusted::new(&document.global()),
            type_: LinkFetchContextType::Prefetch,
            response_body: vec![],
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

        if !MediaList::matches_environment(&document, mq_str) {
            return;
        }

        let media = MediaList::parse_media_list(mq_str, document.window());
        let media = Arc::new(document.style_shared_lock().wrap(media));

        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.as_ref().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        self.request_generation_id
            .set(self.request_generation_id.get().increment());

        let loader = ElementStylesheetLoader::new(self.upcast());
        loader.load(
            StylesheetContextSource::LinkElement,
            media,
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

    fn handle_favicon_url(&self, href: &str) {
        // If el's href attribute's value is the empty string, then return.
        if href.is_empty() {
            return;
        }

        // The spec does not specify this, but we don't fetch favicons for iframes, as
        // they won't be displayed anyways.
        let window = self.owner_window();
        if !window.is_top_level() {
            return;
        }
        let Ok(href) = self.Href().parse() else {
            return;
        };

        // Ignore all previous fetch operations
        self.request_generation_id
            .set(self.request_generation_id.get().increment());

        let cache_result = window.image_cache().get_cached_image_status(
            href,
            window.origin().immutable().clone(),
            cors_setting_for_element(self.upcast()),
        );

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                image, ..
            }) => {
                self.process_favicon_response(image);
            },
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(_, id)) |
            ImageCacheResult::Pending(id) => {
                let sender = self.register_image_cache_callback(id);
                window.image_cache().add_listener(ImageLoadListener::new(
                    sender,
                    window.pipeline_id(),
                    id,
                ));
            },
            ImageCacheResult::ReadyForRequest(id) => {
                let Some(request) = self.default_fetch_and_process_the_linked_resource() else {
                    return;
                };

                let sender = self.register_image_cache_callback(id);
                window.image_cache().add_listener(ImageLoadListener::new(
                    sender,
                    window.pipeline_id(),
                    id,
                ));

                let document = self.upcast::<Node>().owner_doc();
                let fetch_context = FaviconFetchContext {
                    url: self.owner_document().base_url(),
                    image_cache: window.image_cache(),
                    id,
                    link: Trusted::new(self),
                };
                document.fetch_background(request, fetch_context);
            },
            ImageCacheResult::FailedToLoadOrDecode => {},
        };
    }

    fn register_image_cache_callback(&self, id: PendingImageId) -> ImageCacheResponseCallback {
        let trusted_node = Trusted::new(self);
        let window = self.owner_window();
        let request_generation_id = self.get_request_generation_id();
        window.register_image_cache_listener(id, move |response| {
            let trusted_node = trusted_node.clone();
            let link_element = trusted_node.root();
            let window = link_element.owner_window();

            let ImageResponse::Loaded(image, _) = response.response else {
                // We don't care about metadata and such for favicons.
                return;
            };

            if request_generation_id != link_element.get_request_generation_id() {
                // This load is no longer relevant.
                return;
            };

            window
                .as_global_scope()
                .task_manager()
                .networking_task_source()
                .queue(task!(process_favicon_response: move || {
                    let element = trusted_node.root();

                    if request_generation_id != element.get_request_generation_id() {
                        // This load is no longer relevant.
                        return;
                    };

                    element.process_favicon_response(image);
                }));
        })
    }

    /// Rasterizes a loaded favicon file if necessary and notifies the embedder about it.
    fn process_favicon_response(&self, image: Image) {
        // TODO: Include the size attribute here
        let window = self.owner_window();
        let document = self.owner_document();

        let send_rasterized_favicon_to_embedder = |raster_image: &pixels::RasterImage| {
            // Let's not worry about animated favicons...
            let frame = raster_image.first_frame();

            let format = match raster_image.format {
                PixelFormat::K8 => embedder_traits::PixelFormat::K8,
                PixelFormat::KA8 => embedder_traits::PixelFormat::KA8,
                PixelFormat::RGB8 => embedder_traits::PixelFormat::RGB8,
                PixelFormat::RGBA8 => embedder_traits::PixelFormat::RGBA8,
                PixelFormat::BGRA8 => embedder_traits::PixelFormat::BGRA8,
            };

            let embedder_image = embedder_traits::Image::new(
                frame.width,
                frame.height,
                std::sync::Arc::new(GenericSharedMemory::from_bytes(&raster_image.bytes)),
                raster_image.frames[0].byte_range.clone(),
                format,
            );
            document.set_favicon(embedder_image);
        };

        match image {
            Image::Raster(raster_image) => send_rasterized_favicon_to_embedder(&raster_image),
            Image::Vector(vector_image) => {
                // This size is completely arbitrary.
                let size = DeviceIntSize::new(250, 250);

                let image_cache = window.image_cache();
                if let Some(raster_image) =
                    image_cache.rasterize_vector_image(vector_image.id, size)
                {
                    send_rasterized_favicon_to_embedder(&raster_image);
                } else {
                    // The rasterization callback will end up calling "process_favicon_response" again,
                    // but this time with a raster image.
                    let image_cache_sender = self.register_image_cache_callback(vector_image.id);
                    image_cache.add_rasterization_complete_listener(
                        window.pipeline_id(),
                        vector_image.id,
                        size,
                        image_cache_sender,
                    );
                }
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#link-type-preload:fetch-and-process-the-linked-resource-2>
    /// and type matching destination steps of <https://html.spec.whatwg.org/multipage/#preload>
    fn handle_preload_url(&self) {
        // Step 1. Update the source set for el.
        // TODO
        // Step 2. Let options be the result of creating link options from el.
        let mut options = self.processing_options();
        // Step 3. Let destination be the result of translating the keyword
        // representing the state of el's as attribute.
        let Some(destination) = self.compute_destination_for_attribute() else {
            // Step 4. If destination is null, then return.
            return;
        };
        // Step 5. Set options's destination to destination.
        options.destination = destination;
        // Steps for https://html.spec.whatwg.org/multipage/#preload
        {
            // Step 1. If options's type doesn't match options's destination, then return.
            let type_matches_destination = options.type_matches_destination();
            self.previous_type_matched.set(type_matches_destination);
            if !type_matches_destination {
                return;
            }
        }
        // Step 6. Preload options, with the following steps given a response response:
        let document = self.upcast::<Node>().owner_doc();
        options.preload(
            self.owner_window().webview_id(),
            Some(Trusted::new(self)),
            &document,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#link-type-preload:fetch-and-process-the-linked-resource-2>
    pub(crate) fn fire_event_after_response(
        &self,
        response: Result<ResourceFetchTiming, NetworkError>,
        can_gc: CanGc,
    ) {
        // Step 3.1 If response is a network error, fire an event named error at el.
        // Otherwise, fire an event named load at el.
        if response.is_err() {
            self.upcast::<EventTarget>()
                .fire_event(atom!("error"), can_gc);
        } else {
            self.upcast::<EventTarget>()
                .fire_event(atom!("load"), can_gc);
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

    /// <https://html.spec.whatwg.org/multipage/#dom-link-rel>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-link-rellist>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-link-crossorigin>
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-link-crossorigin>
    fn SetCrossOrigin(&self, value: Option<DOMString>, can_gc: CanGc) {
        set_cross_origin_attribute(self.upcast::<Element>(), value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-link-referrerpolicy>
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-link-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    /// <https://drafts.csswg.org/cssom/#dom-linkstyle-sheet>
    fn GetSheet(&self, can_gc: CanGc) -> Option<DomRoot<DOMStyleSheet>> {
        self.get_cssom_stylesheet(can_gc).map(DomRoot::upcast)
    }
}

struct FaviconFetchContext {
    /// The `<link>` element that caused this fetch operation
    link: Trusted<HTMLLinkElement>,
    image_cache: std::sync::Arc<dyn ImageCache>,
    id: PendingImageId,

    /// The base url of the document that the `<link>` element belongs to.
    url: ServoUrl,
}

impl FetchResponseListener for FaviconFetchContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(request_id, metadata.clone()),
        );
    }

    fn process_response_chunk(&mut self, request_id: RequestId, chunk: Vec<u8>) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseChunk(request_id, chunk.into()),
        );
    }

    fn process_response_eof(
        self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response.clone()),
        );
        if let Ok(response) = response {
            submit_timing(&self, &response, CanGc::note());
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        let link = self.link.root();
        let source_position = link
            .upcast::<Element>()
            .compute_source_position(link.line_number as u32);
        global.report_csp_violations(violations, None, Some(source_position));
    }
}

impl ResourceTimingListener for FaviconFetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName("link".to_string()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.link.root().upcast::<Node>().owner_doc().global()
    }
}
