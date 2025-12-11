/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::generic_channel::GenericSharedMemory;
use base::id::{PipelineId, WebViewId};
use content_security_policy::{self as csp};
use http::header::{AUTHORIZATION, HeaderName};
use http::{HeaderMap, Method};
use indexmap::IndexMap;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use malloc_size_of_derive::MallocSizeOf;
use mime::Mime;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};
use uuid::Uuid;

use crate::policy_container::{PolicyContainer, RequestPolicyContainer};
use crate::response::{HttpsState, Response};
use crate::{ReferrerPolicy, ResourceTimingType};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
/// An id to differeniate one network request from another.
pub struct RequestId(pub Uuid);

impl Default for RequestId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

/// An [initiator](https://fetch.spec.whatwg.org/#concept-request-initiator)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Initiator {
    None,
    Download,
    ImageSet,
    Manifest,
    XSLT,
    Prefetch,
    Link,
}

/// A request [destination](https://fetch.spec.whatwg.org/#concept-request-destination)
pub use csp::Destination;

/// A request [origin](https://fetch.spec.whatwg.org/#concept-request-origin)
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Origin {
    Client,
    Origin(ImmutableOrigin),
}

impl Origin {
    pub fn is_opaque(&self) -> bool {
        matches!(self, Origin::Origin(ImmutableOrigin::Opaque(_)))
    }
}

/// A [referer](https://fetch.spec.whatwg.org/#concept-request-referrer)
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Referrer {
    NoReferrer,
    /// Contains the url that "client" would be resolved to. See
    /// [https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer](https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer)
    ///
    /// If you are unsure you should probably use
    /// [`GlobalScope::get_referrer`](https://doc.servo.org/script/dom/globalscope/struct.GlobalScope.html#method.get_referrer)
    Client(ServoUrl),
    ReferrerUrl(ServoUrl),
}

/// A [request mode](https://fetch.spec.whatwg.org/#concept-request-mode)
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum RequestMode {
    Navigate,
    SameOrigin,
    NoCors,
    CorsMode,
    WebSocket {
        protocols: Vec<String>,
        original_url: ServoUrl,
    },
}

/// Request [credentials mode](https://fetch.spec.whatwg.org/#concept-request-credentials-mode)
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum CredentialsMode {
    Omit,
    CredentialsSameOrigin,
    Include,
}

/// [Cache mode](https://fetch.spec.whatwg.org/#concept-request-cache-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CacheMode {
    Default,
    NoStore,
    Reload,
    NoCache,
    ForceCache,
    OnlyIfCached,
}

/// [Service-workers mode](https://fetch.spec.whatwg.org/#request-service-workers-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ServiceWorkersMode {
    All,
    None,
}

/// [Redirect mode](https://fetch.spec.whatwg.org/#concept-request-redirect-mode)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum RedirectMode {
    Follow,
    Error,
    Manual,
}

/// [Response tainting](https://fetch.spec.whatwg.org/#concept-request-response-tainting)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ResponseTainting {
    Basic,
    CorsTainting,
    Opaque,
}

/// <https://html.spec.whatwg.org/multipage/#preload-key>
#[derive(Clone, Debug, Eq, Hash, Deserialize, MallocSizeOf, Serialize, PartialEq)]
pub struct PreloadKey {
    /// <https://html.spec.whatwg.org/multipage/#preload-url>
    url: ServoUrl,
    /// <https://html.spec.whatwg.org/multipage/#preload-destination>
    destination: Destination,
    /// <https://html.spec.whatwg.org/multipage/#preload-mode>
    mode: RequestMode,
    /// <https://html.spec.whatwg.org/multipage/#preload-credentials-mode>
    credentials_mode: CredentialsMode,
}

