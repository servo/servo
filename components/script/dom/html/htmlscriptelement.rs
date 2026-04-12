/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::rc::Rc;

use dom_struct::dom_struct;
use encoding_rs::Encoding;
use html5ever::{LocalName, Prefix, local_name};
use js::context::JSContext;
use js::rust::{HandleObject, Stencil};
use net_traits::http_status::HttpStatus;
use net_traits::request::{
    CorsSettings, Destination, ParserMetadata, Referrer, RequestBuilder, RequestId,
};
use net_traits::{FetchMetadata, Metadata, NetworkError, ResourceFetchTiming};
use servo_base::id::WebViewId;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::str::{HTML_SPACE_CHARACTERS, StaticStringVec};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMTokenListBinding::DOMTokenListMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    TrustedScriptOrString, TrustedScriptURLOrUSVString,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::{CspReporting, GlobalCspReporting, InlineCheckType, Violation};
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{
    AttributeMutation, Element, ElementCreator, cors_setting_for_element,
    cors_settings_attribute_credential_mode, referrer_policy_for_element,
    reflect_cross_origin_attribute, reflect_referrer_policy_attribute, set_cross_origin_attribute,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::global_scope_script_execution::{ClassicScript, ErrorReporting, RethrowErrors};
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{ChildrenMutation, CloneChildrenFlag, Node, NodeTraits, UnbindContext};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::trustedtypes::trustedscript::TrustedScript;
use crate::dom::trustedtypes::trustedscripturl::TrustedScriptURL;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::fetch::{RequestWithGlobalScope, create_a_potential_cors_request};
use crate::network_listener::{self, FetchResponseListener, ResourceTimingListener};
use crate::script_module::{
    ImportMap, ModuleOwner, ModuleTree, ScriptFetchOptions, fetch_an_external_module_script,
    fetch_inline_module_script, parse_an_import_map_string, register_import_map,
};
use crate::script_runtime::{CanGc, IntroductionType};

/// An unique id for script element.
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq, MallocSizeOf)]
pub(crate) struct ScriptId(#[no_trace] Uuid);

#[dom_struct]
pub(crate) struct HTMLScriptElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#concept-script-delay-load>
    delaying_the_load_event: DomRefCell<Option<LoadBlocker>>,

    /// <https://html.spec.whatwg.org/multipage/#already-started>
    already_started: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#parser-inserted>
    parser_inserted: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#non-blocking>
    ///
    /// (currently unused)
    non_blocking: Cell<bool>,

    /// Document of the parser that created this element
    /// <https://html.spec.whatwg.org/multipage/#parser-document>
    parser_document: Dom<Document>,

    /// Prevents scripts that move between documents during preparation from executing.
    /// <https://html.spec.whatwg.org/multipage/#preparation-time-document>
    preparation_time_document: MutNullableDom<Document>,

    /// Track line line_number
    line_number: u64,

    /// Unique id for each script element
    #[ignore_malloc_size_of = "Defined in uuid"]
    id: ScriptId,

    /// <https://w3c.github.io/trusted-types/dist/spec/#htmlscriptelement-script-text>
    script_text: DomRefCell<DOMString>,

    /// <https://html.spec.whatwg.org/multipage/#concept-script-external>
    from_an_external_file: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#dom-script-blocking>
    blocking: MutNullableDom<DOMTokenList>,

    /// Used to keep track whether we consider this script element render blocking during
    /// `prepare`
    marked_as_render_blocking: Cell<bool>,
}

impl HTMLScriptElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        creator: ElementCreator,
    ) -> HTMLScriptElement {
        HTMLScriptElement {
            id: ScriptId(Uuid::new_v4()),
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            already_started: Cell::new(false),
            delaying_the_load_event: Default::default(),
            parser_inserted: Cell::new(creator.is_parser_created()),
            non_blocking: Cell::new(!creator.is_parser_created()),
            parser_document: Dom::from_ref(document),
            preparation_time_document: MutNullableDom::new(None),
            line_number: creator.return_line_number(),
            script_text: DomRefCell::new(DOMString::new()),
            from_an_external_file: Cell::new(false),
            blocking: Default::default(),
            marked_as_render_blocking: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
    ) -> DomRoot<HTMLScriptElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLScriptElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
        )
    }

    pub(crate) fn get_script_id(&self) -> ScriptId {
        self.id
    }

    /// Marks that element as delaying the load event or not.
    ///
    /// Nothing happens if the element was already delaying the load event and
    /// we pass true to that method again.
    ///
    /// <https://html.spec.whatwg.org/multipage/#concept-script-delay-load>
    /// <https://html.spec.whatwg.org/multipage/#delaying-the-load-event-flag>
    pub(crate) fn delay_load_event(&self, url: ServoUrl) {
        let document = self.get_script_active_document();

        let blocker = &self.delaying_the_load_event;
        if blocker.borrow().is_none() {
            *blocker.borrow_mut() = Some(LoadBlocker::new(&document, LoadType::Script(url)));
        }
    }

    /// Helper method to determine the script kind based on attributes and insertion context.
    ///
    /// This duplicates the script preparation logic from the HTML spec to determine the
    /// script's active document without full preparation.
    ///
    /// <https://html.spec.whatwg.org/multipage/#prepare-the-script-element>
    pub(crate) fn get_script_kind(&self) -> ExternalScriptKind {
        let element = self.upcast::<Element>();
        let was_parser_inserted = self.parser_inserted.get();
        let asynch = element.has_attribute(&local_name!("async"));
        let mut script_kind = ExternalScriptKind::Asap;

        match self.get_script_type() {
            Some(ScriptType::Classic) => {
                if element.has_attribute(&local_name!("defer")) && was_parser_inserted && !asynch {
                    script_kind = ExternalScriptKind::Deferred
                } else if was_parser_inserted && !asynch {
                    script_kind = ExternalScriptKind::ParsingBlocking
                } else if !asynch && !self.non_blocking.get() {
                    script_kind = ExternalScriptKind::AsapInOrder
                }
            },
            Some(ScriptType::Module) => {
                if !asynch && was_parser_inserted {
                    script_kind = ExternalScriptKind::Deferred
                } else if !asynch && !self.non_blocking.get() {
                    script_kind = ExternalScriptKind::AsapInOrder
                }
            },
            Some(ScriptType::ImportMap) => (),
            None => (),
        }

        script_kind
    }

    /// <https://html.spec.whatwg.org/multipage/#prepare-the-script-element>
    pub(crate) fn get_script_active_document(&self) -> DomRoot<Document> {
        let script_kind = self.get_script_kind();
        match script_kind {
            ExternalScriptKind::Asap => self.preparation_time_document.get().unwrap(),
            ExternalScriptKind::AsapInOrder => self.preparation_time_document.get().unwrap(),
            ExternalScriptKind::Deferred => self.parser_document.as_rooted(),
            ExternalScriptKind::ParsingBlocking => self.parser_document.as_rooted(),
        }
    }
}

