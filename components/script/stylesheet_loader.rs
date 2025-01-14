/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{Read, Seek, Write};
use std::sync::atomic::AtomicBool;

use base::id::PipelineId;
use cssparser::SourceLocation;
use encoding_rs::UTF_8;
use mime::{self, Mime};
use net_traits::request::{CorsSettings, Destination, Referrer, RequestBuilder, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, FilteredMetadata, Metadata, NetworkError, ReferrerPolicy,
    ResourceFetchTiming, ResourceTimingType,
};
use servo_arc::Arc;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::import_rule::{ImportLayer, ImportSheet, ImportSupportsCondition};
use style::stylesheets::{
    CssRules, ImportRule, Origin, Stylesheet, StylesheetContents,
    StylesheetLoader as StyleStylesheetLoader, UrlExtraData,
};
use style::values::CssUrl;

use crate::document_loader::LoadType;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmllinkelement::{HTMLLinkElement, RequestGenerationId};
use crate::dom::node::NodeTraits;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::shadowroot::ShadowRoot;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::script_runtime::CanGc;
use crate::unminify::{
    create_output_file, create_temp_files, execute_js_beautify, BeautifyFileType,
};

pub(crate) trait StylesheetOwner {
    /// Returns whether this element was inserted by the parser (i.e., it should
    /// trigger a document-load-blocking load).
    fn parser_inserted(&self) -> bool;

    /// Which referrer policy should loads triggered by this owner follow
    fn referrer_policy(&self) -> ReferrerPolicy;

    /// Notes that a new load is pending to finish.
    fn increment_pending_loads_count(&self);

    /// Returns None if there are still pending loads, or whether any load has
    /// failed since the loads started.
    fn load_finished(&self, successful: bool) -> Option<bool>;

    /// Sets origin_clean flag.
    fn set_origin_clean(&self, origin_clean: bool);
}

pub(crate) enum StylesheetContextSource {
    // NB: `media` is just an option so we avoid cloning it.
    LinkElement { media: Option<MediaList> },
    Import(Arc<Stylesheet>),
}

/// The context required for asynchronously loading an external stylesheet.
pub(crate) struct StylesheetContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLElement>,
    source: StylesheetContextSource,
    url: ServoUrl,
    metadata: Option<Metadata>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The node document for elem when the load was initiated.
    document: Trusted<Document>,
    shadow_root: Option<Trusted<ShadowRoot>>,
    origin_clean: bool,
    /// A token which must match the generation id of the `HTMLLinkElement` for it to load the stylesheet.
    /// This is ignored for `HTMLStyleElement` and imports.
    request_generation_id: Option<RequestGenerationId>,
    resource_timing: ResourceFetchTiming,
}

impl StylesheetContext {
    fn unminify_css(&self, data: Vec<u8>, file_url: ServoUrl) -> Vec<u8> {
        let Some(unminified_dir) = self.document.root().window().unminified_css_dir() else {
            return data;
        };

        let mut style_content = data;

        if let Some((input, mut output)) = create_temp_files() {
            if execute_js_beautify(
                input.path(),
                output.try_clone().unwrap(),
                BeautifyFileType::Css,
            ) {
                output.seek(std::io::SeekFrom::Start(0)).unwrap();
                output.read_to_end(&mut style_content).unwrap();
            }
        }
        match create_output_file(unminified_dir, &file_url, None) {
            Ok(mut file) => {
                file.write_all(&style_content).unwrap();
            },
            Err(why) => {
                log::warn!("Could not store script {:?}", why);
            },
        }

        style_content
    }
}

impl PreInvoke for StylesheetContext {}