impl PreloadKey {
    pub fn new(request: &RequestBuilder) -> Self {
        Self {
            url: request.url.clone(),
            destination: request.destination,
            mode: request.mode.clone(),
            credentials_mode: request.credentials_mode,
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#preload-entry>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PreloadEntry {
    /// <https://html.spec.whatwg.org/multipage/#preload-integrity-metadata>
    integrity_metadata: String,
    /// <https://html.spec.whatwg.org/multipage/#preload-response>
    #[serde(skip)]
    response: Option<Response>,
}

impl PreloadEntry {
    pub fn new(integrity_metadata: String) -> Self {
        Self {
            integrity_metadata,
            response: None,
        }
    }

    pub fn with_response(&self, response: Response) -> Self {
        Self {
            integrity_metadata: self.integrity_metadata.clone(),
            response: Some(response),
        }
    }
}

pub type PreloadedResources = Arc<Mutex<IndexMap<PreloadKey, PreloadEntry>>>;

/// <https://fetch.spec.whatwg.org/#concept-request-client>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct RequestClient {
    /// <https://html.spec.whatwg.org/multipage/#map-of-preloaded-resources>
    #[conditional_malloc_size_of]
    #[serde(skip)] // TODO: Figure out what we need to do here to serialize this map
    pub preloaded_resources: PreloadedResources,
    /// <https://html.spec.whatwg.org/multipage/#concept-settings-object-policy-container>
    pub policy_container: RequestPolicyContainer,
    /// <https://html.spec.whatwg.org/multipage/#concept-settings-object-origin>
    pub origin: Origin,
}

impl RequestClient {
    /// <https://html.spec.whatwg.org/multipage/#consume-a-preloaded-resource>
    pub fn consume_preloaded_resource(
        &self,
        request: &Request,
        on_response_available: impl FnOnce(Response),
    ) -> bool {
        // Step 1. Let key be a preload key whose URL is url,
        // destination is destination, mode is mode, and credentials mode is credentialsMode.
        let key = PreloadKey {
            url: request.url().clone(),
            destination: request.destination,
            mode: request.mode.clone(),
            credentials_mode: request.credentials_mode,
        };
        // Step 2. Let preloads be window's associated Document's map of preloaded resources.
        let mut preloads = self.preloaded_resources.lock();
        // Step 4. Let entry be preloads[key].
        let Some(entry) = preloads.get_mut(&key) else {
            // Step 3. If key does not exist in preloads, then return false.
            return false;
        };
        // Step 5. Let consumerIntegrityMetadata be the result of parsing integrityMetadata.
        let consumer_integrity_metadata =
            csp::parse_subresource_integrity_metadata(&request.integrity_metadata);
        // Step 6. Let preloadIntegrityMetadata be the result of parsing entry's integrity metadata.
        let preload_integrity_metadata =
            csp::parse_subresource_integrity_metadata(&entry.integrity_metadata);
        // Step 7. If none of the following conditions apply:
        if !(
            // consumerIntegrityMetadata is no metadata;
            consumer_integrity_metadata == csp::SubresourceIntegrityMetadata::NoMetadata
            // consumerIntegrityMetadata is equal to preloadIntegrityMetadata; or
            || consumer_integrity_metadata == preload_integrity_metadata
        ) {
            // then return false.
            return false;
        }
        // Step 10. Otherwise, call onResponseAvailable with entry's response.
        if let Some(response) = entry.response.as_ref() {
            on_response_available(response.clone());
        } else {
            // Step 9. If entry's response is null, then set entry's on response available to onResponseAvailable.
            // TODO
        }
        // Step 8. Remove preloads[key].
        //
        // Moved down to avoid double borrow on preloads with entry
        preloads.shift_remove(&key);
        // Step 11. Return true.
        true
    }
}

/// <https://html.spec.whatwg.org/multipage/#system-visibility-state>
#[derive(Clone, Copy, Default, MallocSizeOf, PartialEq)]
pub enum SystemVisibilityState {
    #[default]
    Hidden,
    Visible,
}

/// <https://html.spec.whatwg.org/multipage/#traversable-navigable>
#[derive(Clone, Copy, Default, MallocSizeOf, PartialEq)]
pub struct TraversableNavigable {
    /// <https://html.spec.whatwg.org/multipage/#tn-current-session-history-step>
    current_session_history_step: u8,
    // TODO: https://html.spec.whatwg.org/multipage/#tn-session-history-entries
    // TODO: https://html.spec.whatwg.org/multipage/#tn-session-history-traversal-queue
    /// <https://html.spec.whatwg.org/multipage/#tn-running-nested-apply-history-step>
    running_nested_apply_history_step: bool,
    /// <https://html.spec.whatwg.org/multipage/#system-visibility-state>
    system_visibility_state: SystemVisibilityState,
    /// <https://html.spec.whatwg.org/multipage/#is-created-by-web-content>
    is_created_by_web_content: bool,
}

/// <https://fetch.spec.whatwg.org/#concept-request-window>
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum TraversableForUserPrompts {
    NoTraversable,
    Client,
    TraversableNavigable(TraversableNavigable),
}

/// [CORS settings attribute](https://html.spec.whatwg.org/multipage/#attr-crossorigin-anonymous)
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum CorsSettings {
    Anonymous,
    UseCredentials,
}

impl CorsSettings {
    /// <https://html.spec.whatwg.org/multipage/#cors-settings-attribute>
    pub fn from_enumerated_attribute(value: &str) -> CorsSettings {
        match value.to_ascii_lowercase().as_str() {
            "anonymous" => CorsSettings::Anonymous,
            "use-credentials" => CorsSettings::UseCredentials,
            _ => CorsSettings::Anonymous,
        }
    }
}

/// [Parser Metadata](https://fetch.spec.whatwg.org/#concept-request-parser-metadata)
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ParserMetadata {
    Default,
    ParserInserted,
    NotParserInserted,
}

/// <https://fetch.spec.whatwg.org/#concept-body-source>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum BodySource {
    Null,
    Object,
}

