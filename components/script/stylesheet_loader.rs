/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{Read, Seek, Write};

use base::id::PipelineId;
use crossbeam_channel::Sender;
use cssparser::SourceLocation;
use encoding_rs::UTF_8;
use net_traits::mime_classifier::MimeClassifier;
use net_traits::request::{CorsSettings, Destination, RequestId};
use net_traits::{
    FetchMetadata, FilteredMetadata, LoadContext, Metadata, NetworkError, ReferrerPolicy,
    ResourceFetchTiming,
};
use servo_arc::Arc;
use servo_config::pref;
use servo_url::ServoUrl;
use style::context::QuirksMode;
use style::global_style_data::STYLE_THREAD_POOL;
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
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::window::CSSErrorReporter;
use crate::fetch::{RequestWithGlobalScope, create_a_potential_cors_request};
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg};
use crate::network_listener::{self, FetchResponseListener, ResourceTimingListener};
use crate::script_runtime::{CanGc, ScriptThreadEventCategory};
use crate::task_source::TaskSourceName;
use crate::unminify::{
    BeautifyFileType, create_output_file, create_temp_files, execute_js_beautify,
};

pub(crate) trait StylesheetOwner {
    /// Returns whether this element was inserted by the parser (i.e., it should
    /// trigger a document-load-blocking load).
    fn parser_inserted(&self) -> bool;

    /// <https://html.spec.whatwg.org/multipage/#potentially-render-blocking>
    fn potentially_render_blocking(&self) -> bool;

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
    LinkElement,
    Import(Arc<Locked<ImportRule>>),
}

/// The context required for asynchronously loading an external stylesheet.
struct StylesheetContext {
    /// The element that initiated the request.
    element: Trusted<HTMLElement>,
    source: StylesheetContextSource,
    media: Arc<Locked<MediaList>>,
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
    /// <https://html.spec.whatwg.org/multipage/#contributes-a-script-blocking-style-sheet>
    is_script_blocking: bool,
    /// <https://html.spec.whatwg.org/multipage/#render-blocking>
    is_render_blocking: bool,
}

impl StylesheetContext {
    fn unminify_css(&mut self, file_url: ServoUrl) {
        let Some(unminified_dir) = self.document.root().window().unminified_css_dir() else {
            return;
        };

        let mut style_content = std::mem::take(&mut self.data);
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

        self.data = style_content;
    }

    fn empty_stylesheet(&self, document: &Document) -> Arc<Stylesheet> {
        let shared_lock = document.style_shared_lock().clone();
        let quirks_mode = document.quirks_mode();

        Arc::new(Stylesheet::from_bytes(
            &[],
            UrlExtraData(self.url.get_arc()),
            None,
            None,
            Origin::Author,
            self.media.clone(),
            shared_lock,
            None,
            None,
            quirks_mode,
        ))
    }

    fn parse(
        &self,
        quirks_mode: QuirksMode,
        shared_lock: SharedRwLock,
        css_error_reporter: &CSSErrorReporter,
        loader: ElementStylesheetLoader<'_>,
    ) -> Arc<Stylesheet> {
        let metadata = self
            .metadata
            .as_ref()
            .expect("Should never call parse without metadata.");

        let _span = profile_traits::trace_span!("ParseStylesheet").entered();
        Arc::new(Stylesheet::from_bytes(
            &self.data,
            UrlExtraData(metadata.final_url.get_arc()),
            metadata.charset.as_deref(),
            // The CSS environment encoding is the result of running the following steps: [CSSSYNTAX]
            // If el has a charset attribute, get an encoding from that attribute's value. If that succeeds, return the resulting encoding. [ENCODING]
            // Otherwise, return the document's character encoding. [DOM]
            //
            // TODO: Need to implement encoding http://dev.w3.org/csswg/css-syntax/#environment-encoding
            Some(UTF_8),
            Origin::Author,
            self.media.clone(),
            shared_lock,
            Some(&loader),
            Some(css_error_reporter),
            quirks_mode,
        ))
    }

    fn contributes_to_the_styling_processing_model(&self, element: &HTMLElement) -> bool {
        if !element.upcast::<Element>().is_connected() {
            return false;
        }

        // Whether or not this `StylesheetContext` is for a `<link>` element that comes
        // from a previous generation. This prevents processing of earlier stylsheet URLs
        // when the URL has changed.
        //
        // TODO(mrobinson): Shouldn't we also exit early if this is an import that was originally
        // imported from a `<link>` element that has advanced a generation as well?
        if !matches!(&self.source, StylesheetContextSource::LinkElement) {
            return true;
        }
        let link = element.downcast::<HTMLLinkElement>().unwrap();
        self.request_generation_id
            .is_none_or(|generation| generation == link.get_request_generation_id())
    }