impl FetchResponseListener for StylesheetContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        if let Ok(FetchMetadata::Filtered {
            filtered: FilteredMetadata::Opaque | FilteredMetadata::OpaqueRedirect(_),
            ..
        }) = metadata
        {
            self.origin_clean = false;
        }

        self.metadata = metadata.ok().map(|m| match m {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });
    }

    fn process_response_chunk(&mut self, _: RequestId, mut payload: Vec<u8>) {
        self.data.append(&mut payload);
    }

    fn process_response_eof(
        &mut self,
        _: RequestId,
        status: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let elem = self.elem.root();
        let document = self.document.root();
        let mut successful = false;

        if status.is_ok() {
            let metadata = match self.metadata.take() {
                Some(meta) => meta,
                None => return,
            };
            let is_css = metadata.content_type.is_some_and(|ct| {
                let mime: Mime = ct.into_inner().into();
                mime.type_() == mime::TEXT && mime.subtype() == mime::CSS
            });

            let data = if is_css {
                let data = std::mem::take(&mut self.data);
                self.unminify_css(data, metadata.final_url.clone())
            } else {
                vec![]
            };

            // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
            let environment_encoding = UTF_8;
            let protocol_encoding_label = metadata.charset.as_deref();
            let final_url = metadata.final_url;

            let win = elem.owner_window();

            let loader = StylesheetLoader::for_element(&elem);
            match self.source {
                StylesheetContextSource::LinkElement { ref mut media } => {
                    let link = elem.downcast::<HTMLLinkElement>().unwrap();
                    // We must first check whether the generations of the context and the element match up,
                    // else we risk applying the wrong stylesheet when responses come out-of-order.
                    let is_stylesheet_load_applicable = self
                        .request_generation_id
                        .map_or(true, |gen| gen == link.get_request_generation_id());
                    if is_stylesheet_load_applicable {
                        let shared_lock = document.style_shared_lock().clone();
                        let sheet = Arc::new(Stylesheet::from_bytes(
                            &data,
                            UrlExtraData(final_url.get_arc()),
                            protocol_encoding_label,
                            Some(environment_encoding),
                            Origin::Author,
                            media.take().unwrap(),
                            shared_lock,
                            Some(&loader),
                            win.css_error_reporter(),
                            document.quirks_mode(),
                        ));

                        if link.is_alternate() {
                            sheet.set_disabled(true);
                        }

                        link.set_stylesheet(sheet);
                    }
                },
                StylesheetContextSource::Import(ref stylesheet) => {
                    Stylesheet::update_from_bytes(
                        stylesheet,
                        &data,
                        protocol_encoding_label,
                        Some(environment_encoding),
                        UrlExtraData(final_url.get_arc()),
                        Some(&loader),
                        win.css_error_reporter(),
                    );

                    // Layout knows about this stylesheet, because Stylo added it to the Stylist,
                    // but Layout doesn't know about any new web fonts that it contains.
                    document.load_web_fonts_from_stylesheet(stylesheet.clone());
                },
            }

            if let Some(ref shadow_root) = self.shadow_root {
                shadow_root.root().invalidate_stylesheets();
            } else {
                document.invalidate_stylesheets();
            }

            // FIXME: Revisit once consensus is reached at:
            // https://github.com/whatwg/html/issues/1142
            successful = metadata.status == http::StatusCode::OK;
        }

        let owner = elem
            .upcast::<Element>()
            .as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        owner.set_origin_clean(self.origin_clean);
        if owner.parser_inserted() {
            document.decrement_script_blocking_stylesheet_count();
        }

        document.finish_load(LoadType::Stylesheet(self.url.clone()), CanGc::note());

        if let Some(any_failed) = owner.load_finished(successful) {
            let event = if any_failed {
                atom!("error")
            } else {
                atom!("load")
            };
            elem.upcast::<EventTarget>()
                .fire_event(event, CanGc::note());
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
    }
}

impl ResourceTimingListener for StylesheetContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName(
            self.elem
                .root()
                .upcast::<Element>()
                .local_name()
                .to_string(),
        );
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.elem.root().owner_document().global()
    }
}

pub(crate) struct StylesheetLoader<'a> {
    elem: &'a HTMLElement,
}

