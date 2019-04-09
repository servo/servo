/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
use crate::dom::node::{document_from_node, window_from_node};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use crate::script_thread::MainThreadScriptMsg as Msg;
use crate::task::TaskOnce;
use crate::task_source::TaskSourceName;
use crossbeam_channel::Sender;
use cssparser::SourceLocation;
use encoding_rs::UTF_8;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use mime::{self, Mime};
use msg::constellation_msg::PipelineId;
use net_traits::request::{CorsSettings, CredentialsMode, Destination, RequestInit, RequestMode};
use net_traits::{
    FetchMetadata, FetchResponseListener, FilteredMetadata, Metadata, NetworkError, ReferrerPolicy,
};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use parking_lot::RwLock;
use servo_arc::Arc;
use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use std::mem;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use style::global_style_data::STYLE_THREAD_POOL;
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::import_rule::ImportSheet;
use style::stylesheets::StylesheetLoader as StyleStylesheetLoader;
use style::stylesheets::{
    CssRules, ImportRule, Namespaces, Origin, Stylesheet, StylesheetContents,
};
use style::values::CssUrl;

pub trait StylesheetOwner {
    /// Returns whether this element was inserted by the parser (i.e., it should
    /// trigger a document-load-blocking load).
    fn parser_inserted(&self) -> bool;

    /// Which referrer policy should loads triggered by this owner follow, or
    /// `None` for the default.
    fn referrer_policy(&self) -> Option<ReferrerPolicy>;

    /// Notes that a new load is pending to finish.
    fn increment_pending_loads_count(&self);

    /// Returns None if there are still pending loads, or whether any load has
    /// failed since the loads started.
    fn load_finished(&self, successful: bool) -> Option<bool>;

    /// Sets origin_clean flag.
    fn set_origin_clean(&self, origin_clean: bool);
}

#[derive(Clone)]
pub enum StylesheetSource {
    // NB: `MediaList` is just an option so we avoid cloning it by taking it out
    // of the option when passing it to `Stylesheet::from_bytes`.
    LinkElement(Option<MediaList>),
    Import(Arc<Stylesheet>),
}

/// The context required for asynchronously loading an external stylesheet.
/// Responses to the stylesheet fetch are processed in this context, such as the
/// initial metadata or when the file is completely received, in which case
/// parsing is kicked off, either on the script thread or on the thread pool.
pub struct StylesheetLoadContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLElement>,
    source: StylesheetSource,
    url: ServoUrl,
    metadata: Option<Metadata>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The node document for elem when the load was initiated.
    document: Trusted<Document>,
    origin_clean: bool,
    /// A token which must match the generation id of the `HTMLLinkElement` for
    /// it to load the stylesheet. This is ignored for `HTMLStyleElement` and
    /// imports.
    request_generation_id: Option<RequestGenerationId>,
    resource_timing: ResourceFetchTiming,
}

impl StylesheetLoadContext {
    fn parse_sync(&mut self, status: Result<ResourceFetchTiming, NetworkError>) {
        let elem = self.elem.root();
        let document = self.document.root();
        let mut successful = false;

        if status.is_ok() {
            let metadata = match self.metadata.take() {
                Some(meta) => meta,
                None => return,
            };
            let is_css = metadata.content_type.map_or(false, |ct| {
                let mime: Mime = ct.into_inner().into();
                mime.type_() == mime::TEXT && mime.subtype() == mime::CSS
            });

            let data = if is_css {
                mem::replace(&mut self.data, vec![])
            } else {
                vec![]
            };

            // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
            let environment_encoding = UTF_8;
            let protocol_encoding_label = metadata.charset.as_ref().map(|s| &**s);
            let final_url = metadata.final_url;

            let window = window_from_node(&*elem);

            let loader = StylesheetLoader::for_element(&elem);
            match self.source {
                StylesheetSource::LinkElement(ref mut media) => {
                    let link = elem.downcast::<HTMLLinkElement>().unwrap();
                    // We must first check whether the generations of the
                    // context and the element match up, else we risk applying
                    // the wrong stylesheet when responses come out-of-order.
                    let is_stylesheet_load_applicable = self
                        .request_generation_id
                        .map_or(true, |gen| gen == link.get_request_generation_id());
                    if is_stylesheet_load_applicable {
                        let shared_lock = document.style_shared_lock().clone();
                        let sheet = Arc::new(Stylesheet::from_bytes(
                            &data,
                            final_url,
                            protocol_encoding_label,
                            Some(environment_encoding),
                            Origin::Author,
                            media.take().unwrap(),
                            shared_lock,
                            Some(&loader),
                            window.css_error_reporter(),
                            document.quirks_mode(),
                        ));

                        if link.is_alternate() {
                            sheet.set_disabled(true);
                        }

                        link.set_stylesheet(sheet);
                    }
                },
                StylesheetSource::Import(ref stylesheet) => {
                    Stylesheet::update_from_bytes(
                        &stylesheet,
                        &data,
                        protocol_encoding_label,
                        Some(environment_encoding),
                        final_url,
                        Some(&loader),
                        window.css_error_reporter(),
                    );
                },
            }

            document.invalidate_stylesheets();

            // FIXME: Revisit once consensus is reached at:
            // https://github.com/whatwg/html/issues/1142
            successful = metadata.status.map_or(false, |(code, _)| code == 200);
        }

        post_parse_steps(
            document,
            elem,
            self.url.clone(),
            self.origin_clean,
            successful,
        );
    }