    /// <https://html.spec.whatwg.org/multipage/#contributes-a-script-blocking-style-sheet>
    fn contributes_a_script_blocking_style_sheet(
        &self,
        element: &HTMLElement,
        owner: &dyn StylesheetOwner,
        document: &Document,
    ) -> bool {
        // el was created by that Document's parser.
        owner.parser_inserted()
        // el is either a style element or a link element that was an external resource link that
        // contributes to the styling processing model when the el was created by the parser.
        && element.downcast::<HTMLLinkElement>().is_none_or(|link|
            self.contributes_to_the_styling_processing_model(element)
            // el's style sheet was enabled when the element was created by the parser.
            && !link.is_effectively_disabled()
        )
        // el's media attribute's value matches the environment.
        && element.media_attribute_matches_media_environment()
        // The last time the event loop reached step 1, el's root was that Document.
        && *element.owner_document() == *document
        // The user agent hasn't given up on loading that particular style sheet yet.
        // A user agent may give up on loading a style sheet at any time.
        //
        // This might happen when we time out a resource, but that happens in `fetch` instead
    }

    fn decrement_load_and_render_blockers(&self, document: &Document) {
        if self.is_script_blocking {
            document.decrement_script_blocking_stylesheet_count();
        }

        if self.is_render_blocking {
            document.decrement_render_blocking_element_count();
        }
    }