/// Supported script types as defined by
/// <https://html.spec.whatwg.org/multipage/#javascript-mime-type>.
pub(crate) static SCRIPT_JS_MIMES: StaticStringVec = &[
    "application/ecmascript",
    "application/javascript",
    "application/x-ecmascript",
    "application/x-javascript",
    "text/ecmascript",
    "text/javascript",
    "text/javascript1.0",
    "text/javascript1.1",
    "text/javascript1.2",
    "text/javascript1.3",
    "text/javascript1.4",
    "text/javascript1.5",
    "text/jscript",
    "text/livescript",
    "text/x-ecmascript",
    "text/x-javascript",
];

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ScriptType {
    Classic,
    Module,
    ImportMap,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct CompiledSourceCode {
    #[ignore_malloc_size_of = "SM handles JS values"]
    pub(crate) source_code: Stencil,
    #[conditional_malloc_size_of = "Rc is hard"]
    pub(crate) original_text: Rc<DOMString>,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum SourceCode {
    Text(#[conditional_malloc_size_of] Rc<DOMString>),
    Compiled(CompiledSourceCode),
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ScriptOrigin {
    pub code: SourceCode,
    #[no_trace]
    pub url: ServoUrl,
    external: bool,
    pub fetch_options: ScriptFetchOptions,
    type_: ScriptType,
    unminified_dir: Option<String>,
    import_map: Fallible<ImportMap>,
}

impl ScriptOrigin {
    pub(crate) fn internal(
        text: Rc<DOMString>,
        url: ServoUrl,
        fetch_options: ScriptFetchOptions,
        type_: ScriptType,
        unminified_dir: Option<String>,
        import_map: Fallible<ImportMap>,
    ) -> ScriptOrigin {
        ScriptOrigin {
            code: SourceCode::Text(text),
            url,
            external: false,
            fetch_options,
            type_,
            unminified_dir,
            import_map,
        }
    }

    pub(crate) fn external(
        text: Rc<DOMString>,
        url: ServoUrl,
        fetch_options: ScriptFetchOptions,
        type_: ScriptType,
        unminified_dir: Option<String>,
    ) -> ScriptOrigin {
        ScriptOrigin {
            code: SourceCode::Text(text),
            url,
            external: true,
            fetch_options,
            type_,
            unminified_dir,
            import_map: Err(Error::NotFound(None)),
        }
    }

    pub(crate) fn text(&self) -> Rc<DOMString> {
        match &self.code {
            SourceCode::Text(text) => Rc::clone(text),
            SourceCode::Compiled(compiled_script) => Rc::clone(&compiled_script.original_text),
        }
    }
}

/// Final steps of <https://html.spec.whatwg.org/multipage/#prepare-the-script-element>
pub(crate) fn finish_fetching_a_script(
    elem: &HTMLScriptElement,
    script_kind: ExternalScriptKind,
    load: ScriptResult,
    cx: &mut js::context::JSContext,
) {
    // Step 33. The "steps to run when the result is ready" for each type of script in 33.2-33.5.
    // of https://html.spec.whatwg.org/multipage/#prepare-the-script-element
    let document;

    match script_kind {
        ExternalScriptKind::Asap => {
            document = elem.preparation_time_document.get().unwrap();
            document.asap_script_loaded(cx, elem, load)
        },
        ExternalScriptKind::AsapInOrder => {
            document = elem.preparation_time_document.get().unwrap();
            document.asap_in_order_script_loaded(cx, elem, load)
        },
        ExternalScriptKind::Deferred => {
            document = elem.parser_document.as_rooted();
            document.deferred_script_loaded(cx, elem, load);
        },
        ExternalScriptKind::ParsingBlocking => {
            document = elem.parser_document.as_rooted();
            document.pending_parsing_blocking_script_loaded(elem, load, cx);
        },
    }

    // <https://html.spec.whatwg.org/multipage/#steps-to-run-when-the-result-is-ready>
    // Step 4
    LoadBlocker::terminate(&elem.delaying_the_load_event, cx);
}

pub(crate) type ScriptResult = Result<Script, ()>;

// TODO merge classic and module scripts
#[derive(JSTraceable, MallocSizeOf)]
#[expect(clippy::large_enum_variant)]
pub(crate) enum Script {
    Classic(ClassicScript),
    Module(#[conditional_malloc_size_of] Rc<ModuleTree>),
    ImportMap(ScriptOrigin),
}

/// The context required for asynchronously loading an external script source.
struct ClassicContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLScriptElement>,
    /// The kind of external script.
    kind: ExternalScriptKind,
    /// The (fallback) character encoding argument to the "fetch a classic
    /// script" algorithm.
    character_encoding: &'static Encoding,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The initial URL requested.
    url: ServoUrl,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// The fetch options of the script
    fetch_options: ScriptFetchOptions,
    /// Used to set muted errors flag of classic scripts
    response_was_cors_cross_origin: bool,
}

impl FetchResponseListener for ClassicContext {
    // TODO(KiChjang): Perhaps add custom steps to perform fetch here?
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: &mut js::context::JSContext,
        _: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        self.metadata = metadata.ok().map(|meta| {
            self.response_was_cors_cross_origin = meta.is_cors_cross_origin();
            match meta {
                FetchMetadata::Unfiltered(m) => m,
                FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
            }
        });

        let status = self
            .metadata
            .as_ref()
            .map(|m| m.status.clone())
            .unwrap_or_else(HttpStatus::new_error);

        self.status = {
            if status.is_error() {
                Err(NetworkError::ResourceLoadError(
                    "No http status code received".to_owned(),
                ))
            } else if status.is_success() {
                Ok(())
            } else {
                Err(NetworkError::ResourceLoadError(format!(
                    "HTTP error code {}",
                    status.code()
                )))
            }
        };
    }

    fn process_response_chunk(
        &mut self,
        _: &mut js::context::JSContext,
        _: RequestId,
        mut chunk: Vec<u8>,
    ) {
        if self.status.is_ok() {
            self.data.append(&mut chunk);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
    /// step 4-9
    fn process_response_eof(
        mut self,
        cx: &mut js::context::JSContext,
        _: RequestId,
        response: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        match (response.as_ref(), self.status.as_ref()) {
            (Err(error), _) | (_, Err(error)) => {
                error!("Fetching classic script failed {:?}", error);
                // Step 6, response is an error.
                finish_fetching_a_script(&self.elem.root(), self.kind, Err(()), cx);

                // Resource timing is expected to be available before "error" or "load" events are fired.
                network_listener::submit_timing(cx, &self, &response, &timing);
                return;
            },
            _ => {},
        };

        let metadata = self.metadata.take().unwrap();
        let final_url = metadata.final_url;

        // Step 5.3. Let potentialMIMETypeForEncoding be the result of extracting a MIME type given response's header list.
        // Step 5.4. Set encoding to the result of legacy extracting an encoding given potentialMIMETypeForEncoding and encoding.
        let encoding = metadata
            .charset
            .and_then(|encoding| Encoding::for_label(encoding.as_bytes()))
            .unwrap_or(self.character_encoding);

        // Step 5.5. Let sourceText be the result of decoding bodyBytes to Unicode, using encoding as the fallback encoding.
        let (mut source_text, _, _) = encoding.decode(&self.data);

        let elem = self.elem.root();
        let global = elem.global();

        if let Some(window) = global.downcast::<Window>() {
            substitute_with_local_script(window, &mut source_text, final_url.clone());
        }

        // Step 5.6. Let mutedErrors be true if response was CORS-cross-origin, and false otherwise.
        let muted_errors = self.response_was_cors_cross_origin;

        // Step 5.7. Let script be the result of creating a classic script given
        // sourceText, settingsObject, response's URL, options, mutedErrors, and url.
        let script = global.create_a_classic_script(
            cx,
            source_text,
            final_url,
            self.fetch_options.clone(),
            ErrorReporting::from(muted_errors),
            Some(IntroductionType::SRC_SCRIPT),
            1,
            true,
        );

        /*
        let options = unsafe { CompileOptionsWrapper::new(*cx, final_url.as_str(), 1) };

        let can_compile_off_thread = pref!(dom_script_asynch) &&
            unsafe { CanCompileOffThread(*cx, options.ptr as *const _, source_text.len()) };

        if can_compile_off_thread {
            let source_string = source_text.to_string();

            let context = Box::new(OffThreadCompilationContext {
                script_element: self.elem.clone(),
                script_kind: self.kind,
                final_url,
                url: self.url.clone(),
                task_source: elem.owner_global().task_manager().dom_manipulation_task_source(),
                script_text: source_string,
                fetch_options: self.fetch_options.clone(),
            });

            unsafe {
                assert!(!CompileToStencilOffThread1(
                    *cx,
                    options.ptr as *const _,
                    &mut transform_str_to_source_text(&context.script_text) as *mut _,
                    Some(off_thread_compilation_callback),
                    Box::into_raw(context) as *mut c_void,
                )
                .is_null());
            }
        } else {*/
        let load = Script::Classic(script);
        finish_fetching_a_script(&elem, self.kind, Ok(load), cx);
        // }

        network_listener::submit_timing(cx, &self, &response, &timing);
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        let elem = self.elem.root();
        global.report_csp_violations(violations, Some(elem.upcast()), None);
    }
}

impl ResourceTimingListener for ClassicContext {
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

/// Steps 1-2 of <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
// This function is also used to prefetch a script in `script::dom::servoparser::prefetch`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn script_fetch_request(
    webview_id: WebViewId,
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    options: ScriptFetchOptions,
    referrer: Referrer,
) -> RequestBuilder {
    // We intentionally ignore options' credentials_mode member for classic scripts.
    // The mode is initialized by create_a_potential_cors_request.
    create_a_potential_cors_request(
        Some(webview_id),
        url,
        Destination::Script,
        cors_setting,
        None,
        referrer,
    )
    .parser_metadata(options.parser_metadata)
    .integrity_metadata(options.integrity_metadata.clone())
    .referrer_policy(options.referrer_policy)
    .cryptographic_nonce_metadata(options.cryptographic_nonce)
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
fn fetch_a_classic_script(
    script: &HTMLScriptElement,
    kind: ExternalScriptKind,
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    options: ScriptFetchOptions,
    character_encoding: &'static Encoding,
) {
    // Step 1, 2.
    let doc = script.owner_document();
    let global = script.global();
    let referrer = global.get_referrer();
    let request = script_fetch_request(
        doc.webview_id(),
        url.clone(),
        cors_setting,
        options.clone(),
        referrer,
    )
    .with_global_scope(&global);

    // TODO: Step 3, Add custom steps to perform fetch

    let context = ClassicContext {
        elem: Trusted::new(script),
        kind,
        character_encoding,
        data: vec![],
        metadata: None,
        url,
        status: Ok(()),
        fetch_options: options,
        response_was_cors_cross_origin: false,
    };
    doc.fetch_background(request, context);
}

impl HTMLScriptElement {
    /// <https://w3c.github.io/trusted-types/dist/spec/#setting-slot-values-from-parser>
    pub(crate) fn set_initial_script_text(&self) {
        *self.script_text.borrow_mut() = self.text();
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#abstract-opdef-prepare-the-script-text>
    fn prepare_the_script_text(&self, cx: &mut JSContext) -> Fallible<()> {
        // Step 1. If script’s script text value is not equal to its child text content,
        // set script’s script text to the result of executing
        // Get Trusted Type compliant string, with the following arguments:
        if self.script_text.borrow().clone() != self.text() {
            *self.script_text.borrow_mut() = TrustedScript::get_trusted_type_compliant_string(
                cx,
                &self.owner_global(),
                self.Text(),
                "HTMLScriptElement text",
            )?;
        }

        Ok(())
    }

    fn has_render_blocking_attribute(&self) -> bool {
        self.blocking
            .get()
            .is_some_and(|list| list.Contains("render".into()))
    }

    /// <https://html.spec.whatwg.org/multipage/#potentially-render-blocking>
    fn potentially_render_blocking(&self) -> bool {
        // An element is potentially render-blocking if its blocking tokens set contains "render",
        // or if it is implicitly potentially render-blocking, which will be defined at the individual elements.
        // By default, an element is not implicitly potentially render-blocking.
        if self.has_render_blocking_attribute() {
            return true;
        }
        let element = self.upcast::<Element>();
        // https://html.spec.whatwg.org/multipage/#script-processing-model:implicitly-potentially-render-blocking
        // > A script element el is implicitly potentially render-blocking if el's type is "classic",
        // > el is parser-inserted, and el does not have an async or defer attribute.
        self.get_script_type()
            .is_some_and(|script_type| script_type == ScriptType::Classic) &&
            self.parser_inserted.get() &&
            !element.has_attribute(&local_name!("async")) &&
            !element.has_attribute(&local_name!("defer"))
    }

    /// <https://html.spec.whatwg.org/multipage/#prepare-the-script-element>
    pub(crate) fn prepare(
        &self,
        cx: &mut JSContext,
        introduction_type_override: Option<&'static CStr>,
    ) {
        let introduction_type =
            introduction_type_override.or(Some(IntroductionType::INLINE_SCRIPT));

        // Step 1. If el's already started is true, then return.
        if self.already_started.get() {
            return;
        }

        // Step 2. Let parser document be el's parser document.
        // TODO

        // Step 3. Set el's parser document to null.
        let was_parser_inserted = self.parser_inserted.get();
        self.parser_inserted.set(false);

        // Step 4.
        // If parser document is non-null and el does not have an async attribute, then set el's force async to true.
        let element = self.upcast::<Element>();
        let asynch = element.has_attribute(&local_name!("async"));
        // Note: confusingly, this is done if the element does *not* have an "async" attribute.
        if was_parser_inserted && !asynch {
            self.non_blocking.set(true);
        }

        // Step 5. Execute the Prepare the script text algorithm on el.
        // If that algorithm threw an error, then return.
        if self.prepare_the_script_text(cx).is_err() {
            return;
        }
        // Step 5a. Let source text be el’s script text value.
        let text = self.script_text.borrow().clone();
        // Step 6. If el has no src attribute, and source text is the empty string, then return.
        if text.is_empty() && !element.has_attribute(&local_name!("src")) {
            return;
        }

        // Step 7. If el is not connected, then return.
        if !self.upcast::<Node>().is_connected() {
            return;
        }

        let script_type = if let Some(ty) = self.get_script_type() {
            // Step 9-11.
            ty
        } else {
            // Step 12. Otherwise, return. (No script is executed, and el's type is left as null.)
            return;
        };

        // Step 13.
        // If parser document is non-null, then set el's parser document back to parser document and set el's force
        // async to false.
        if was_parser_inserted {
            self.parser_inserted.set(true);
            self.non_blocking.set(false);
        }

        // Step 14. Set el's already started to true.
        self.already_started.set(true);

        // Step 15. Set el's preparation-time document to its node document.
        let doc = self.owner_document();
        self.preparation_time_document.set(Some(&doc));

        // Step 16.
        // If parser document is non-null, and parser document is not equal to el's preparation-time document, then
        // return.
        if self.parser_inserted.get() && *self.parser_document != *doc {
            return;
        }

        // Step 17. If scripting is disabled for el, then return.
        if !doc.scripting_enabled() {
            return;
        }

        // Step 18. If el has a nomodule content attribute and its type is "classic", then return.
        if element.has_attribute(&local_name!("nomodule")) && script_type == ScriptType::Classic {
            return;
        }

        let global = &doc.global();

        // Step 19. CSP.
        if !element.has_attribute(&local_name!("src")) &&
            global
                .get_csp_list()
                .should_elements_inline_type_behavior_be_blocked(
                    global,
                    element,
                    InlineCheckType::Script,
                    &text.str(),
                    self.line_number as u32,
                )
        {
            warn!("Blocking inline script due to CSP");
            return;
        }

        // Step 20. If el has an event attribute and a for attribute, and el's type is "classic", then:
        if script_type == ScriptType::Classic {
            let for_attribute = element.get_attribute(&local_name!("for"));
            let event_attribute = element.get_attribute(&local_name!("event"));
            if let (Some(ref for_attribute), Some(ref event_attribute)) =
                (for_attribute, event_attribute)
            {
                let for_value = for_attribute.value().to_ascii_lowercase();
                let for_value = for_value.trim_matches(HTML_SPACE_CHARACTERS);
                if for_value != "window" {
                    return;
                }

                let event_value = event_attribute.value().to_ascii_lowercase();
                let event_value = event_value.trim_matches(HTML_SPACE_CHARACTERS);
                if event_value != "onload" && event_value != "onload()" {
                    return;
                }
            }
        }

        // Step 21. If el has a charset attribute, then let encoding be the result of getting
        // an encoding from the value of the charset attribute.
        // If el does not have a charset attribute, or if getting an encoding failed,
        // then let encoding be el's node document's the encoding.
        let encoding = element
            .get_attribute(&local_name!("charset"))
            .and_then(|charset| Encoding::for_label(charset.value().as_bytes()))
            .unwrap_or_else(|| doc.encoding());

        // Step 22. CORS setting.
        let cors_setting = cors_setting_for_element(element);

        // Step 23. Let module script credentials mode be the CORS settings attribute credentials mode for el's crossorigin content attribute.
        let module_credentials_mode = cors_settings_attribute_credential_mode(element);

        // Step 24. Let cryptographic nonce be el's [[CryptographicNonce]] internal slot's value.
        // If the element has a nonce content attribute but is not nonceable strip the nonce to prevent injection attacks.
        // Elements without a nonce content attribute (e.g. JS-created with .nonce = "abc")
        // use the internal slot directly — the nonceable check only applies to parser-created elements.
        let el = self.upcast::<Element>();
        let cryptographic_nonce = if el.is_nonceable() || !el.has_attribute(&local_name!("nonce")) {
            el.nonce_value().trim().to_owned()
        } else {
            String::new()
        };

        // Step 25. If el has an integrity attribute, then let integrity metadata be that attribute's value.
        // Otherwise, let integrity metadata be the empty string.
        let im_attribute = element.get_attribute(&local_name!("integrity"));
        let integrity_val = im_attribute.as_ref().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        // Step 26. Let referrer policy be the current state of el's referrerpolicy content attribute.
        let referrer_policy = referrer_policy_for_element(self.upcast::<Element>());

        // TODO: Step 27. Fetch priority.

        // Step 28. Let parser metadata be "parser-inserted" if el is parser-inserted,
        // and "not-parser-inserted" otherwise.
        let parser_metadata = if self.parser_inserted.get() {
            ParserMetadata::ParserInserted
        } else {
            ParserMetadata::NotParserInserted
        };

        // Step 29. Fetch options.
        let mut options = ScriptFetchOptions {
            cryptographic_nonce,
            integrity_metadata: integrity_metadata.to_owned(),
            parser_metadata,
            referrer_policy,
            credentials_mode: module_credentials_mode,
            render_blocking: false,
        };

        // Step 30. Let settings object be el's node document's relevant settings object.
        // This is done by passing ModuleOwner in step 31.11 and step 32.2.
        // What we actually need is global's import map eventually.

        let base_url = doc.base_url();

        let kind = self.get_script_kind();

        if let Some(src) = element.get_attribute(&local_name!("src")) {
            // Step 31. If el has a src content attribute, then:

            // Step 31.1. If el's type is "importmap".
            if script_type == ScriptType::ImportMap {
                // then queue an element task on the DOM manipulation task source
                // given el to fire an event named error at el, and return.
                self.queue_error_event();
                return;
            }

            // Step 31.2. Let src be the value of el's src attribute.
            let src = src.value();

            // Step 31.3. If src is the empty string.
            if src.is_empty() {
                self.queue_error_event();
                return;
            }

            // Step 31.4. Set el's from an external file to true.
            self.from_an_external_file.set(true);

            // Step 31.5-31.6. Parse URL.
            let url = match base_url.join(&src) {
                Ok(url) => url,
                Err(_) => {
                    warn!("error parsing URL for script {}", &**src);
                    self.queue_error_event();
                    return;
                },
            };

            // Step 31.7. If el is potentially render-blocking, then block rendering on el.
            if self.potentially_render_blocking() && doc.allows_adding_render_blocking_elements() {
                self.marked_as_render_blocking.set(true);
                doc.increment_render_blocking_element_count();
            }

            // Step 31.8. Set el's delaying the load event to true.
            self.delay_load_event(url.clone());

            // Step 31.9. If el is currently render-blocking, then set options's render-blocking to true.
            if self.marked_as_render_blocking.get() {
                options.render_blocking = true;
            }

            // Step 31.11. Switch on el's type:
            match script_type {
                ScriptType::Classic => {
                    // Step 31.11. Fetch a classic script.
                    fetch_a_classic_script(self, kind, url, cors_setting, options, encoding);
                },
                ScriptType::Module => {
                    // If el does not have an integrity attribute, then set options's integrity metadata to
                    // the result of resolving a module integrity metadata with url and settings object.
                    if integrity_val.is_none() {
                        options.integrity_metadata = global
                            .import_map()
                            .resolve_a_module_integrity_metadata(&url);
                    }

                    // Step 31.11. Fetch an external module script graph.
                    fetch_an_external_module_script(
                        cx,
                        url,
                        ModuleOwner::Window(Trusted::new(self)),
                        options,
                    );
                },
                ScriptType::ImportMap => (),
            }
        } else {
            // Step 32. If el does not have a src content attribute:

            assert!(!text.is_empty());

            let text_rc = Rc::new(text.clone());

            // Step 32.2: Switch on el's type:
            match script_type {
                ScriptType::Classic => {
                    // Step 32.2.1 Let script be the result of creating a classic script
                    // using source text, settings object, base URL, and options.
                    let script = self.global().create_a_classic_script(
                        cx,
                        std::borrow::Cow::Borrowed(&text.str()),
                        base_url,
                        options,
                        ErrorReporting::Unmuted,
                        introduction_type,
                        self.line_number as u32,
                        false,
                    );
                    let result = Ok(Script::Classic(script));

                    if was_parser_inserted &&
                        doc.get_current_parser()
                            .is_some_and(|parser| parser.script_nesting_level() <= 1) &&
                        doc.get_script_blocking_stylesheets_count() > 0
                    {
                        // Step 34.2: classic, has no src, was parser-inserted, is blocked on stylesheet.
                        doc.set_pending_parsing_blocking_script(self, Some(result));
                    } else {
                        // Step 34.3: otherwise.
                        self.execute(cx, result);
                    }
                    return;
                },
                ScriptType::Module => {
                    // Just to make sure we running in the correct document incase the script has been moved
                    let doc = self.get_script_active_document();

                    // Step 32.2.2.1 Set el's delaying the load event to true.
                    self.delay_load_event(base_url.clone());

                    // Step 32.2.2.2 If el is potentially render-blocking, then:
                    if self.potentially_render_blocking() &&
                        doc.allows_adding_render_blocking_elements()
                    {
                        // Step 32.2.2.2.1 Block rendering on el.
                        self.marked_as_render_blocking.set(true);
                        doc.increment_render_blocking_element_count();

                        // Step 32.2.2.2.2 Set options's render-blocking to true.
                        options.render_blocking = true;
                    }

                    match kind {
                        ExternalScriptKind::Deferred => doc.add_deferred_script(self),
                        ExternalScriptKind::ParsingBlocking => {},
                        ExternalScriptKind::AsapInOrder => doc.push_asap_in_order_script(self),
                        ExternalScriptKind::Asap => doc.add_asap_script(self),
                    }

                    fetch_inline_module_script(
                        cx,
                        ModuleOwner::Window(Trusted::new(self)),
                        text_rc,
                        base_url,
                        options,
                        self.line_number as u32,
                        introduction_type,
                    );
                    return;
                },
                ScriptType::ImportMap => {
                    // Step 32.1 Let result be the result of creating an import map
                    // parse result given source text and base URL.
                    let import_map_result = parse_an_import_map_string(
                        ModuleOwner::Window(Trusted::new(self)),
                        Rc::clone(&text_rc),
                        base_url.clone(),
                    );
                    let script = Script::ImportMap(ScriptOrigin::internal(
                        text_rc,
                        base_url,
                        options,
                        script_type,
                        self.global().unminified_js_dir(),
                        import_map_result,
                    ));

                    // Step 34.3
                    self.execute(cx, Ok(script));
                    return;
                },
            }
        }

        // Just to make sure we running in the correct document incase the script has been moved
        let doc = self.get_script_active_document();

        // Step 33.2/33.3/33.4/33.5, substeps 1-2. Add el to the corresponding script list.
        match kind {
            ExternalScriptKind::Deferred => doc.add_deferred_script(self),
            ExternalScriptKind::ParsingBlocking => {
                if Some(element.get_attribute(&local_name!("src"))).is_some() &&
                    script_type == ScriptType::Classic
                {
                    doc.set_pending_parsing_blocking_script(self, None);
                }
            },
            ExternalScriptKind::AsapInOrder => doc.push_asap_in_order_script(self),
            ExternalScriptKind::Asap => doc.add_asap_script(self),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#execute-the-script-element>
    pub(crate) fn execute(&self, cx: &mut JSContext, result: ScriptResult) {
        // Step 1. Let document be el's node document.
        let doc = self.owner_document();

        // Step 2. If el's preparation-time document is not equal to document, then return.
        if *doc != *self.preparation_time_document.get().unwrap() {
            return;
        }

        // Step 3. Unblock rendering on el.
        if self.marked_as_render_blocking.replace(false) {
            self.marked_as_render_blocking.set(false);
            doc.decrement_render_blocking_element_count();
        }

        let script = match result {
            // Step 4. If el's result is null, then fire an event named error at el, and return.
            Err(_) => {
                self.dispatch_event(cx, atom!("error"));
                return;
            },

            Ok(script) => script,
        };

        // Step 5.
        // If el's from an external file is true, or el's type is "module", then increment document's
        // ignore-destructive-writes counter.
        let neutralized_doc =
            if self.from_an_external_file.get() || matches!(script, Script::Module(_)) {
                let doc = self.owner_document();
                doc.incr_ignore_destructive_writes_counter();
                Some(doc)
            } else {
                None
            };

        let document = self.owner_document();

        match script {
            Script::Classic(script) => {
                // Step 6."classic".1. Let oldCurrentScript be the value to which document's currentScript object was most recently set.
                let old_script = document.GetCurrentScript();

                // Step 6."classic".2. If el's root is not a shadow root,
                // then set document's currentScript attribute to el. Otherwise, set it to null.
                if self.upcast::<Node>().is_in_a_shadow_tree() {
                    document.set_current_script(None)
                } else {
                    document.set_current_script(Some(self))
                }

                // Step 6."classic".3. Run the classic script given by el's result.
                _ = self.owner_window().as_global_scope().run_a_classic_script(
                    cx,
                    script,
                    RethrowErrors::No,
                );

                // Step 6."classic".4. Set document's currentScript attribute to oldCurrentScript.
                document.set_current_script(old_script.as_deref());
            },
            Script::Module(module_tree) => {
                // TODO Step 6."module".1. Assert: document's currentScript attribute is null.
                document.set_current_script(None);

                // Step 6."module".2. Run the module script given by el's result.
                self.owner_window()
                    .as_global_scope()
                    .run_a_module_script(cx, module_tree, false);
            },
            Script::ImportMap(script) => {
                // Step 6."importmap".1. Register an import map given el's relevant global object and el's result.
                register_import_map(&self.owner_global(), script.import_map, CanGc::from_cx(cx));
            },
        }

        // Step 7.
        // Decrement the ignore-destructive-writes counter of document, if it was incremented in the earlier step.
        if let Some(doc) = neutralized_doc {
            doc.decr_ignore_destructive_writes_counter();
        }

        // Step 8. If el's from an external file is true, then fire an event named load at el.
        if self.from_an_external_file.get() {
            self.dispatch_event(cx, atom!("load"));
        }
    }

    pub(crate) fn queue_error_event(&self) {
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(self.upcast(), atom!("error"));
    }

    // <https://html.spec.whatwg.org/multipage/#prepare-a-script> Step 7.
    pub(crate) fn get_script_type(&self) -> Option<ScriptType> {
        let element = self.upcast::<Element>();

        let type_attr = element.get_attribute(&local_name!("type"));
        let language_attr = element.get_attribute(&local_name!("language"));

        match (
            type_attr.as_ref().map(|t| t.value()),
            language_attr.as_ref().map(|l| l.value()),
        ) {
            (Some(ref ty), _) if ty.is_empty() => {
                debug!("script type empty, inferring js");
                Some(ScriptType::Classic)
            },
            (None, Some(ref lang)) if lang.is_empty() => {
                debug!("script type empty, inferring js");
                Some(ScriptType::Classic)
            },
            (None, None) => {
                debug!("script type empty, inferring js");
                Some(ScriptType::Classic)
            },
            (None, Some(ref lang)) => {
                debug!("script language={}", &***lang);
                let language = format!("text/{}", &***lang);

                if SCRIPT_JS_MIMES.contains(&language.to_ascii_lowercase().as_str()) {
                    Some(ScriptType::Classic)
                } else {
                    None
                }
            },
            (Some(ref ty), _) => {
                debug!("script type={}", &***ty);

                if ty.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS) == "module" {
                    return Some(ScriptType::Module);
                }

                if ty.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS) == "importmap" {
                    return Some(ScriptType::ImportMap);
                }

                if SCRIPT_JS_MIMES
                    .contains(&ty.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS))
                {
                    Some(ScriptType::Classic)
                } else {
                    None
                }
            },
        }
    }

    pub(crate) fn set_parser_inserted(&self, parser_inserted: bool) {
        self.parser_inserted.set(parser_inserted);
    }

    pub(crate) fn get_parser_inserted(&self) -> bool {
        self.parser_inserted.get()
    }

    pub(crate) fn set_already_started(&self, already_started: bool) {
        self.already_started.set(already_started);
    }

    pub(crate) fn get_non_blocking(&self) -> bool {
        self.non_blocking.get()
    }

    fn dispatch_event(&self, cx: &mut JSContext, type_: Atom) -> bool {
        let window = self.owner_window();
        let event = Event::new(
            window.upcast(),
            type_,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            CanGc::from_cx(cx),
        );
        event.fire(self.upcast(), CanGc::from_cx(cx))
    }

    fn text(&self) -> DOMString {
        match self.Text() {
            TrustedScriptOrString::String(value) => value,
            TrustedScriptOrString::TrustedScript(trusted_script) => {
                DOMString::from(trusted_script.to_string())
            },
        }
    }
}

impl VirtualMethods for HTMLScriptElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        if *attr.local_name() == local_name!("src") {
            if let AttributeMutation::Set(..) = mutation {
                if !self.parser_inserted.get() && self.upcast::<Node>().is_connected() {
                    self.prepare(cx, Some(IntroductionType::INJECTED_SCRIPT));
                }
            }
        } else if *attr.local_name() == local_name!("blocking") &&
            !self.has_render_blocking_attribute() &&
            self.marked_as_render_blocking.replace(false)
        {
            let document = self.owner_document();
            document.decrement_render_blocking_element_count();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#script-processing-model:the-script-element-26>
    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(cx, mutation);
        }

        if self.upcast::<Node>().is_connected() && !self.parser_inserted.get() {
            let script = DomRoot::from_ref(self);
            // This method can be invoked while there are script/layout blockers present
            // as DOM mutations have not yet settled. We use a delayed task to avoid
            // running any scripts until the DOM tree is safe for interactions.
            self.owner_document().add_delayed_task(
                task!(ScriptPrepare: |cx, script: DomRoot<HTMLScriptElement>| {
                    script.prepare(cx, Some(IntroductionType::INJECTED_SCRIPT));
                }),
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#script-processing-model:the-script-element-20>
    fn post_connection_steps(&self, cx: &mut JSContext) {
        if let Some(s) = self.super_type() {
            s.post_connection_steps(cx);
        }

        if self.upcast::<Node>().is_connected() && !self.parser_inserted.get() {
            self.prepare(cx, Some(IntroductionType::INJECTED_SCRIPT));
        }
    }

    fn cloning_steps(
        &self,
        cx: &mut JSContext,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(cx, copy, maybe_doc, clone_children);
        }

        // https://html.spec.whatwg.org/multipage/#already-started
        if self.already_started.get() {
            copy.downcast::<HTMLScriptElement>()
                .unwrap()
                .set_already_started(true);
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        if self.marked_as_render_blocking.replace(false) {
            let document = self.owner_document();
            document.decrement_render_blocking_element_count();
        }
    }
}

impl HTMLScriptElementMethods<crate::DomTypeHolder> for HTMLScriptElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-script-src>
    fn Src(&self) -> TrustedScriptURLOrUSVString {
        let element = self.upcast::<Element>();
        element.get_trusted_type_url_attribute(&local_name!("src"))
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-src-idl-attribute>
    fn SetSrc(&self, cx: &mut JSContext, value: TrustedScriptURLOrUSVString) -> Fallible<()> {
        let element = self.upcast::<Element>();
        let local_name = &local_name!("src");
        let value = TrustedScriptURL::get_trusted_type_compliant_string(
            cx,
            &element.owner_global(),
            value,
            &format!("HTMLScriptElement {}", local_name),
        )?;
        element.set_attribute(
            local_name,
            AttrValue::String(value.str().to_owned()),
            CanGc::from_cx(cx),
        );
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-type
    make_getter!(Type, "type");
    // https://html.spec.whatwg.org/multipage/#dom-script-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-script-charset
    make_getter!(Charset, "charset");
    // https://html.spec.whatwg.org/multipage/#dom-script-charset
    make_setter!(SetCharset, "charset");

    /// <https://html.spec.whatwg.org/multipage/#dom-script-async>
    fn Async(&self) -> bool {
        self.non_blocking.get() ||
            self.upcast::<Element>()
                .has_attribute(&local_name!("async"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-async>
    fn SetAsync(&self, value: bool, can_gc: CanGc) {
        self.non_blocking.set(false);
        self.upcast::<Element>()
            .set_bool_attribute(&local_name!("async"), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_getter!(Defer, "defer");
    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_setter!(SetDefer, "defer");

    /// <https://html.spec.whatwg.org/multipage/#attr-script-blocking>
    fn Blocking(&self, cx: &mut JSContext) -> DomRoot<DOMTokenList> {
        self.blocking.or_init(|| {
            DOMTokenList::new(
                self.upcast(),
                &local_name!("blocking"),
                Some(vec![Atom::from("render")]),
                CanGc::from_cx(cx),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-nomodule
    make_bool_getter!(NoModule, "nomodule");
    // https://html.spec.whatwg.org/multipage/#dom-script-nomodule
    make_bool_setter!(SetNoModule, "nomodule");

    // https://html.spec.whatwg.org/multipage/#dom-script-integrity
    make_getter!(Integrity, "integrity");
    // https://html.spec.whatwg.org/multipage/#dom-script-integrity
    make_setter!(SetIntegrity, "integrity");

    // https://html.spec.whatwg.org/multipage/#dom-script-event
    make_getter!(Event, "event");
    // https://html.spec.whatwg.org/multipage/#dom-script-event
    make_setter!(SetEvent, "event");

    // https://html.spec.whatwg.org/multipage/#dom-script-htmlfor
    make_getter!(HtmlFor, "for");
    // https://html.spec.whatwg.org/multipage/#dom-script-htmlfor
    make_setter!(SetHtmlFor, "for");

    /// <https://html.spec.whatwg.org/multipage/#dom-script-crossorigin>
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-crossorigin>
    fn SetCrossOrigin(&self, cx: &mut JSContext, value: Option<DOMString>) {
        set_cross_origin_attribute(cx, self.upcast::<Element>(), value);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy>
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    /// <https://w3c.github.io/trusted-types/dist/spec/#dom-htmlscriptelement-innertext>
    fn InnerText(&self) -> TrustedScriptOrString {
        // Step 1: Return the result of running get the text steps with this.
        TrustedScriptOrString::String(self.upcast::<HTMLElement>().get_inner_outer_text())
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-innerText-idl-attribute>
    fn SetInnerText(&self, cx: &mut JSContext, input: TrustedScriptOrString) -> Fallible<()> {
        // Step 1: Let value be the result of calling Get Trusted Type compliant string with TrustedScript,
        // this's relevant global object, the given value, HTMLScriptElement innerText, and script.
        let value = TrustedScript::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            input,
            "HTMLScriptElement innerText",
        )?;
        *self.script_text.borrow_mut() = value.clone();
        // Step 3: Run set the inner text steps with this and value.
        self.upcast::<HTMLElement>().set_inner_text(cx, value);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-text>
    fn Text(&self) -> TrustedScriptOrString {
        TrustedScriptOrString::String(self.upcast::<Node>().child_text_content())
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-text-idl-attribute>
    fn SetText(&self, cx: &mut JSContext, value: TrustedScriptOrString) -> Fallible<()> {
        // Step 1: Let value be the result of calling Get Trusted Type compliant string with TrustedScript,
        // this's relevant global object, the given value, HTMLScriptElement text, and script.
        let value = TrustedScript::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            value,
            "HTMLScriptElement text",
        )?;
        // Step 2: Set this's script text value to the given value.
        *self.script_text.borrow_mut() = value.clone();
        // Step 3: String replace all with the given value within this.
        Node::string_replace_all(cx, value, self.upcast::<Node>());
        Ok(())
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-textContent-idl-attribute>
    fn GetTextContent(&self) -> Option<TrustedScriptOrString> {
        // Step 1: Return the result of running get text content with this.
        Some(TrustedScriptOrString::String(
            self.upcast::<Node>().GetTextContent()?,
        ))
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-textContent-idl-attribute>
    fn SetTextContent(
        &self,
        cx: &mut JSContext,
        value: Option<TrustedScriptOrString>,
    ) -> Fallible<()> {
        // Step 1: Let value be the result of calling Get Trusted Type compliant string with TrustedScript,
        // this's relevant global object, the given value, HTMLScriptElement textContent, and script.
        let value = TrustedScript::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            value.unwrap_or(TrustedScriptOrString::String(DOMString::from(""))),
            "HTMLScriptElement textContent",
        )?;
        // Step 2: Set this's script text value to value.
        *self.script_text.borrow_mut() = value.clone();
        // Step 3: Run set text content with this and value.
        self.upcast::<Node>()
            .set_text_content_for_element(cx, Some(value));
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-supports>
    fn Supports(_window: &Window, type_: DOMString) -> bool {
        // The type argument has to exactly match these values,
        // we do not perform an ASCII case-insensitive match.
        matches!(&*type_.str(), "classic" | "module" | "importmap")
    }
}

pub(crate) fn substitute_with_local_script(
    window: &Window,
    script: &mut Cow<'_, str>,
    url: ServoUrl,
) {
    if window.local_script_source().is_none() {
        return;
    }
    let mut path = PathBuf::from(window.local_script_source().clone().unwrap());
    path = path.join(&url[url::Position::BeforeHost..]);
    debug!("Attempting to read script stored at: {:?}", path);
    match read_to_string(path.clone()) {
        Ok(local_script) => {
            debug!("Found script stored at: {:?}", path);
            *script = Cow::Owned(local_script);
        },
        Err(why) => warn!("Could not restore script from file {:?}", why),
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ExternalScriptKind {
    Deferred,
    ParsingBlocking,
    AsapInOrder,
    Asap,
}