    fn parse_async(&mut self, status: Result<ResourceFetchTiming, NetworkError>) {
        let thread_pool = match STYLE_THREAD_POOL.style_thread_pool.as_ref() {
            Some(thread_pool) => thread_pool,
            // If we don't have a thread pool, fall back to sync parsing.
            None => return self.parse_sync(status),
        };

        let elem = self.elem.root();
        let document = self.document.root();

        if status.is_ok() {
            let metadata = match self.metadata.take() {
                Some(meta) => meta,
                None => return,
            };
            let is_css = metadata.content_type.map_or(false, |ct| {
                let mime: Mime = ct.into_inner().into();
                mime.type_() == mime::TEXT && mime.subtype() == mime::CSS
            });
            let data = if is_css {
                mem::replace(&mut self.data, vec![])
            } else {
                vec![]
            };

            // TODO: Get the actual value. http://dev.w3.org/csswg/css-syntax/#environment-encoding
            let environment_encoding = UTF_8;
            let final_url = metadata.final_url;
            // FIXME: Revisit once consensus is reached at:
            // https://github.com/whatwg/html/issues/1142
            let successful = metadata.status.map_or(false, |(code, _)| code == 200);
            let charset = metadata.charset;

            let pipeline_id = document.window().pipeline_id();
            let shared_lock = document.style_shared_lock().clone();
            let quirks_mode = document.quirks_mode();

            let trusted_elem = self.elem.clone();
            let script_chan = window_from_node(&*elem).main_thread_script_chan().clone();

            let origin_clean = self.origin_clean;
            let url = self.url.clone();

            let loader = ThreadSafeStylesheetLoader {
                elem: trusted_elem.clone(),
                script_chan: script_chan.clone(),
                pipeline_id: pipeline_id,
            };

            let document = self.document.clone();
            let request_generation_id = self.request_generation_id;

            // Can't clone `MediaList` or move out of `self`, so manually move or
            // clone sheet source as appropriate.
            let mut source = match self.source {
                StylesheetSource::LinkElement(ref mut media) => {
                    StylesheetSource::LinkElement(Some(media.take().unwrap()))
                },
                StylesheetSource::Import(ref mut sheet) => StylesheetSource::Import(sheet.clone()),
            };

            thread_pool.spawn(move || {
                let protocol_encoding_label = charset.as_ref().map(|s| &**s);
                let sheet = match source {
                    StylesheetSource::LinkElement(ref mut media) => {
                        Some(Arc::new(Stylesheet::from_bytes(
                            &data,
                            final_url,
                            protocol_encoding_label,
                            Some(environment_encoding),
                            Origin::Author,
                            media.take().unwrap(),
                            shared_lock,
                            Some(&loader),
                            // No error reporting in async parse mode.
                            None,
                            quirks_mode,
                        )))
                    },
                    StylesheetSource::Import(ref stylesheet) => {
                        Stylesheet::update_from_bytes(
                            &stylesheet,
                            &data,
                            protocol_encoding_label,
                            Some(environment_encoding),
                            final_url,
                            Some(&loader),
                            // No error reporting in async parse mode.
                            None,
                        );
                        // We already have a stylesheet for the import rule,
                        // so no need to create and return a new one.
                        None
                    },
                };

                // TODO(mandreyel): What to do with errors?
                let _ = script_chan.send(Msg::Common(CommonScriptMsg::Task(
                    ScriptThreadEventCategory::StylesheetLoad,
                    Box::new(FinishAsyncStylesheetLoadTask {
                        successful,
                        origin_clean,
                        top_level_stylesheet: sheet,
                        elem: trusted_elem,
                        document,
                        url,
                        request_generation_id,
                    }),
                    pipeline_id,
                    TaskSourceName::Networking,
                )));
            });
        } else {
            post_parse_steps(document, elem, self.url.clone(), self.origin_clean, false);
        }
    }
}

