/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{Read, Seek, Write};

use cssparser::SourceLocation;
use encoding_rs::UTF_8;
use mime::{self, Mime};
use net_traits::request::{CorsSettings, Destination, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, FilteredMetadata, Metadata, NetworkError, ReferrerPolicy,
    ResourceFetchTiming, ResourceTimingType,
};
use servo_arc::Arc;
use servo_url::ServoUrl;
use style::context::QuirksMode;
use style::media_queries::MediaList;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::import_rule::{ImportLayer, ImportSheet, ImportSupportsCondition};
use style::stylesheets::{
    ImportRule, Origin, Stylesheet, StylesheetLoader as StyleStylesheetLoader, UrlExtraData,
};
use style::values::CssUrl;

use crate::document_loader::LoadType;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmllinkelement::{HTMLLinkElement, RequestGenerationId};
use crate::dom::node::NodeTraits;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::shadowroot::ShadowRoot;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::script_runtime::CanGc;
use crate::unminify::{
    BeautifyFileType, create_output_file, create_temp_files, execute_js_beautify,
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
    LinkElement {
        media: Option<MediaList>,
    },
    Import {
        import_rule: Arc<Locked<ImportRule>>,
        media: Option<MediaList>,
    },
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
        let element = self.elem.root();
        let document = self.document.root();
        let mut successful = false;

        if status.is_ok() {
            let metadata = match self.metadata.take() {
                Some(meta) => meta,
                None => return,
            };

            let mut is_css = metadata.content_type.is_some_and(|ct| {
                let mime: Mime = ct.into_inner().into();
                mime.type_() == mime::TEXT && mime.subtype() == mime::CSS
            }) || (
                // Quirk: If the document has been set to quirks mode,
                // has the same origin as the URL of the external resource,
                // and the Content-Type metadata of the external resource
                // is not a supported style sheet type, the user agent must
                // instead assume it to be text/css.
                // <https://html.spec.whatwg.org/multipage/#link-type-stylesheet>
                document.quirks_mode() == QuirksMode::Quirks &&
                    document.origin().immutable().clone() == metadata.final_url.origin()
            );

            // From <https://html.spec.whatwg.org/multipage/#link-type-stylesheet>:
            // > Quirk: If the document has been set to quirks mode, has the same origin as
            // > the URL of the external resource, and the Content-Type metadata of the
            // > external resource is not a supported style sheet type, the user agent must
            // > instead assume it to be text/css.
            if document.quirks_mode() == QuirksMode::Quirks &&
                document.url().origin() == self.url.origin()
            {
                is_css = true;
            }

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

            let win = element.owner_window();

            let loader = ElementStylesheetLoader::new(&element);
            let shared_lock = document.style_shared_lock();
            let stylesheet = |media| {
                #[cfg(feature = "tracing")]
                let _span =
                    tracing::trace_span!("ParseStylesheet", servo_profiling = true).entered();
                Arc::new(Stylesheet::from_bytes(
                    &data,
                    UrlExtraData(final_url.get_arc()),
                    protocol_encoding_label,
                    Some(environment_encoding),
                    Origin::Author,
                    media,
                    shared_lock.clone(),
                    Some(&loader),
                    win.css_error_reporter(),
                    document.quirks_mode(),
                ))
            };
            match self.source {
                StylesheetContextSource::LinkElement { ref mut media } => {
                    let link = element.downcast::<HTMLLinkElement>().unwrap();
                    // We must first check whether the generations of the context and the element match up,
                    // else we risk applying the wrong stylesheet when responses come out-of-order.
                    let is_stylesheet_load_applicable = self
                        .request_generation_id
                        .is_none_or(|generation| generation == link.get_request_generation_id());
                    if is_stylesheet_load_applicable {
                        let stylesheet = stylesheet(media.take().unwrap());
                        if link.is_effectively_disabled() {
                            stylesheet.set_disabled(true);
                        }
                        link.set_stylesheet(stylesheet);
                    }
                },
                StylesheetContextSource::Import {
                    ref mut import_rule,
                    ref mut media,
                } => {
                    let stylesheet = stylesheet(media.take().unwrap());

                    // Layout knows about this stylesheet, because Stylo added it to the Stylist,
                    // but Layout doesn't know about any new web fonts that it contains.
                    document.load_web_fonts_from_stylesheet(&stylesheet);

                    let mut guard = shared_lock.write();
                    import_rule.write_with(&mut guard).stylesheet = ImportSheet::Sheet(stylesheet);
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

        let owner = element
            .upcast::<Element>()
            .as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        owner.set_origin_clean(self.origin_clean);
        if owner.parser_inserted() {
            document.decrement_script_blocking_stylesheet_count();
        }

        // From <https://html.spec.whatwg.org/multipage/#link-type-stylesheet>:
        // > A link element of this type is implicitly potentially render-blocking if the element
        // > was created by its node document's parser.
        if matches!(self.source, StylesheetContextSource::LinkElement { .. }) &&
            owner.parser_inserted()
        {
            document.decrement_render_blocking_element_count();
        }

        document.finish_load(LoadType::Stylesheet(self.url.clone()), CanGc::note());

        if let Some(any_failed) = owner.load_finished(successful) {
            let event = if any_failed {
                atom!("error")
            } else {
                atom!("load")
            };
            element
                .upcast::<EventTarget>()
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

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
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

pub(crate) struct ElementStylesheetLoader<'a> {
    element: &'a HTMLElement,
}

impl<'a> ElementStylesheetLoader<'a> {
    pub(crate) fn new(element: &'a HTMLElement) -> Self {
        ElementStylesheetLoader { element }
    }
}

impl ElementStylesheetLoader<'_> {
    pub(crate) fn load(
        &self,
        source: StylesheetContextSource,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
        integrity_metadata: String,
    ) {
        let document = self.element.owner_document();
        let shadow_root = self
            .element
            .containing_shadow_root()
            .map(|sr| Trusted::new(&*sr));
        let generation = self
            .element
            .downcast::<HTMLLinkElement>()
            .map(HTMLLinkElement::get_request_generation_id);
        let context = StylesheetContext {
            elem: Trusted::new(self.element),
            source,
            url: url.clone(),
            metadata: None,
            data: vec![],
            document: Trusted::new(&*document),
            shadow_root,
            origin_clean: true,
            request_generation_id: generation,
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
        };

        let owner = self
            .element
            .upcast::<Element>()
            .as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        let referrer_policy = owner.referrer_policy();
        owner.increment_pending_loads_count();
        if owner.parser_inserted() {
            document.increment_script_blocking_stylesheet_count();
        }

        // From <https://html.spec.whatwg.org/multipage/#link-type-stylesheet>:
        // > A link element of this type is implicitly potentially render-blocking if the element
        // > was created by its node document's parser.
        if matches!(context.source, StylesheetContextSource::LinkElement { .. }) &&
            owner.parser_inserted()
        {
            document.increment_render_blocking_element_count();
        }

        // https://html.spec.whatwg.org/multipage/#default-fetch-and-process-the-linked-resource
        let global = self.element.global();
        let request = create_a_potential_cors_request(
            Some(document.webview_id()),
            url.clone(),
            Destination::Style,
            cors_setting,
            None,
            global.get_referrer(),
            document.insecure_requests_policy(),
            document.has_trustworthy_ancestor_or_current_origin(),
            global.policy_container(),
        )
        .origin(document.origin().immutable().clone())
        .pipeline_id(Some(self.element.global().pipeline_id()))
        .referrer_policy(referrer_policy)
        .integrity_metadata(integrity_metadata);

        document.fetch(LoadType::Stylesheet(url), request, context);
    }
}

impl StyleStylesheetLoader for ElementStylesheetLoader<'_> {
    /// Request a stylesheet after parsing a given `@import` rule, and return
    /// the constructed `@import` rule.
    fn request_stylesheet(
        &self,
        url: CssUrl,
        source_location: SourceLocation,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
        supports: Option<ImportSupportsCondition>,
        layer: ImportLayer,
    ) -> Arc<Locked<ImportRule>> {
        // Ensure the supports conditions for this @import are true, if not, refuse to load
        if supports.as_ref().is_some_and(|s| !s.enabled) {
            return Arc::new(lock.wrap(ImportRule {
                url,
                stylesheet: ImportSheet::new_refused(),
                supports,
                layer,
                source_location,
            }));
        }

        let resolved_url = match url.url().cloned() {
            Some(url) => url,
            None => {
                return Arc::new(lock.wrap(ImportRule {
                    url,
                    stylesheet: ImportSheet::new_refused(),
                    supports,
                    layer,
                    source_location,
                }));
            },
        };

        let import_rule = Arc::new(lock.wrap(ImportRule {
            url,
            stylesheet: ImportSheet::new_pending(),
            supports,
            layer,
            source_location,
        }));

        // TODO (mrnayak) : Whether we should use the original loader's CORS
        // setting? Fix this when spec has more details.
        let source = StylesheetContextSource::Import {
            import_rule: import_rule.clone(),
            media: Some(media.read_with(&lock.read()).clone()),
        };
        self.load(source, resolved_url.into(), None, "".to_owned());

        import_rule
    }
}