/// Messages used to implement <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
/// which are sent from script to net.
#[derive(Debug, Deserialize, Serialize)]
pub enum BodyChunkResponse {
    /// A chunk of bytes.
    Chunk(GenericSharedMemory),
    /// The body is done.
    Done,
    /// There was an error streaming the body,
    /// terminate fetch.
    Error,
}

/// Messages used to implement <https://fetch.spec.whatwg.org/#concept-request-transmit-body>
/// which are sent from net to script
/// (with the exception of Done, which is sent from script to script).
#[derive(Debug, Deserialize, Serialize)]
pub enum BodyChunkRequest {
    /// Connect a fetch in `net`, with a stream of bytes from `script`.
    Connect(IpcSender<BodyChunkResponse>),
    /// Re-extract a new stream from the source, following a redirect.
    Extract(IpcReceiver<BodyChunkRequest>),
    /// Ask for another chunk.
    Chunk,
    /// Signal the stream is done(sent from script to script).
    Done,
    /// Signal the stream has errored(sent from script to script).
    Error,
}

/// The net component's view into <https://fetch.spec.whatwg.org/#bodies>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct RequestBody {
    /// Net's channel to communicate with script re this body.
    #[ignore_malloc_size_of = "Channels are hard"]
    chan: Arc<Mutex<IpcSender<BodyChunkRequest>>>,
    /// <https://fetch.spec.whatwg.org/#concept-body-source>
    source: BodySource,
    /// <https://fetch.spec.whatwg.org/#concept-body-total-bytes>
    total_bytes: Option<usize>,
}

impl RequestBody {
    pub fn new(
        chan: IpcSender<BodyChunkRequest>,
        source: BodySource,
        total_bytes: Option<usize>,
    ) -> Self {
        RequestBody {
            chan: Arc::new(Mutex::new(chan)),
            source,
            total_bytes,
        }
    }

    /// Step 12 of <https://fetch.spec.whatwg.org/#concept-http-redirect-fetch>
    pub fn extract_source(&mut self) {
        match self.source {
            BodySource::Null => panic!("Null sources should never be re-directed."),
            BodySource::Object => {
                let (chan, port) = ipc::channel().unwrap();
                let mut selfchan = self.chan.lock();
                let _ = selfchan.send(BodyChunkRequest::Extract(port));
                *selfchan = chan;
            },
        }
    }

    pub fn take_stream(&self) -> Arc<Mutex<IpcSender<BodyChunkRequest>>> {
        self.chan.clone()
    }