impl PreInvoke for StylesheetLoadContext {}

impl FetchResponseListener for StylesheetLoadContext {
    fn process_request_body(&mut self) {}

    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        if let Ok(FetchMetadata::Filtered { ref filtered, .. }) = metadata {
            match *filtered {
                FilteredMetadata::Opaque | FilteredMetadata::OpaqueRedirect => {
                    self.origin_clean = false;
                },
                _ => {},
            }
        }

        self.metadata = metadata.ok().map(|m| match m {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });
    }

    fn process_response_chunk(&mut self, mut payload: Vec<u8>) {
        self.data.append(&mut payload);
    }

    fn process_response_eof(&mut self, status: Result<ResourceFetchTiming, NetworkError>) {
        // TODO(mandreyel): This is a placeholder. How should this be set?
        let parse_async = PREFS
            .get("css-parsing.parallel")
            .as_boolean()
            //.unwrap_or(false);
            .unwrap_or(true); // TODO remove this is just for testing
        if parse_async {
            self.parse_async(status);
        } else {
            self.parse_sync(status);
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self)
    }
}

impl ResourceTimingListener for StylesheetLoadContext {
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
        document_from_node(&*self.elem.root()).global()
    }
}

/// This loader loads a stylesheet synchronously. It's used for top-level and
/// imported stylesheets alike. However, if parallel CSS parsing is enabled, the
/// `StylesheetLoadContext` receiving the fetch responses and starting the
/// parsing will use `AsyncStylesheetLoader` for any subsequent @import
/// stylesheet loading.
pub struct StylesheetLoader<'a> {
    elem: &'a HTMLElement,
}

impl<'a> StylesheetLoader<'a> {
    pub fn for_element(element: &'a HTMLElement) -> Self {
        StylesheetLoader { elem: element }
    }
}

impl<'a> StylesheetLoader<'a> {
    /// This is the entry point for initiating a load for the stylesheet located
    /// at `source`. Note that this must only be called from the main script
    /// thread, because fetches must be enqueued on a document.
    pub fn load(
        &self,
        source: StylesheetSource,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
        integrity_metadata: String,
    ) {
        start_stylesheet_load(self.elem, source, url, cors_setting, integrity_metadata);
    }
}

