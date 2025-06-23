/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(unused_imports)]
use core::ffi::c_void;
use std::cell::Cell;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::Command;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use base::id::{PipelineId, WebViewId};
use content_security_policy as csp;
use devtools_traits::{ScriptToDevtoolsControlMsg, SourceInfo};
use dom_struct::dom_struct;
use encoding_rs::Encoding;
use html5ever::serialize::TraversalScope;
use html5ever::{LocalName, Prefix, local_name, namespace_url, ns};
use ipc_channel::ipc;
use js::jsval::UndefinedValue;
use js::rust::{CompileOptionsWrapper, HandleObject, Stencil, transform_str_to_source_text};
use net_traits::http_status::HttpStatus;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CorsSettings, CredentialsMode, Destination, InsecureRequestsPolicy, ParserMetadata,
    RequestBuilder, RequestId,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, IpcSend, Metadata, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::attr::AttrValue;
use style::str::{HTML_SPACE_CHARACTERS, StaticStringVec};
use stylo_atoms::Atom;
use uuid::Uuid;

use crate::HasParent;
use crate::document_loader::LoadType;
use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::GenericBindings::HTMLElementBinding::HTMLElement_Binding::HTMLElementMethods;
use crate::dom::bindings::codegen::UnionTypes::{
    TrustedScriptOrString, TrustedScriptURLOrUSVString,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::NoTrace;
use crate::dom::document::Document;
use crate::dom::element::{
    AttributeMutation, Element, ElementCreator, cors_setting_for_element,
    referrer_policy_for_element, reflect_cross_origin_attribute, reflect_referrer_policy_attribute,
    set_cross_origin_attribute,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{ChildrenMutation, CloneChildrenFlag, Node, NodeTraits};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::trustedscript::TrustedScript;
use crate::dom::trustedscripturl::TrustedScriptURL;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::fetch::{create_a_potential_cors_request, load_whole_resource};
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::realms::enter_realm;
use crate::script_module::{
    ImportMap, ModuleOwner, ScriptFetchOptions, fetch_external_module_script,
    fetch_inline_module_script, parse_an_import_map_string, register_import_map,
};
use crate::script_runtime::CanGc;
use crate::task_source::{SendableTaskSource, TaskSourceName};
use crate::unminify::{ScriptSource, unminify_js};

impl ScriptSource for ScriptOrigin {
    fn unminified_dir(&self) -> Option<String> {
        self.unminified_dir.clone()
    }

    fn extract_bytes(&self) -> &[u8] {
        match &self.code {
            SourceCode::Text(text) => text.as_bytes(),
            SourceCode::Compiled(compiled_source_code) => {
                compiled_source_code.original_text.as_bytes()
            },
        }
    }

    fn rewrite_source(&mut self, source: Rc<DOMString>) {
        self.code = SourceCode::Text(source);
    }

    fn url(&self) -> ServoUrl {
        self.url.clone()
    }

    fn is_external(&self) -> bool {
        self.external
    }
}

// TODO Implement offthread compilation in mozjs
/*pub(crate) struct OffThreadCompilationContext {
    script_element: Trusted<HTMLScriptElement>,
    script_kind: ExternalScriptKind,
    final_url: ServoUrl,
    url: ServoUrl,
    task_source: TaskSource,
    script_text: String,
    fetch_options: ScriptFetchOptions,
}

#[allow(unsafe_code)]
unsafe extern "C" fn off_thread_compilation_callback(
    token: *mut OffThreadToken,
    callback_data: *mut c_void,
) {
    let mut context = Box::from_raw(callback_data as *mut OffThreadCompilationContext);
    let token = OffThreadCompilationToken(token);

    let url = context.url.clone();
    let final_url = context.final_url.clone();
    let script_element = context.script_element.clone();
    let script_kind = context.script_kind;
    let script = std::mem::take(&mut context.script_text);
    let fetch_options = context.fetch_options.clone();

    // Continue with <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
    let _ = context.task_source.queue(
        task!(off_thread_compile_continue: move || {
            let elem = script_element.root();
            let global = elem.global();
            let cx = GlobalScope::get_cx();
            let _ar = enter_realm(&*global);

            // TODO: This is necessary because the rust compiler will otherwise try to move the *mut
            // OffThreadToken directly, which isn't marked as Send. The correct fix is that this
            // type is marked as Send in mozjs.
            let used_token = token;

            let compiled_script = FinishOffThreadStencil(*cx, used_token.0, ptr::null_mut());
            let load = if compiled_script.is_null() {
                Err(NoTrace(NetworkError::Internal(
                    "Off-thread compilation failed.".into(),
                )))
            } else {
                let script_text = DOMString::from(script);
                let code = SourceCode::Compiled(CompiledSourceCode {
                    source_code: compiled_script,
                    original_text: Rc::new(script_text),
                });

                Ok(ScriptOrigin {
                    code,
                    url: final_url,
                    external: true,
                    fetch_options,
                    type_: ScriptType::Classic,
                })
            };

            finish_fetching_a_classic_script(&elem, script_kind, url, load);
        })
    );
}*/

/// An unique id for script element.
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub(crate) struct ScriptId(#[no_trace] Uuid);

#[dom_struct]
pub(crate) struct HTMLScriptElement {
    htmlelement: HTMLElement,

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
            parser_inserted: Cell::new(creator.is_parser_created()),
            non_blocking: Cell::new(!creator.is_parser_created()),
            parser_document: Dom::from_ref(document),
            preparation_time_document: MutNullableDom::new(None),
            line_number: creator.return_line_number(),
            script_text: DomRefCell::new(DOMString::new()),
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
    ) -> DomRoot<HTMLScriptElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLScriptElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_script_id(&self) -> ScriptId {
        self.id
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
    code: SourceCode,
    #[no_trace]
    url: ServoUrl,
    external: bool,
    fetch_options: ScriptFetchOptions,
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
            import_map: Err(Error::NotFound),
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
fn finish_fetching_a_classic_script(
    elem: &HTMLScriptElement,
    script_kind: ExternalScriptKind,
    url: ServoUrl,
    load: ScriptResult,
    can_gc: CanGc,
) {
    // Step 33. The "steps to run when the result is ready" for each type of script in 33.2-33.5.
    // of https://html.spec.whatwg.org/multipage/#prepare-the-script-element
    let document;

    match script_kind {
        ExternalScriptKind::Asap => {
            document = elem.preparation_time_document.get().unwrap();
            document.asap_script_loaded(elem, load, can_gc)
        },
        ExternalScriptKind::AsapInOrder => {
            document = elem.preparation_time_document.get().unwrap();
            document.asap_in_order_script_loaded(elem, load, can_gc)
        },
        ExternalScriptKind::Deferred => {
            document = elem.parser_document.as_rooted();
            document.deferred_script_loaded(elem, load, can_gc);
        },
        ExternalScriptKind::ParsingBlocking => {
            document = elem.parser_document.as_rooted();
            document.pending_parsing_blocking_script_loaded(elem, load, can_gc);
        },
    }

    document.finish_load(LoadType::Script(url), can_gc);
}

pub(crate) type ScriptResult = Result<ScriptOrigin, NoTrace<NetworkError>>;

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
    /// Timing object for this resource
    resource_timing: ResourceFetchTiming,
}

impl FetchResponseListener for ClassicContext {
    // TODO(KiChjang): Perhaps add custom steps to perform fetch here?
    fn process_request_body(&mut self, _: RequestId) {}

    // TODO(KiChjang): Perhaps add custom steps to perform fetch here?
    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        let status = self
            .metadata
            .as_ref()
            .map(|m| m.status.clone())
            .unwrap_or_else(HttpStatus::new_error);

        self.status = {
            if status.is_error() {
                Err(NetworkError::Internal(
                    "No http status code received".to_owned(),
                ))
            } else if status.is_success() {
                Ok(())
            } else {
                Err(NetworkError::Internal(format!(
                    "HTTP error code {}",
                    status.code()
                )))
            }
        };
    }

    fn process_response_chunk(&mut self, _: RequestId, mut chunk: Vec<u8>) {
        if self.status.is_ok() {
            self.data.append(&mut chunk);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
    /// step 4-9
    #[allow(unsafe_code)]
    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let (source_text, final_url) = match (response.as_ref(), self.status.as_ref()) {
            (Err(err), _) | (_, Err(err)) => {
                // Step 6, response is an error.
                finish_fetching_a_classic_script(
                    &self.elem.root(),
                    self.kind,
                    self.url.clone(),
                    Err(NoTrace(err.clone())),
                    CanGc::note(),
                );
                return;
            },
            (Ok(_), Ok(_)) => {
                let metadata = self.metadata.take().unwrap();

                // Step 7.
                let encoding = metadata
                    .charset
                    .and_then(|encoding| Encoding::for_label(encoding.as_bytes()))
                    .unwrap_or(self.character_encoding);

                // Step 8.
                let (source_text, _, _) = encoding.decode(&self.data);
                (source_text, metadata.final_url)
            },
        };

        let elem = self.elem.root();
        let global = elem.global();
        //let cx = GlobalScope::get_cx();
        let _ar = enter_realm(&*global);

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
        let load = ScriptOrigin::external(
            Rc::new(DOMString::from(source_text)),
            final_url.clone(),
            self.fetch_options.clone(),
            ScriptType::Classic,
            elem.parser_document.global().unminified_js_dir(),
        );
        finish_fetching_a_classic_script(
            &elem,
            self.kind,
            self.url.clone(),
            Ok(load),
            CanGc::note(),
        );
        //}
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

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<csp::Violation>) {
        let global = &self.resource_timing_global();
        let elem = self.elem.root();
        global.report_csp_violations(violations, Some(elem.upcast::<Element>()));
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

impl PreInvoke for ClassicContext {}

/// Steps 1-2 of <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
// This function is also used to prefetch a script in `script::dom::servoparser::prefetch`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn script_fetch_request(
    webview_id: WebViewId,
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    origin: ImmutableOrigin,
    pipeline_id: PipelineId,
    options: ScriptFetchOptions,
    insecure_requests_policy: InsecureRequestsPolicy,
    has_trustworthy_ancestor_origin: bool,
    policy_container: PolicyContainer,
) -> RequestBuilder {
    // We intentionally ignore options' credentials_mode member for classic scripts.
    // The mode is initialized by create_a_potential_cors_request.
    create_a_potential_cors_request(
        Some(webview_id),
        url,
        Destination::Script,
        cors_setting,
        None,
        options.referrer,
        insecure_requests_policy,
        has_trustworthy_ancestor_origin,
        policy_container,
    )
    .origin(origin)
    .pipeline_id(Some(pipeline_id))
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
    let request = script_fetch_request(
        doc.webview_id(),
        url.clone(),
        cors_setting,
        doc.origin().immutable().clone(),
        global.pipeline_id(),
        options.clone(),
        doc.insecure_requests_policy(),
        doc.has_trustworthy_ancestor_origin(),
        global.policy_container(),
    );
    let request = doc.prepare_request(request);

    // TODO: Step 3, Add custom steps to perform fetch

    let context = ClassicContext {
        elem: Trusted::new(script),
        kind,
        character_encoding,
        data: vec![],
        metadata: None,
        url: url.clone(),
        status: Ok(()),
        fetch_options: options,
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
    };
    doc.fetch(LoadType::Script(url), request, context);
}

impl HTMLScriptElement {
    /// <https://w3c.github.io/trusted-types/dist/spec/#setting-slot-values-from-parser>
    pub(crate) fn set_initial_script_text(&self) {
        *self.script_text.borrow_mut() = self.text();
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#abstract-opdef-prepare-the-script-text>
    fn prepare_the_script_text(&self, can_gc: CanGc) -> Fallible<()> {
        // Step 1. If script’s script text value is not equal to its child text content,
        // set script’s script text to the result of executing
        // Get Trusted Type compliant string, with the following arguments:
        if self.script_text.borrow().clone() != self.text() {
            *self.script_text.borrow_mut() = TrustedScript::get_trusted_script_compliant_string(
                &self.owner_global(),
                self.Text(),
                "HTMLScriptElement",
                "text",
                can_gc,
            )?;
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#prepare-the-script-element>
    pub(crate) fn prepare(&self, can_gc: CanGc) {
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
        if self.prepare_the_script_text(can_gc).is_err() {
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
        if !doc.is_scripting_enabled() {
            return;
        }

        // Step 18. If el has a nomodule content attribute and its type is "classic", then return.
        if element.has_attribute(&local_name!("nomodule")) && script_type == ScriptType::Classic {
            return;
        }

        // Step 19. CSP.
        if !element.has_attribute(&local_name!("src")) &&
            doc.should_elements_inline_type_behavior_be_blocked(
                element,
                csp::InlineCheckType::Script,
                &text,
            ) == csp::CheckResult::Blocked
        {
            warn!("Blocking inline script due to CSP");
            return;
        }

        // Step 20. If el has an event attribute and a for attribute, and el's type is "classic", then:
        if script_type == ScriptType::Classic {
            let for_attribute = element.get_attribute(&ns!(), &local_name!("for"));
            let event_attribute = element.get_attribute(&ns!(), &local_name!("event"));
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
            .get_attribute(&ns!(), &local_name!("charset"))
            .and_then(|charset| Encoding::for_label(charset.value().as_bytes()))
            .unwrap_or_else(|| doc.encoding());

        // Step 22. CORS setting.
        let cors_setting = cors_setting_for_element(element);

        // Step 23. Module script credentials mode.
        let module_credentials_mode = match script_type {
            ScriptType::Classic => CredentialsMode::CredentialsSameOrigin,
            ScriptType::Module | ScriptType::ImportMap => reflect_cross_origin_attribute(element)
                .map_or(
                    CredentialsMode::CredentialsSameOrigin,
                    |attr| match &*attr {
                        "use-credentials" => CredentialsMode::Include,
                        "anonymous" => CredentialsMode::CredentialsSameOrigin,
                        _ => CredentialsMode::CredentialsSameOrigin,
                    },
                ),
        };

        // Step 24. Let cryptographic nonce be el's [[CryptographicNonce]] internal slot's value.
        let cryptographic_nonce = self.upcast::<Element>().nonce_value();

        // Step 25. If el has an integrity attribute, then let integrity metadata be that attribute's value.
        // Otherwise, let integrity metadata be the empty string.
        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
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
        let options = ScriptFetchOptions {
            cryptographic_nonce,
            integrity_metadata: integrity_metadata.to_owned(),
            parser_metadata,
            referrer: self.global().get_referrer(),
            referrer_policy,
            credentials_mode: module_credentials_mode,
        };

        // Step 30. Let settings object be el's node document's relevant settings object.
        // This is done by passing ModuleOwner in step 31.11 and step 32.2.
        // What we actually need is global's import map eventually.

        let base_url = doc.base_url();
        if let Some(src) = element.get_attribute(&ns!(), &local_name!("src")) {
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
            // The "from an external file"" flag is stored in ScriptOrigin.

            // Step 31.5-31.6. Parse URL.
            let url = match base_url.join(&src) {
                Ok(url) => url,
                Err(_) => {
                    warn!("error parsing URL for script {}", &**src);
                    self.queue_error_event();
                    return;
                },
            };

            // TODO:
            // Step 31.7. If el is potentially render-blocking, then block rendering on el.
            // Step 31.8. Set el's delaying the load event to true.
            // Step 31.9. If el is currently render-blocking, then set options's render-blocking to true.

            // Step 31.11. Switch on el's type:
            match script_type {
                ScriptType::Classic => {
                    let kind = if element.has_attribute(&local_name!("defer")) &&
                        was_parser_inserted &&
                        !asynch
                    {
                        // Step 33.4: classic, has src, has defer, was parser-inserted, is not async.
                        ExternalScriptKind::Deferred
                    } else if was_parser_inserted && !asynch {
                        // Step 33.5: classic, has src, was parser-inserted, is not async.
                        ExternalScriptKind::ParsingBlocking
                    } else if !asynch && !self.non_blocking.get() {
                        // Step 33.3: classic, has src, is not async, is not non-blocking.
                        ExternalScriptKind::AsapInOrder
                    } else {
                        // Step 33.2: classic, has src.
                        ExternalScriptKind::Asap
                    };

                    // Step 31.11. Fetch a classic script.
                    fetch_a_classic_script(self, kind, url, cors_setting, options, encoding);

                    // Step 33.2/33.3/33.4/33.5, substeps 1-2. Add el to the corresponding script list.
                    match kind {
                        ExternalScriptKind::Deferred => doc.add_deferred_script(self),
                        ExternalScriptKind::ParsingBlocking => {
                            doc.set_pending_parsing_blocking_script(self, None)
                        },
                        ExternalScriptKind::AsapInOrder => doc.push_asap_in_order_script(self),
                        ExternalScriptKind::Asap => doc.add_asap_script(self),
                    }
                },
                ScriptType::Module => {
                    // Step 31.11. Fetch an external module script graph.
                    fetch_external_module_script(
                        ModuleOwner::Window(Trusted::new(self)),
                        url.clone(),
                        Destination::Script,
                        options,
                        can_gc,
                    );

                    if !asynch && was_parser_inserted {
                        // 33.4: module, not async, parser-inserted
                        doc.add_deferred_script(self);
                    } else if !asynch && !self.non_blocking.get() {
                        // 33.3: module, not parser-inserted
                        doc.push_asap_in_order_script(self);
                    } else {
                        // 33.2: module, async
                        doc.add_asap_script(self);
                    };
                },
                ScriptType::ImportMap => (),
            }
        } else {
            // Step 32. If el does not have a src content attribute:

            assert!(!text.is_empty());

            let text_rc = Rc::new(text);

            // Step 32.2: Switch on el's type:
            match script_type {
                ScriptType::Classic => {
                    let result = Ok(ScriptOrigin::internal(
                        text_rc,
                        base_url,
                        options,
                        script_type,
                        self.global().unminified_js_dir(),
                        Err(Error::NotFound),
                    ));

                    if was_parser_inserted &&
                        doc.get_current_parser()
                            .is_some_and(|parser| parser.script_nesting_level() <= 1) &&
                        doc.get_script_blocking_stylesheets_count() > 0
                    {
                        // Step 34.2: classic, has no src, was parser-inserted, is blocked on stylesheet.
                        doc.set_pending_parsing_blocking_script(self, Some(result));
                    } else {
                        // Step 34.3: otherwise.
                        self.execute(result, can_gc);
                    }
                },
                ScriptType::Module => {
                    // We should add inline module script elements
                    // into those vectors in case that there's no
                    // descendants in the inline module script.
                    if !asynch && was_parser_inserted {
                        doc.add_deferred_script(self);
                    } else if !asynch && !self.non_blocking.get() {
                        doc.push_asap_in_order_script(self);
                    } else {
                        doc.add_asap_script(self);
                    };

                    fetch_inline_module_script(
                        ModuleOwner::Window(Trusted::new(self)),
                        text_rc,
                        base_url.clone(),
                        self.id,
                        options,
                        can_gc,
                    );
                },
                ScriptType::ImportMap => {
                    // Step 32.1 Let result be the result of creating an import map
                    // parse result given source text and base URL.
                    let import_map_result = parse_an_import_map_string(
                        ModuleOwner::Window(Trusted::new(self)),
                        Rc::clone(&text_rc),
                        base_url.clone(),
                        can_gc,
                    );
                    let result = Ok(ScriptOrigin::internal(
                        text_rc,
                        base_url,
                        options,
                        script_type,
                        self.global().unminified_js_dir(),
                        import_map_result,
                    ));

                    // Step 34.3
                    self.execute(result, can_gc);
                },
            }
        }
    }

    fn substitute_with_local_script(&self, script: &mut ScriptOrigin) {
        if self
            .parser_document
            .window()
            .local_script_source()
            .is_none() ||
            !script.external
        {
            return;
        }
        let mut path = PathBuf::from(
            self.parser_document
                .window()
                .local_script_source()
                .clone()
                .unwrap(),
        );
        path = path.join(&script.url[url::Position::BeforeHost..]);
        debug!("Attempting to read script stored at: {:?}", path);
        match read_to_string(path.clone()) {
            Ok(local_script) => {
                debug!("Found script stored at: {:?}", path);
                script.code = SourceCode::Text(Rc::new(DOMString::from(local_script)));
            },
            Err(why) => warn!("Could not restore script from file {:?}", why),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#execute-the-script-element>
    pub(crate) fn execute(&self, result: ScriptResult, can_gc: CanGc) {
        // Step 1. Let document be el's node document.
        let doc = self.owner_document();

        // Step 2. If el's preparation-time document is not equal to document, then return.
        if *doc != *self.preparation_time_document.get().unwrap() {
            return;
        }

        // TODO: Step 3. Unblock rendering on el.
        let mut script = match result {
            // Step 4. If el's result is null, then fire an event named error at el, and return.
            Err(e) => {
                warn!("error loading script {:?}", e);
                self.dispatch_error_event(can_gc);
                return;
            },

            Ok(script) => script,
        };

        if let Some(chan) = self.global().devtools_chan() {
            let pipeline_id = self.global().pipeline_id();

            let (url, content, content_type, is_external) = if script.external {
                let content = match &script.code {
                    SourceCode::Text(text) => text.to_string(),
                    SourceCode::Compiled(compiled) => compiled.original_text.to_string(),
                };

                // content_type: https://html.spec.whatwg.org/multipage/#scriptingLanguages
                (script.url.clone(), Some(content), "text/javascript", true)
            } else {
                // TODO: if needed, fetch the page again, in the same way as in the original request.
                // Fetch it from cache, even if the original request was non-idempotent (e.g. POST).
                // If we can’t fetch it from cache, we should probably give up, because with a real
                // fetch, the server could return a different response.

                // TODO: handle cases where Content-Type is not text/html.
                (doc.url(), None, "text/html", false)
            };

            let source_info = SourceInfo {
                url,
                external: is_external,
                worker_id: None,
                content,
                content_type: Some(content_type.to_string()),
            };
            let _ = chan.send(ScriptToDevtoolsControlMsg::CreateSourceActor(
                pipeline_id,
                source_info,
            ));
        }

        if script.type_ == ScriptType::Classic {
            unminify_js(&mut script);
            self.substitute_with_local_script(&mut script);
        }

        // Step 5.
        // If el's from an external file is true, or el's type is "module", then increment document's
        // ignore-destructive-writes counter.
        let neutralized_doc = if script.external || script.type_ == ScriptType::Module {
            debug!("loading external script, url = {}", script.url);
            let doc = self.owner_document();
            doc.incr_ignore_destructive_writes_counter();
            Some(doc)
        } else {
            None
        };

        // Step 6.
        let document = self.owner_document();
        let old_script = document.GetCurrentScript();

        match script.type_ {
            ScriptType::Classic => {
                if self.upcast::<Node>().is_in_a_shadow_tree() {
                    document.set_current_script(None)
                } else {
                    document.set_current_script(Some(self))
                }
                self.run_a_classic_script(&script, can_gc);
                document.set_current_script(old_script.as_deref());
            },
            ScriptType::Module => {
                document.set_current_script(None);
                self.run_a_module_script(&script, false, can_gc);
            },
            ScriptType::ImportMap => {
                // Step 6.1 Register an import map given el's relevant global object and el's result.
                register_import_map(&self.owner_global(), script.import_map, can_gc);
            },
        }

        // Step 7.
        // Decrement the ignore-destructive-writes counter of document, if it was incremented in the earlier step.
        if let Some(doc) = neutralized_doc {
            doc.decr_ignore_destructive_writes_counter();
        }

        // Step 8. If el's from an external file is true, then fire an event named load at el.
        if script.external {
            self.dispatch_load_event(can_gc);
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-a-classic-script
    pub(crate) fn run_a_classic_script(&self, script: &ScriptOrigin, can_gc: CanGc) {
        // TODO use a settings object rather than this element's document/window
        // Step 2
        let document = self.owner_document();
        if !document.is_fully_active() || !document.is_scripting_enabled() {
            return;
        }

        // Steps 4-10
        let window = self.owner_window();
        let line_number = if script.external {
            1
        } else {
            self.line_number as u32
        };
        rooted!(in(*GlobalScope::get_cx()) let mut rval = UndefinedValue());
        window
            .as_global_scope()
            .evaluate_script_on_global_with_result(
                &script.code,
                script.url.as_str(),
                rval.handle_mut(),
                line_number,
                script.fetch_options.clone(),
                script.url.clone(),
                can_gc,
            );
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#run-a-module-script>
    pub(crate) fn run_a_module_script(
        &self,
        script: &ScriptOrigin,
        _rethrow_errors: bool,
        can_gc: CanGc,
    ) {
        // TODO use a settings object rather than this element's document/window
        // Step 2
        let document = self.owner_document();
        if !document.is_fully_active() || !document.is_scripting_enabled() {
            return;
        }

        // Step 4
        let window = self.owner_window();
        let global = window.as_global_scope();
        let _aes = AutoEntryScript::new(global);

        let tree = if script.external {
            global.get_module_map().borrow().get(&script.url).cloned()
        } else {
            global
                .get_inline_module_map()
                .borrow()
                .get(&self.id.clone())
                .cloned()
        };

        if let Some(module_tree) = tree {
            // Step 6.
            {
                let module_error = module_tree.get_rethrow_error().borrow();
                let network_error = module_tree.get_network_error().borrow();
                if module_error.is_some() && network_error.is_none() {
                    module_tree.report_error(global, can_gc);
                    return;
                }
            }

            let record = module_tree
                .get_record()
                .borrow()
                .as_ref()
                .map(|record| record.handle());

            if let Some(record) = record {
                rooted!(in(*GlobalScope::get_cx()) let mut rval = UndefinedValue());
                let evaluated =
                    module_tree.execute_module(global, record, rval.handle_mut().into(), can_gc);

                if let Err(exception) = evaluated {
                    module_tree.set_rethrow_error(exception);
                    module_tree.report_error(global, can_gc);
                }
            }
        }
    }

    pub(crate) fn queue_error_event(&self) {
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(self.upcast(), atom!("error"));
    }

    pub(crate) fn dispatch_load_event(&self, can_gc: CanGc) {
        self.dispatch_event(
            atom!("load"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
    }

    pub(crate) fn dispatch_error_event(&self, can_gc: CanGc) {
        self.dispatch_event(
            atom!("error"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            can_gc,
        );
    }

    // https://html.spec.whatwg.org/multipage/#prepare-a-script Step 7.
    pub(crate) fn get_script_type(&self) -> Option<ScriptType> {
        let element = self.upcast::<Element>();

        let type_attr = element.get_attribute(&ns!(), &local_name!("type"));
        let language_attr = element.get_attribute(&ns!(), &local_name!("language"));

        let script_type = match (
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
        };

        // https://github.com/rust-lang/rust/issues/21114
        script_type
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

    fn dispatch_event(
        &self,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        can_gc: CanGc,
    ) -> EventStatus {
        let window = self.owner_window();
        let event = Event::new(window.upcast(), type_, bubbles, cancelable, can_gc);
        event.fire(self.upcast(), can_gc)
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

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        if *attr.local_name() == local_name!("src") {
            if let AttributeMutation::Set(_) = mutation {
                if !self.parser_inserted.get() && self.upcast::<Node>().is_connected() {
                    self.prepare(can_gc);
                }
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#script-processing-model:the-script-element-26>
    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }

        if self.upcast::<Node>().is_connected() && !self.parser_inserted.get() {
            let script = Trusted::new(self);
            // This method can be invoked while there are script/layout blockers present
            // as DOM mutations have not yet settled. We use a delayed task to avoid
            // running any scripts until the DOM tree is safe for interactions.
            self.owner_document()
                .add_delayed_task(task!(ScriptPrepare: move || {
                    let this = script.root();
                    this.prepare(CanGc::note());
                }));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#script-processing-model:the-script-element-20>
    fn post_connection_steps(&self) {
        if let Some(s) = self.super_type() {
            s.post_connection_steps();
        }

        if self.upcast::<Node>().is_connected() && !self.parser_inserted.get() {
            self.prepare(CanGc::note());
        }
    }

    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
        can_gc: CanGc,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children, can_gc);
        }

        // https://html.spec.whatwg.org/multipage/#already-started
        if self.already_started.get() {
            copy.downcast::<HTMLScriptElement>()
                .unwrap()
                .set_already_started(true);
        }
    }
}

impl HTMLScriptElementMethods<crate::DomTypeHolder> for HTMLScriptElement {
    // https://html.spec.whatwg.org/multipage/#dom-script-src
    fn Src(&self) -> TrustedScriptURLOrUSVString {
        let element = self.upcast::<Element>();
        element.get_trusted_type_url_attribute(&local_name!("src"))
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-src-idl-attribute>
    fn SetSrc(&self, value: TrustedScriptURLOrUSVString, can_gc: CanGc) -> Fallible<()> {
        let element = self.upcast::<Element>();
        let local_name = &local_name!("src");
        let value = TrustedScriptURL::get_trusted_script_url_compliant_string(
            &element.owner_global(),
            value,
            "HTMLScriptElement",
            local_name,
            can_gc,
        )?;
        element.set_attribute(
            local_name,
            AttrValue::String(value.as_ref().to_owned()),
            can_gc,
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

    // https://html.spec.whatwg.org/multipage/#dom-script-async
    fn Async(&self) -> bool {
        self.non_blocking.get() ||
            self.upcast::<Element>()
                .has_attribute(&local_name!("async"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-async
    fn SetAsync(&self, value: bool, can_gc: CanGc) {
        self.non_blocking.set(false);
        self.upcast::<Element>()
            .set_bool_attribute(&local_name!("async"), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_getter!(Defer, "defer");
    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_setter!(SetDefer, "defer");

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

    // https://html.spec.whatwg.org/multipage/#dom-script-crossorigin
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-crossorigin
    fn SetCrossOrigin(&self, value: Option<DOMString>, can_gc: CanGc) {
        set_cross_origin_attribute(self.upcast::<Element>(), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    /// <https://w3c.github.io/trusted-types/dist/spec/#dom-htmlscriptelement-innertext>
    fn InnerText(&self, can_gc: CanGc) -> TrustedScriptOrString {
        // Step 1: Return the result of running get the text steps with this.
        TrustedScriptOrString::String(self.upcast::<HTMLElement>().get_inner_outer_text(can_gc))
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-innerText-idl-attribute>
    fn SetInnerText(&self, input: TrustedScriptOrString, can_gc: CanGc) -> Fallible<()> {
        // Step 1: Let value be the result of calling Get Trusted Type compliant string with TrustedScript,
        // this's relevant global object, the given value, HTMLScriptElement innerText, and script.
        let value = TrustedScript::get_trusted_script_compliant_string(
            &self.owner_global(),
            input,
            "HTMLScriptElement",
            "innerText",
            can_gc,
        )?;
        *self.script_text.borrow_mut() = value.clone();
        // Step 3: Run set the inner text steps with this and value.
        self.upcast::<HTMLElement>().set_inner_text(value, can_gc);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-text>
    fn Text(&self) -> TrustedScriptOrString {
        TrustedScriptOrString::String(self.upcast::<Node>().child_text_content())
    }

    /// <https://w3c.github.io/trusted-types/dist/spec/#the-text-idl-attribute>
    fn SetText(&self, value: TrustedScriptOrString, can_gc: CanGc) -> Fallible<()> {
        // Step 1: Let value be the result of calling Get Trusted Type compliant string with TrustedScript,
        // this's relevant global object, the given value, HTMLScriptElement text, and script.
        let value = TrustedScript::get_trusted_script_compliant_string(
            &self.owner_global(),
            value,
            "HTMLScriptElement",
            "text",
            can_gc,
        )?;
        // Step 2: Set this's script text value to the given value.
        *self.script_text.borrow_mut() = value.clone();
        // Step 3: String replace all with the given value within this.
        Node::string_replace_all(value, self.upcast::<Node>(), can_gc);
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
    fn SetTextContent(&self, value: Option<TrustedScriptOrString>, can_gc: CanGc) -> Fallible<()> {
        // Step 1: Let value be the result of calling Get Trusted Type compliant string with TrustedScript,
        // this's relevant global object, the given value, HTMLScriptElement textContent, and script.
        let value = TrustedScript::get_trusted_script_compliant_string(
            &self.owner_global(),
            value.unwrap_or(TrustedScriptOrString::String(DOMString::from(""))),
            "HTMLScriptElement",
            "textContent",
            can_gc,
        )?;
        // Step 2: Set this's script text value to value.
        *self.script_text.borrow_mut() = value.clone();
        // Step 3: Run set text content with this and value.
        self.upcast::<Node>().SetTextContent(Some(value), can_gc);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-script-supports>
    fn Supports(_window: &Window, type_: DOMString) -> bool {
        // The type argument has to exactly match these values,
        // we do not perform an ASCII case-insensitive match.
        matches!(type_.str(), "classic" | "module" | "importmap")
    }
}

#[derive(Clone, Copy)]
enum ExternalScriptKind {
    Deferred,
    ParsingBlocking,
    AsapInOrder,
    Asap,
}