    fn do_post_parse_tasks(self, success: bool, stylesheet: Arc<Stylesheet>) {
        let element = self.element.root();
        let document = self.document.root();
        let owner = element
            .upcast::<Element>()
            .as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");

        match &self.source {
            // https://html.spec.whatwg.org/multipage/#link-type-stylesheet%3Aprocess-the-linked-resource
            StylesheetContextSource::LinkElement => {
                let link = element
                    .downcast::<HTMLLinkElement>()
                    .expect("Should be HTMLinkElement due to StylesheetContextSource");
                // https://html.spec.whatwg.org/multipage/#link-type-stylesheet
                // > When the disabled attribute of a link element with a stylesheet keyword is set,
                // > disable the associated CSS style sheet.
                if link.is_effectively_disabled() {
                    stylesheet.set_disabled(true);
                }
                // Step 3. If el has an associated CSS style sheet, remove the CSS style sheet.
                // Step 4. If success is true, then:
                // Step 4.1. Create a CSS style sheet with the following properties:
                //
                // Note that even in the failure case, we should create an empty stylesheet.
                // That's why `set_stylesheet` also removes the previous stylesheet
                link.set_stylesheet(stylesheet);
            },
            StylesheetContextSource::Import(import_rule) => {
                // Construct a new WebFontDocumentContext for the stylesheet
                let window = element.owner_window();
                let document_context = window.web_font_context();

                // Layout knows about this stylesheet, because Stylo added it to the Stylist,
                // but Layout doesn't know about any new web fonts that it contains.
                document.load_web_fonts_from_stylesheet(&stylesheet, &document_context);

                let mut guard = document.style_shared_lock().write();
                import_rule.write_with(&mut guard).stylesheet = ImportSheet::Sheet(stylesheet);
            },
        }

        if let Some(ref shadow_root) = self.shadow_root {
            shadow_root.root().invalidate_stylesheets();
        } else {
            document.invalidate_stylesheets();
        }
        owner.set_origin_clean(self.origin_clean);

        // Remaining steps are a combination of
        // https://html.spec.whatwg.org/multipage/#link-type-stylesheet%3Aprocess-the-linked-resource
        // and https://html.spec.whatwg.org/multipage/#the-style-element%3Acritical-subresources

        // Step 4.2. Fire an event named load at el.
        // Step 5. Otherwise, fire an event named error at el.
        if let Some(any_failed) = owner.load_finished(success) {
            // Only fire an event if we have no more pending events
            // (in which case `owner.load_finished` would return None)
            let event = match any_failed {
                true => atom!("error"),
                false => atom!("load"),
            };
            element
                .upcast::<EventTarget>()
                .fire_event(event, CanGc::note());
        }
        // Regardless if there are other pending events, we need to unblock
        // rendering for this particular request and signal that the load has finished

        // Step 6. If el contributes a script-blocking style sheet, then:
        // Step 7. Unblock rendering on el.
        self.decrement_load_and_render_blockers(&document);
        document.finish_load(LoadType::Stylesheet(self.url), CanGc::note());
    }
}

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
        mut self,
        _: RequestId,
        status: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        network_listener::submit_timing(&self, &status, &timing, CanGc::note());

        let document = self.document.root();
        let Some(metadata) = self.metadata.as_ref() else {
            let empty_stylesheet = self.empty_stylesheet(&document);
            self.do_post_parse_tasks(false, empty_stylesheet);
            return;
        };

        let element = self.element.root();

        // https://html.spec.whatwg.org/multipage/#link-type-stylesheet:process-the-linked-resource
        if element.downcast::<HTMLLinkElement>().is_some() {
            // Step 1. If the resource's Content-Type metadata is not text/css, then set success to false.
            let is_css = MimeClassifier::is_css(
                &metadata.resource_content_type_metadata(LoadContext::Style, &self.data),
            ) || (
                // From <https://html.spec.whatwg.org/multipage/#link-type-stylesheet>:
                // > Quirk: If the document has been set to quirks mode, has the same origin as
                // > the URL of the external resource, and the Content-Type metadata of the
                // > external resource is not a supported style sheet type, the user agent must
                // > instead assume it to be text/css.
                document.quirks_mode() == QuirksMode::Quirks &&
                    document.origin().immutable().clone() == metadata.final_url.origin()
            );

            if !is_css {
                let empty_stylesheet = self.empty_stylesheet(&document);
                self.do_post_parse_tasks(false, empty_stylesheet);
                return;
            }

            // Step 2. If el no longer creates an external resource link that contributes to the styling processing model,
            // or if, since the resource in question was fetched, it has become appropriate to fetch it again, then:
            if !self.contributes_to_the_styling_processing_model(&element) {
                // Step 2.1. Remove el from el's node document's script-blocking style sheet set.
                self.decrement_load_and_render_blockers(&document);
                document.finish_load(LoadType::Stylesheet(self.url), CanGc::note());
                // Step 2.2. Return.
                return;
            }
        }

        if metadata.status != http::StatusCode::OK {
            let empty_stylesheet = self.empty_stylesheet(&document);
            self.do_post_parse_tasks(false, empty_stylesheet);
            return;
        }

        self.unminify_css(metadata.final_url.clone());

        let loader = if pref!(dom_parallel_css_parsing_enabled) {
            ElementStylesheetLoader::Asynchronous(AsynchronousStylesheetLoader::new(&element))
        } else {
            ElementStylesheetLoader::Synchronous { element: &element }
        };
        loader.parse(self, &element, &document);
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for StylesheetContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName(
            self.element
                .root()
                .upcast::<Element>()
                .local_name()
                .to_string(),
        );
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.element.root().owner_document().global()
    }
}

pub(crate) enum ElementStylesheetLoader<'a> {
    Synchronous { element: &'a HTMLElement },
    Asynchronous(AsynchronousStylesheetLoader),
}

impl<'a> ElementStylesheetLoader<'a> {
    pub(crate) fn new(element: &'a HTMLElement) -> Self {
        ElementStylesheetLoader::Synchronous { element }
    }
}