impl<'a> StyleStylesheetLoader for StylesheetLoader<'a> {
    /// Request a stylesheet after parsing a given `@import` rule, and return
    /// the constructed `@import` rule.
    fn request_stylesheet(
        &self,
        url: CssUrl,
        source_location: SourceLocation,
        context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>> {
        let sheet = Arc::new(Stylesheet {
            contents: StylesheetContents {
                rules: CssRules::new(Vec::new(), lock),
                origin: context.stylesheet_origin,
                url_data: RwLock::new(context.url_data.clone()),
                quirks_mode: context.quirks_mode,
                namespaces: RwLock::new(Namespaces::default()),
                source_map_url: RwLock::new(None),
                source_url: RwLock::new(None),
            },
            media: media,
            shared_lock: lock.clone(),
            disabled: AtomicBool::new(false),
        });

        let stylesheet = ImportSheet(sheet.clone());
        let import = ImportRule {
            url,
            source_location,
            stylesheet,
        };

        let url = match import.url.url().cloned() {
            Some(url) => url,
            None => return Arc::new(lock.wrap(import)),
        };

        // TODO (mrnayak) : Whether we should use the original loader's CORS
        // setting? Fix this when spec has more details.
        let source = StylesheetSource::Import(sheet.clone());
        self.load(source, url, None, "".to_owned());

        Arc::new(lock.wrap(import))
    }
}

/// This loader is used for loading import rules from another thread, but never
/// for loading top-level stylesheets, since those are always loaded by
/// `StylesheetLoader`.
struct ThreadSafeStylesheetLoader {
    elem: Trusted<HTMLElement>,
    script_chan: Sender<Msg>,
    pipeline_id: Option<PipelineId>,
}

impl StyleStylesheetLoader for ThreadSafeStylesheetLoader {
    /// Create and return an Arc to an empty import sheet and send script thread
    /// a task with a clone of this import sheet Arc to initiate the fetch for
    /// the stylesheet. Once it is imported and parsed, the empty import sheet
    /// will be updated with the newly parsed rules.
    fn request_stylesheet(
        &self,
        url: CssUrl,
        source_location: SourceLocation,
        context: &ParserContext,
        lock: &SharedRwLock,
        media: Arc<Locked<MediaList>>,
    ) -> Arc<Locked<ImportRule>> {
        let sheet = Arc::new(Stylesheet {
            contents: StylesheetContents {
                rules: CssRules::new(Vec::new(), lock),
                origin: context.stylesheet_origin,
                url_data: RwLock::new(context.url_data.clone()),
                quirks_mode: context.quirks_mode,
                namespaces: RwLock::new(Namespaces::default()),
                source_map_url: RwLock::new(None),
                source_url: RwLock::new(None),
            },
            media: media,
            shared_lock: lock.clone(),
            disabled: AtomicBool::new(false),
        });

        let stylesheet = ImportSheet(sheet.clone());
        let import = ImportRule {
            url,
            source_location,
            stylesheet,
        };

        let url = match import.url.url().cloned() {
            Some(url) => url,
            None => return Arc::new(lock.wrap(import)),
        };

        // TODO (mrnayak) : Whether we should use the original loader's CORS
        // setting? Fix this when spec has more details.
        let source = StylesheetSource::Import(sheet.clone());

        // Tell the event loop to import the stylesheet for us.
        // TODO(mandreyel): What to do with errors?
        let _ = self.script_chan.send(Msg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::StylesheetLoad,
            Box::new(ImportStylesheetTask {
                elem: self.elem.clone(),
                source,
                url,
            }),
            self.pipeline_id,
            TaskSourceName::Networking,
        )));

        Arc::new(lock.wrap(import))
    }
}

/// A stylesheet may be parsed on a thead pool instead of its document's event
/// loop. Since parsing a stylesheet may initiate further stylesheed loads (when
/// encountering @import rules at the start of the sheet) that need to be
/// initiated on the event loop, the thread encountering the import rule sends
/// this task to the script thread managing the document to start the load in
/// a thread-safe manner.
struct ImportStylesheetTask {
    elem: Trusted<HTMLElement>,
    source: StylesheetSource,
    url: ServoUrl,
}

impl TaskOnce for ImportStylesheetTask {
    fn run_once(self) {
        let elem = self.elem.root();
        start_stylesheet_load(&*elem, self.source, self.url, None, "".to_owned());
    }
}

/// A stylesheet may be parsed on a thread pool rather than its document's event
/// loop. However, setting the parsed stylesheet on the HTML element and the
/// document needs to be done on the event loop, thus when the thread parsing
/// the sheet is done it enqueues this task on the script thread's task queue
/// that's managing the document.
///
/// This task is used for both top-level as well as nested stylesheet loads. For
/// top-level sheets the `top_level_stylesheet` field must be set, but parsing
/// an imported sheet (in loader's `request_stylesheet`) returns an empty sheet
/// wrapped in an Arc in an `ImportRule` (which is inserted in its parent's rule
/// list), and after an async fetch and parse, the empty stylesheet is set
/// through the Arc reference.
///
/// This task is only enqueued on the event loop if the load was successful.
struct FinishAsyncStylesheetLoadTask {
    /// Used to tell the document whether the load has concluded successfully.
    successful: bool,
    origin_clean: bool,
    /// If this was a top-level stylesheet load, the parsed stylesheet is placed
    /// here.
    top_level_stylesheet: Option<Arc<Stylesheet>>,
    /// The element for which the stylesheet was loaded.
    elem: Trusted<HTMLElement>,
    /// The document for which the stylesheet was loaded.
    document: Trusted<Document>,
    /// The URL of the stylesheet.
    url: ServoUrl,
    /// A token which must match the generation id of the `HTMLLinkElement` for
    /// it to load the stylesheet. This is ignored for `HTMLStyleElement` and
    /// imports. This is done after parsing because checking against this token
    /// before spawning the parser thread is liable to introduce race
    /// conditions.
    request_generation_id: Option<RequestGenerationId>,
}