    pub fn source_is_null(&self) -> bool {
        self.source == BodySource::Null
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> Option<usize> {
        self.total_bytes
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum InsecureRequestsPolicy {
    DoNotUpgrade,
    Upgrade,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct RequestBuilder {
    pub id: RequestId,

    /// <https://fetch.spec.whatwg.org/#concept-request-method>
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub method: Method,

    /// <https://fetch.spec.whatwg.org/#concept-request-url>
    pub url: ServoUrl,

    /// <https://fetch.spec.whatwg.org/#concept-request-header-list>
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub headers: HeaderMap,

    /// <https://fetch.spec.whatwg.org/#unsafe-request-flag>
    pub unsafe_request: bool,

    /// <https://fetch.spec.whatwg.org/#concept-request-body>
    pub body: Option<RequestBody>,

    /// <https://fetch.spec.whatwg.org/#request-service-workers-mode>
    pub service_workers_mode: ServiceWorkersMode,
    pub client: Option<RequestClient>,
    /// <https://fetch.spec.whatwg.org/#concept-request-destination>
    pub destination: Destination,
    pub synchronous: bool,
    pub mode: RequestMode,

    /// <https://fetch.spec.whatwg.org/#concept-request-cache-mode>
    pub cache_mode: CacheMode,

    /// <https://fetch.spec.whatwg.org/#use-cors-preflight-flag>
    pub use_cors_preflight: bool,

    /// <https://fetch.spec.whatwg.org/#concept-request-credentials-mode>
    pub credentials_mode: CredentialsMode,
    pub use_url_credentials: bool,

    /// <https://fetch.spec.whatwg.org/#concept-request-origin>
    pub origin: Origin,

    /// <https://fetch.spec.whatwg.org/#concept-request-policy-container>
    pub policy_container: RequestPolicyContainer,
    pub insecure_requests_policy: InsecureRequestsPolicy,
    pub has_trustworthy_ancestor_origin: bool,

    /// <https://fetch.spec.whatwg.org/#concept-request-referrer>
    pub referrer: Referrer,

    /// <https://fetch.spec.whatwg.org/#concept-request-referrer-policy>
    pub referrer_policy: ReferrerPolicy,
    pub pipeline_id: Option<PipelineId>,
    pub target_webview_id: Option<WebViewId>,

    /// <https://fetch.spec.whatwg.org/#concept-request-redirect-mode>
    pub redirect_mode: RedirectMode,

    /// <https://fetch.spec.whatwg.org/#concept-request-integrity-metadata>
    pub integrity_metadata: String,

    /// <https://fetch.spec.whatwg.org/#concept-request-nonce-metadata>
    pub cryptographic_nonce_metadata: String,

    // to keep track of redirects
    pub url_list: Vec<ServoUrl>,

    /// <https://fetch.spec.whatwg.org/#concept-request-parser-metadata>
    pub parser_metadata: ParserMetadata,

    /// <https://fetch.spec.whatwg.org/#concept-request-initiator>
    pub initiator: Initiator,
    pub https_state: HttpsState,
    pub response_tainting: ResponseTainting,
    /// Servo internal: if crash details are present, trigger a crash error page with these details.
    pub crash: Option<String>,
}

impl RequestBuilder {
    pub fn new(webview_id: Option<WebViewId>, url: ServoUrl, referrer: Referrer) -> RequestBuilder {
        RequestBuilder {
            id: RequestId::default(),
            method: Method::GET,
            url,
            headers: HeaderMap::new(),
            unsafe_request: false,
            body: None,
            service_workers_mode: ServiceWorkersMode::All,
            destination: Destination::None,
            synchronous: false,
            mode: RequestMode::NoCors,
            cache_mode: CacheMode::Default,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::CredentialsSameOrigin,
            use_url_credentials: false,
            origin: Origin::Client,
            client: None,
            policy_container: RequestPolicyContainer::default(),
            insecure_requests_policy: InsecureRequestsPolicy::DoNotUpgrade,
            has_trustworthy_ancestor_origin: false,
            referrer,
            referrer_policy: ReferrerPolicy::EmptyString,
            pipeline_id: None,
            target_webview_id: webview_id,
            redirect_mode: RedirectMode::Follow,
            integrity_metadata: "".to_owned(),
            cryptographic_nonce_metadata: "".to_owned(),
            url_list: vec![],
            parser_metadata: ParserMetadata::Default,
            initiator: Initiator::None,
            https_state: HttpsState::None,
            response_tainting: ResponseTainting::Basic,
            crash: None,
        }
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-initiator>
    pub fn initiator(mut self, initiator: Initiator) -> RequestBuilder {
        self.initiator = initiator;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-method>
    pub fn method(mut self, method: Method) -> RequestBuilder {
        self.method = method;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-header-list>
    pub fn headers(mut self, headers: HeaderMap) -> RequestBuilder {
        self.headers = headers;
        self
    }

    /// <https://fetch.spec.whatwg.org/#unsafe-request-flag>
    pub fn unsafe_request(mut self, unsafe_request: bool) -> RequestBuilder {
        self.unsafe_request = unsafe_request;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-body>
    pub fn body(mut self, body: Option<RequestBody>) -> RequestBuilder {
        self.body = body;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-destination>
    pub fn destination(mut self, destination: Destination) -> RequestBuilder {
        self.destination = destination;
        self
    }

    pub fn synchronous(mut self, synchronous: bool) -> RequestBuilder {
        self.synchronous = synchronous;
        self
    }

    pub fn mode(mut self, mode: RequestMode) -> RequestBuilder {
        self.mode = mode;
        self
    }

    /// <https://fetch.spec.whatwg.org/#use-cors-preflight-flag>
    pub fn use_cors_preflight(mut self, use_cors_preflight: bool) -> RequestBuilder {
        self.use_cors_preflight = use_cors_preflight;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-credentials-mode>
    pub fn credentials_mode(mut self, credentials_mode: CredentialsMode) -> RequestBuilder {
        self.credentials_mode = credentials_mode;
        self
    }

    pub fn use_url_credentials(mut self, use_url_credentials: bool) -> RequestBuilder {
        self.use_url_credentials = use_url_credentials;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-origin>
    pub fn origin(mut self, origin: ImmutableOrigin) -> RequestBuilder {
        self.origin = Origin::Origin(origin);
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-referrer-policy>
    pub fn referrer_policy(mut self, referrer_policy: ReferrerPolicy) -> RequestBuilder {
        self.referrer_policy = referrer_policy;
        self
    }

    pub fn pipeline_id(mut self, pipeline_id: Option<PipelineId>) -> RequestBuilder {
        self.pipeline_id = pipeline_id;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-redirect-mode>
    pub fn redirect_mode(mut self, redirect_mode: RedirectMode) -> RequestBuilder {
        self.redirect_mode = redirect_mode;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-integrity-metadata>
    pub fn integrity_metadata(mut self, integrity_metadata: String) -> RequestBuilder {
        self.integrity_metadata = integrity_metadata;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-nonce-metadata>
    pub fn cryptographic_nonce_metadata(mut self, nonce_metadata: String) -> RequestBuilder {
        self.cryptographic_nonce_metadata = nonce_metadata;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-parser-metadata>
    pub fn parser_metadata(mut self, parser_metadata: ParserMetadata) -> RequestBuilder {
        self.parser_metadata = parser_metadata;
        self
    }

    pub fn https_state(mut self, https_state: HttpsState) -> RequestBuilder {
        self.https_state = https_state;
        self
    }

    pub fn response_tainting(mut self, response_tainting: ResponseTainting) -> RequestBuilder {
        self.response_tainting = response_tainting;
        self
    }

    pub fn crash(mut self, crash: Option<String>) -> Self {
        self.crash = crash;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-policy-container>
    pub fn policy_container(mut self, policy_container: PolicyContainer) -> RequestBuilder {
        self.policy_container = RequestPolicyContainer::PolicyContainer(policy_container);
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-client>
    pub fn client(mut self, client: RequestClient) -> RequestBuilder {
        self.client = Some(client);
        self
    }

    pub fn insecure_requests_policy(
        mut self,
        insecure_requests_policy: InsecureRequestsPolicy,
    ) -> RequestBuilder {
        self.insecure_requests_policy = insecure_requests_policy;
        self
    }

    pub fn has_trustworthy_ancestor_origin(
        mut self,
        has_trustworthy_ancestor_origin: bool,
    ) -> RequestBuilder {
        self.has_trustworthy_ancestor_origin = has_trustworthy_ancestor_origin;
        self
    }

    /// <https://fetch.spec.whatwg.org/#request-service-workers-mode>
    pub fn service_workers_mode(
        mut self,
        service_workers_mode: ServiceWorkersMode,
    ) -> RequestBuilder {
        self.service_workers_mode = service_workers_mode;
        self
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-cache-mode>
    pub fn cache_mode(mut self, cache_mode: CacheMode) -> RequestBuilder {
        self.cache_mode = cache_mode;
        self
    }

    pub fn build(self) -> Request {
        let mut request = Request::new(
            self.id,
            self.url.clone(),
            Some(self.origin),
            self.referrer,
            self.pipeline_id,
            self.target_webview_id,
            self.https_state,
        );
        request.initiator = self.initiator;
        request.method = self.method;
        request.headers = self.headers;
        request.unsafe_request = self.unsafe_request;
        request.body = self.body;
        request.service_workers_mode = self.service_workers_mode;
        request.destination = self.destination;
        request.synchronous = self.synchronous;
        request.mode = self.mode;
        request.use_cors_preflight = self.use_cors_preflight;
        request.credentials_mode = self.credentials_mode;
        request.use_url_credentials = self.use_url_credentials;
        request.cache_mode = self.cache_mode;
        request.referrer_policy = self.referrer_policy;
        request.redirect_mode = self.redirect_mode;
        let mut url_list = self.url_list;
        if url_list.is_empty() {
            url_list.push(self.url);
        }
        request.redirect_count = url_list.len() as u32 - 1;
        request.url_list = url_list;
        request.integrity_metadata = self.integrity_metadata;
        request.cryptographic_nonce_metadata = self.cryptographic_nonce_metadata;
        request.parser_metadata = self.parser_metadata;
        request.response_tainting = self.response_tainting;
        request.crash = self.crash;
        request.client = self.client;
        request.policy_container = self.policy_container;
        request.insecure_requests_policy = self.insecure_requests_policy;
        request.has_trustworthy_ancestor_origin = self.has_trustworthy_ancestor_origin;
        request
    }
}

/// A [Request](https://fetch.spec.whatwg.org/#concept-request) as defined by
/// the Fetch spec.
#[derive(Clone, MallocSizeOf)]
pub struct Request {
    /// The unique id of this request so that the task that triggered it can route
    /// messages to the correct listeners. This is a UUID that is generated when a request
    /// is being built.
    pub id: RequestId,
    /// <https://fetch.spec.whatwg.org/#concept-request-method>
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub method: Method,
    /// <https://fetch.spec.whatwg.org/#local-urls-only-flag>
    pub local_urls_only: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-header-list>
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub headers: HeaderMap,
    /// <https://fetch.spec.whatwg.org/#unsafe-request-flag>
    pub unsafe_request: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-body>
    pub body: Option<RequestBody>,
    /// <https://fetch.spec.whatwg.org/#concept-request-client>
    pub client: Option<RequestClient>,
    /// <https://fetch.spec.whatwg.org/#concept-request-window>
    pub traversable_for_user_prompts: TraversableForUserPrompts,
    pub target_webview_id: Option<WebViewId>,
    /// <https://fetch.spec.whatwg.org/#request-keepalive-flag>
    pub keep_alive: bool,
    /// <https://fetch.spec.whatwg.org/#request-service-workers-mode>
    pub service_workers_mode: ServiceWorkersMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-initiator>
    pub initiator: Initiator,
    /// <https://fetch.spec.whatwg.org/#concept-request-destination>
    pub destination: Destination,
    // TODO: priority object
    /// <https://fetch.spec.whatwg.org/#concept-request-origin>
    pub origin: Origin,
    /// <https://fetch.spec.whatwg.org/#concept-request-referrer>
    pub referrer: Referrer,
    /// <https://fetch.spec.whatwg.org/#concept-request-referrer-policy>
    pub referrer_policy: ReferrerPolicy,
    pub pipeline_id: Option<PipelineId>,
    /// <https://fetch.spec.whatwg.org/#synchronous-flag>
    pub synchronous: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-mode>
    pub mode: RequestMode,
    /// <https://fetch.spec.whatwg.org/#use-cors-preflight-flag>
    pub use_cors_preflight: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-credentials-mode>
    pub credentials_mode: CredentialsMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-use-url-credentials-flag>
    pub use_url_credentials: bool,
    /// <https://fetch.spec.whatwg.org/#concept-request-cache-mode>
    pub cache_mode: CacheMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-redirect-mode>
    pub redirect_mode: RedirectMode,
    /// <https://fetch.spec.whatwg.org/#concept-request-integrity-metadata>
    pub integrity_metadata: String,
    /// <https://fetch.spec.whatwg.org/#concept-request-nonce-metadata>
    pub cryptographic_nonce_metadata: String,
    // Use the last method on url_list to act as spec current url field, and
    // first method to act as spec url field
    /// <https://fetch.spec.whatwg.org/#concept-request-url-list>
    pub url_list: Vec<ServoUrl>,
    /// <https://fetch.spec.whatwg.org/#concept-request-redirect-count>
    pub redirect_count: u32,
    /// <https://fetch.spec.whatwg.org/#concept-request-response-tainting>
    pub response_tainting: ResponseTainting,
    /// <https://fetch.spec.whatwg.org/#concept-request-parser-metadata>
    pub parser_metadata: ParserMetadata,
    /// <https://fetch.spec.whatwg.org/#concept-request-policy-container>
    pub policy_container: RequestPolicyContainer,
    /// <https://w3c.github.io/webappsec-upgrade-insecure-requests/#insecure-requests-policy>
    pub insecure_requests_policy: InsecureRequestsPolicy,
    pub has_trustworthy_ancestor_origin: bool,
    pub https_state: HttpsState,
    /// Servo internal: if crash details are present, trigger a crash error page with these details.
    pub crash: Option<String>,
}

impl Request {
    pub fn new(
        id: RequestId,
        url: ServoUrl,
        origin: Option<Origin>,
        referrer: Referrer,
        pipeline_id: Option<PipelineId>,
        webview_id: Option<WebViewId>,
        https_state: HttpsState,
    ) -> Request {
        Request {
            id,
            method: Method::GET,
            local_urls_only: false,
            headers: HeaderMap::new(),
            unsafe_request: false,
            body: None,
            client: None,
            traversable_for_user_prompts: TraversableForUserPrompts::Client,
            keep_alive: false,
            service_workers_mode: ServiceWorkersMode::All,
            initiator: Initiator::None,
            destination: Destination::None,
            origin: origin.unwrap_or(Origin::Client),
            referrer,
            referrer_policy: ReferrerPolicy::EmptyString,
            pipeline_id,
            target_webview_id: webview_id,
            synchronous: false,
            mode: RequestMode::NoCors,
            use_cors_preflight: false,
            credentials_mode: CredentialsMode::CredentialsSameOrigin,
            use_url_credentials: false,
            cache_mode: CacheMode::Default,
            redirect_mode: RedirectMode::Follow,
            integrity_metadata: String::new(),
            cryptographic_nonce_metadata: String::new(),
            url_list: vec![url],
            parser_metadata: ParserMetadata::Default,
            redirect_count: 0,
            response_tainting: ResponseTainting::Basic,
            policy_container: RequestPolicyContainer::Client,
            insecure_requests_policy: InsecureRequestsPolicy::DoNotUpgrade,
            has_trustworthy_ancestor_origin: false,
            https_state,
            crash: None,
        }
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-url>
    pub fn url(&self) -> ServoUrl {
        self.url_list.first().unwrap().clone()
    }

    pub fn original_url(&self) -> ServoUrl {
        match self.mode {
            RequestMode::WebSocket {
                protocols: _,
                ref original_url,
            } => original_url.clone(),
            _ => self.url(),
        }
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-current-url>
    pub fn current_url(&self) -> ServoUrl {
        self.url_list.last().unwrap().clone()
    }

    /// <https://fetch.spec.whatwg.org/#concept-request-current-url>
    pub fn current_url_mut(&mut self) -> &mut ServoUrl {
        self.url_list.last_mut().unwrap()
    }

    /// <https://fetch.spec.whatwg.org/#navigation-request>
    pub fn is_navigation_request(&self) -> bool {
        matches!(
            self.destination,
            Destination::Document |
                Destination::Embed |
                Destination::Frame |
                Destination::IFrame |
                Destination::Object
        )
    }

    /// <https://fetch.spec.whatwg.org/#subresource-request>
    pub fn is_subresource_request(&self) -> bool {
        matches!(
            self.destination,
            Destination::Audio |
                Destination::Font |
                Destination::Image |
                Destination::Manifest |
                Destination::Script |
                Destination::Style |
                Destination::Track |
                Destination::Video |
                Destination::Xslt |
                Destination::None
        )
    }

    pub fn timing_type(&self) -> ResourceTimingType {
        if self.is_navigation_request() {
            ResourceTimingType::Navigation
        } else {
            ResourceTimingType::Resource
        }
    }

    /// <https://fetch.spec.whatwg.org/#populate-request-from-client>
    pub fn populate_request_from_client(&mut self) {
        // Step 1. If request’s traversable for user prompts is "client":
        if self.traversable_for_user_prompts == TraversableForUserPrompts::Client {
            // Step 1.1. Set request’s traversable for user prompts to "no-traversable".
            self.traversable_for_user_prompts = TraversableForUserPrompts::NoTraversable;
            // Step 1.2. If request’s client is non-null:
            if self.client.is_some() {
                // Step 1.2.1. Let global be request’s client’s global object.
                // TODO
                // Step 1.2.2. If global is a Window object and global’s navigable is not null,
                // then set request’s traversable for user prompts to global’s navigable’s traversable navigable.
                self.traversable_for_user_prompts =
                    TraversableForUserPrompts::TraversableNavigable(Default::default());
            }
        }
        // Step 2. If request’s origin is "client":
        if self.origin == Origin::Client {
            let Some(client) = self.client.as_ref() else {
                // Step 2.1. Assert: request’s client is non-null.
                unreachable!();
            };
            // Step 2.2. Set request’s origin to request’s client’s origin.
            self.origin = client.origin.clone();
        }
        // Step 3. If request’s policy container is "client":
        if matches!(self.policy_container, RequestPolicyContainer::Client) {
            // Step 3.1. If request’s client is non-null, then set request’s
            // policy container to a clone of request’s client’s policy container. [HTML]
            if let Some(client) = self.client.as_ref() {
                self.policy_container = client.policy_container.clone();
            } else {
                // Step 3.2. Otherwise, set request’s policy container to a new policy container.
                self.policy_container =
                    RequestPolicyContainer::PolicyContainer(PolicyContainer::default());
            }
        }
    }
}

impl Referrer {
    pub fn to_url(&self) -> Option<&ServoUrl> {
        match *self {
            Referrer::NoReferrer => None,
            Referrer::Client(ref url) => Some(url),
            Referrer::ReferrerUrl(ref url) => Some(url),
        }
    }
}

// https://fetch.spec.whatwg.org/#cors-unsafe-request-header-byte
// TODO: values in the control-code range are being quietly stripped out by
// HeaderMap and never reach this function to be loudly rejected!
fn is_cors_unsafe_request_header_byte(value: &u8) -> bool {
    matches!(value,
        0x00..=0x08 |
        0x10..=0x19 |
        0x22 |
        0x28 |
        0x29 |
        0x3A |
        0x3C |
        0x3E |
        0x3F |
        0x40 |
        0x5B |
        0x5C |
        0x5D |
        0x7B |
        0x7D |
        0x7F
    )
}

// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
// subclause `accept`
fn is_cors_safelisted_request_accept(value: &[u8]) -> bool {
    !(value.iter().any(is_cors_unsafe_request_header_byte))
}

// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
// subclauses `accept-language`, `content-language`
fn is_cors_safelisted_language(value: &[u8]) -> bool {
    value.iter().all(|&x| {
        matches!(x,
            0x30..=0x39 |
            0x41..=0x5A |
            0x61..=0x7A |
            0x20 |
            0x2A |
            0x2C |
            0x2D |
            0x2E |
            0x3B |
            0x3D
        )
    })
}

// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
// subclause `content-type`
pub fn is_cors_safelisted_request_content_type(value: &[u8]) -> bool {
    // step 1
    if value.iter().any(is_cors_unsafe_request_header_byte) {
        return false;
    }
    // step 2
    let value_string = if let Ok(s) = std::str::from_utf8(value) {
        s
    } else {
        return false;
    };
    let value_mime_result: Result<Mime, _> = value_string.parse();
    match value_mime_result {
        Err(_) => false, // step 3
        Ok(value_mime) => match (value_mime.type_(), value_mime.subtype()) {
            (mime::APPLICATION, mime::WWW_FORM_URLENCODED) |
            (mime::MULTIPART, mime::FORM_DATA) |
            (mime::TEXT, mime::PLAIN) => true,
            _ => false, // step 4
        },
    }
}

// TODO: "DPR", "Downlink", "Save-Data", "Viewport-Width", "Width":
// ... once parsed, the value should not be failure.
// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
pub fn is_cors_safelisted_request_header<N: AsRef<str>, V: AsRef<[u8]>>(
    name: &N,
    value: &V,
) -> bool {
    let name: &str = name.as_ref();
    let value: &[u8] = value.as_ref();
    if value.len() > 128 {
        return false;
    }
    match name {
        "accept" => is_cors_safelisted_request_accept(value),
        "accept-language" | "content-language" => is_cors_safelisted_language(value),
        "content-type" => is_cors_safelisted_request_content_type(value),
        "range" => is_cors_safelisted_request_range(value),
        _ => false,
    }
}

pub fn is_cors_safelisted_request_range(value: &[u8]) -> bool {
    if let Ok(value_str) = std::str::from_utf8(value) {
        return validate_range_header(value_str);
    }
    false
}

fn validate_range_header(value: &str) -> bool {
    let trimmed = value.trim();
    if !trimmed.starts_with("bytes=") {
        return false;
    }

    if let Some(range) = trimmed.strip_prefix("bytes=") {
        let mut parts = range.split('-');
        let start = parts.next();
        let end = parts.next();

        if let Some(start) = start {
            if let Ok(start_num) = start.parse::<u64>() {
                return match end {
                    Some(e) if !e.is_empty() => {
                        e.parse::<u64>().is_ok_and(|end_num| start_num <= end_num)
                    },
                    _ => true,
                };
            }
        }
    }
    false
}

/// <https://fetch.spec.whatwg.org/#cors-safelisted-method>
pub fn is_cors_safelisted_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::POST)
}

/// <https://fetch.spec.whatwg.org/#cors-non-wildcard-request-header-name>
pub fn is_cors_non_wildcard_request_header_name(name: &HeaderName) -> bool {
    name == AUTHORIZATION
}

/// <https://fetch.spec.whatwg.org/#cors-unsafe-request-header-names>
pub fn get_cors_unsafe_header_names(headers: &HeaderMap) -> Vec<HeaderName> {
    // Step 1
    let mut unsafe_names: Vec<&HeaderName> = vec![];
    // Step 2
    let mut potentillay_unsafe_names: Vec<&HeaderName> = vec![];
    // Step 3
    let mut safelist_value_size = 0;

    // Step 4
    for (name, value) in headers.iter() {
        if !is_cors_safelisted_request_header(&name, &value) {
            unsafe_names.push(name);
        } else {
            potentillay_unsafe_names.push(name);
            safelist_value_size += value.as_ref().len();
        }
    }

    // Step 5
    if safelist_value_size > 1024 {
        unsafe_names.extend_from_slice(&potentillay_unsafe_names);
    }

    // Step 6
    convert_header_names_to_sorted_lowercase_set(unsafe_names)
}

/// <https://fetch.spec.whatwg.org/#ref-for-convert-header-names-to-a-sorted-lowercase-set>
pub fn convert_header_names_to_sorted_lowercase_set(
    header_names: Vec<&HeaderName>,
) -> Vec<HeaderName> {
    // HeaderName does not implement the needed traits to use a BTreeSet
    // So create a new Vec, sort, then dedup
    let mut ordered_set = header_names.to_vec();
    ordered_set.sort_by(|a, b| a.as_str().partial_cmp(b.as_str()).unwrap());
    ordered_set.dedup();
    ordered_set.into_iter().cloned().collect()
}

pub fn create_request_body_with_content(content: &str) -> RequestBody {
    let content_bytes = GenericSharedMemory::from_bytes(content.as_bytes());
    let content_len = content_bytes.len();

    let (chunk_request_sender, chunk_request_receiver) = ipc::channel().unwrap();
    ROUTER.add_typed_route(
        chunk_request_receiver,
        Box::new(move |message| {
            let request = message.unwrap();
            if let BodyChunkRequest::Connect(sender) = request {
                let _ = sender.send(BodyChunkResponse::Chunk(content_bytes.clone()));
                let _ = sender.send(BodyChunkResponse::Done);
            }
        }),
    );

    RequestBody::new(chunk_request_sender, BodySource::Object, Some(content_len))
}