impl<'a> StylesheetLoader<'a> {
    pub(crate) fn for_element(element: &'a HTMLElement) -> Self {
        StylesheetLoader { elem: element }
    }
}

impl StylesheetLoader<'_> {
    pub(crate) fn load(
        &self,
        source: StylesheetContextSource,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
        integrity_metadata: String,
    ) {
        let document = self.elem.owner_document();
        let shadow_root = self
            .elem
            .containing_shadow_root()
            .map(|sr| Trusted::new(&*sr));
        let gen = self
            .elem
            .downcast::<HTMLLinkElement>()
            .map(HTMLLinkElement::get_request_generation_id);
        let context = StylesheetContext {
            elem: Trusted::new(self.elem),
            source,
            url: url.clone(),
            metadata: None,
            data: vec![],
            document: Trusted::new(&*document),
            shadow_root,
            origin_clean: true,
            request_generation_id: gen,
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        };

        let owner = self
            .elem
            .upcast::<Element>()
            .as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        let referrer_policy = owner.referrer_policy();
        owner.increment_pending_loads_count();
        if owner.parser_inserted() {
            document.increment_script_blocking_stylesheet_count();
        }

        let request = stylesheet_fetch_request(
            url.clone(),
            cors_setting,
            document.origin().immutable().clone(),
            self.elem.global().pipeline_id(),
            self.elem.global().get_referrer(),
            referrer_policy,
            integrity_metadata,
        );
        let request = document.prepare_request(request);

        document.fetch(LoadType::Stylesheet(url), request, context);
    }
}

// This function is also used to prefetch a stylesheet in `script::dom::servoparser::prefetch`.
// https://html.spec.whatwg.org/multipage/#default-fetch-and-process-the-linked-resource
pub(crate) fn stylesheet_fetch_request(
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    origin: ImmutableOrigin,
    pipeline_id: PipelineId,
    referrer: Referrer,
    referrer_policy: ReferrerPolicy,
    integrity_metadata: String,
) -> RequestBuilder {
    create_a_potential_cors_request(url, Destination::Style, cors_setting, None, referrer)
        .origin(origin)
        .pipeline_id(Some(pipeline_id))
        .referrer_policy(referrer_policy)
        .integrity_metadata(integrity_metadata)
}

impl StyleStylesheetLoader for StylesheetLoader<'_> {
    /// Request a stylesheet after parsing a given `@import` rule, and return
    /// the constructed `@import` rule.
    fn request_stylesheet(
        &self,
        url: CssUrl,
        source_location: SourceLocation,
        context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
        supports: Option<ImportSupportsCondition>,
        layer: ImportLayer,
    ) -> Arc<Locked<ImportRule>> {
        // Ensure the supports conditions for this @import are true, if not, refuse to load
        if !supports.as_ref().map_or(true, |s| s.enabled) {
            return Arc::new(lock.wrap(ImportRule {
                url,
                stylesheet: ImportSheet::new_refused(),
                supports,
                layer,
                source_location,
            }));
        }

        let sheet = Arc::new(Stylesheet {
            contents: StylesheetContents::from_data(
                CssRules::new(Vec::new(), lock),
                context.stylesheet_origin,
                context.url_data.clone(),
                context.quirks_mode,
            ),
            media,
            shared_lock: lock.clone(),
            disabled: AtomicBool::new(false),
        });

        let stylesheet = ImportSheet::new(sheet.clone());
        let import = ImportRule {
            url,
            stylesheet,
            supports,
            layer,
            source_location,
        };

        let url = match import.url.url().cloned() {
            Some(url) => url,
            None => return Arc::new(lock.wrap(import)),
        };

        // TODO (mrnayak) : Whether we should use the original loader's CORS
        // setting? Fix this when spec has more details.
        let source = StylesheetContextSource::Import(sheet.clone());
        self.load(source, url.into(), None, "".to_owned());

        Arc::new(lock.wrap(import))
    }
}