impl TaskOnce for FinishAsyncStylesheetLoadTask {
    /// Set the parsed stylesheet (if any) on the link element if this was
    /// a load for a top-level stylesheet. If this was the last missing
    /// stylesheet for the document, fire a load event.
    fn run_once(self) {
        let elem = self.elem.root();
        let document = self.document.root();

        if let Some(sheet) = self.top_level_stylesheet {
            let link = elem.downcast::<HTMLLinkElement>().unwrap();
            // We must first check whether the generations of the context and
            // the element match up, else we risk applying the wrong stylesheet
            // when responses come out-of-order.
            let is_stylesheet_load_applicable = self
                .request_generation_id
                .map_or(true, |gen| gen == link.get_request_generation_id());
            if is_stylesheet_load_applicable {
                if link.is_alternate() {
                    sheet.set_disabled(true);
                }
                link.set_stylesheet(sheet);
            }
        }

        document.invalidate_stylesheets();

        post_parse_steps(
            document,
            elem,
            self.url.clone(),
            self.origin_clean,
            self.successful,
        );
    }
}

fn post_parse_steps(
    document: DomRoot<Document>,
    elem: DomRoot<HTMLElement>,
    url: ServoUrl,
    origin_clean: bool,
    successful: bool,
) {
    let owner = elem
        .upcast::<Element>()
        .as_stylesheet_owner()
        .expect("Stylesheet not loaded by <style> or <link> element!");
    owner.set_origin_clean(origin_clean);
    if owner.parser_inserted() {
        document.decrement_script_blocking_stylesheet_count();
    }

    document.finish_load(LoadType::Stylesheet(url));

    if let Some(any_failed) = owner.load_finished(successful) {
        let event = if any_failed {
            atom!("error")
        } else {
            atom!("load")
        };
        elem.upcast::<EventTarget>().fire_event(event);
    }
}

/// Since both `StylesheetLoader` and `ImportStylesheetTask` are used for
/// initiating a stylesheet load in the same way, the preocedure is extraced
/// here. This function sets up the load context, channels to communicate fetch
/// responses, the request object, and issues an async fetch on the element's
/// document.
fn start_stylesheet_load(
    elem: &HTMLElement,
    source: StylesheetSource,
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    integrity_metadata: String,
) {
    let document = document_from_node(elem);
    let gen = elem
        .downcast::<HTMLLinkElement>()
        .map(HTMLLinkElement::get_request_generation_id);
    let context = ::std::sync::Arc::new(Mutex::new(StylesheetLoadContext {
        elem: Trusted::new(&*elem),
        source: source,
        url: url.clone(),
        metadata: None,
        data: vec![],
        document: Trusted::new(&*document),
        origin_clean: true,
        request_generation_id: gen,
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
    }));

    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let (task_source, canceller) = document
        .window()
        .task_manager()
        .networking_task_source_with_canceller();
    let listener = NetworkListener {
        context,
        task_source,
        canceller: Some(canceller),
    };
    ROUTER.add_route(
        action_receiver.to_opaque(),
        Box::new(move |message| {
            listener.notify_fetch(message.to().unwrap());
        }),
    );

    let owner = elem
        .upcast::<Element>()
        .as_stylesheet_owner()
        .expect("Stylesheet not loaded by <style> or <link> element!");
    let referrer_policy = owner
        .referrer_policy()
        .or_else(|| document.get_referrer_policy());
    owner.increment_pending_loads_count();
    if owner.parser_inserted() {
        document.increment_script_blocking_stylesheet_count();
    }

    let request = RequestInit {
        url: url.clone(),
        destination: Destination::Style,
        // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
        // Step 1
        mode: match cors_setting {
            Some(_) => RequestMode::CorsMode,
            None => RequestMode::NoCors,
        },
        // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
        // Step 3-4
        credentials_mode: match cors_setting {
            Some(CorsSettings::Anonymous) => CredentialsMode::CredentialsSameOrigin,
            _ => CredentialsMode::Include,
        },
        origin: document.origin().immutable().clone(),
        pipeline_id: Some(elem.global().pipeline_id()),
        referrer_url: Some(document.url()),
        referrer_policy: referrer_policy,
        integrity_metadata: integrity_metadata,
        ..RequestInit::default()
    };

    document.fetch_async(LoadType::Stylesheet(url), request, action_sender);
}