impl ElementStylesheetLoader<'_> {
    pub(crate) fn load_with_element(
        element: &HTMLElement,
        source: StylesheetContextSource,
        media: Arc<Locked<MediaList>>,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
        integrity_metadata: String,
    ) {
        let document = element.owner_document();
        let shadow_root = element
            .containing_shadow_root()
            .map(|shadow_root| Trusted::new(&*shadow_root));
        let generation = element
            .downcast::<HTMLLinkElement>()
            .map(HTMLLinkElement::get_request_generation_id);
        let mut context = StylesheetContext {
            element: Trusted::new(element),
            source,
            media,
            url: url.clone(),
            metadata: None,
            data: vec![],
            document: Trusted::new(&*document),
            shadow_root,
            origin_clean: true,
            request_generation_id: generation,
            is_script_blocking: false,
            is_render_blocking: false,
        };

        let owner = element
            .upcast::<Element>()
            .as_stylesheet_owner()
            .expect("Stylesheet not loaded by <style> or <link> element!");
        let referrer_policy = owner.referrer_policy();
        owner.increment_pending_loads_count();

        // Final steps of https://html.spec.whatwg.org/multipage/#update-a-style-block
        // and part of https://html.spec.whatwg.org/multipage/#link-type-stylesheet:linked-resource-fetch-setup-steps

        // If element contributes a script-blocking style sheet, append element to its node document's script-blocking style sheet set.
        context.is_script_blocking =
            context.contributes_a_script_blocking_style_sheet(element, owner, &document);
        if context.is_script_blocking {
            document.increment_script_blocking_stylesheet_count();
        }

        // If element's media attribute's value matches the environment and
        // element is potentially render-blocking, then block rendering on element.
        context.is_render_blocking = element.media_attribute_matches_media_environment() &&
            owner.potentially_render_blocking();
        if context.is_render_blocking {
            document.increment_render_blocking_element_count();
        }

        // https://html.spec.whatwg.org/multipage/#default-fetch-and-process-the-linked-resource
        let global = element.global();
        let request = create_a_potential_cors_request(
            Some(document.webview_id()),
            url.clone(),
            Destination::Style,
            cors_setting,
            None,
            global.get_referrer(),
        )
        .with_global_scope(&global)
        .referrer_policy(referrer_policy)
        .integrity_metadata(integrity_metadata);

        document.fetch(LoadType::Stylesheet(url), request, context);
    }

    fn parse(self, listener: StylesheetContext, element: &HTMLElement, document: &Document) {
        let shared_lock = document.style_shared_lock().clone();
        let quirks_mode = document.quirks_mode();
        let window = element.owner_window();

        match self {
            ElementStylesheetLoader::Synchronous { .. } => {
                let stylesheet =
                    listener.parse(quirks_mode, shared_lock, window.css_error_reporter(), self);
                listener.do_post_parse_tasks(true, stylesheet);
            },
            ElementStylesheetLoader::Asynchronous(asynchronous_loader) => {
                let css_error_reporter = window.css_error_reporter().clone();
                let thread_pool = STYLE_THREAD_POOL.pool();
                let thread_pool = thread_pool.as_ref().unwrap();

                thread_pool.spawn(move || {
                    let pipeline_id = asynchronous_loader.pipeline_id;
                    let main_thread_sender = asynchronous_loader.main_thread_sender.clone();

                    let loader = ElementStylesheetLoader::Asynchronous(asynchronous_loader);
                    let stylesheet =
                        listener.parse(quirks_mode, shared_lock, &css_error_reporter, loader);

                    let task = task!(finish_parsing_of_stylesheet_on_main_thread: move || {
                        listener.do_post_parse_tasks(true, stylesheet);
                    });

                    let _ = main_thread_sender.send(MainThreadScriptMsg::Common(
                        CommonScriptMsg::Task(
                            ScriptThreadEventCategory::StylesheetLoad,
                            Box::new(task),
                            Some(pipeline_id),
                            TaskSourceName::Networking,
                        ),
                    ));
                });
            },
        };
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
        let source = StylesheetContextSource::Import(import_rule.clone());

        match self {
            ElementStylesheetLoader::Synchronous { element } => {
                Self::load_with_element(
                    element,
                    source,
                    media,
                    resolved_url.into(),
                    None,
                    "".to_owned(),
                );
            },
            ElementStylesheetLoader::Asynchronous(AsynchronousStylesheetLoader {
                element,
                main_thread_sender,
                pipeline_id,
            }) => {
                let element = element.clone();
                let task = task!(load_import_stylesheet_on_main_thread: move || {
                    Self::load_with_element(
                        &element.root(),
                        source,
                        media,
                        resolved_url.into(),
                        None,
                        "".to_owned()
                    );
                });
                let _ =
                    main_thread_sender.send(MainThreadScriptMsg::Common(CommonScriptMsg::Task(
                        ScriptThreadEventCategory::StylesheetLoad,
                        Box::new(task),
                        Some(*pipeline_id),
                        TaskSourceName::Networking,
                    )));
            },
        }

        import_rule
    }
}

pub(crate) struct AsynchronousStylesheetLoader {
    element: Trusted<HTMLElement>,
    main_thread_sender: Sender<MainThreadScriptMsg>,
    pipeline_id: PipelineId,
}

impl AsynchronousStylesheetLoader {
    pub(crate) fn new(element: &HTMLElement) -> Self {
        let window = element.owner_window();
        Self {
            element: Trusted::new(element),
            main_thread_sender: window.main_thread_script_chan().clone(),
            pipeline_id: window.pipeline_id(),
        }
    }
}
