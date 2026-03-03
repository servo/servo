/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::fmt::{self, Debug, Display};
use std::sync::{LazyLock, OnceLock};
use std::thread::{self, JoinHandle};

use base::cross_process_instant::CrossProcessInstant;
use base::generic_channel::{self, GenericOneshotSender, GenericSend, GenericSender, SendResult};
use base::id::{CookieStoreId, HistoryStateId, PipelineId};
use content_security_policy::{self as csp};
use cookie::Cookie;
use crossbeam_channel::{Receiver, Sender, unbounded};
use headers::{ContentType, HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use http::{HeaderMap, HeaderValue, StatusCode, header};
use hyper_serde::Serde;
use hyper_util::client::legacy::Error as HyperError;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use mime::Mime;
use profile_traits::mem::ReportsChan;
use rand::{RngCore, rng};
use request::RequestId;
use rustc_hash::FxHashMap;
use rustls_pki_types::CertificateDer;
use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::fetch::headers::determine_nosniff;
use crate::filemanager_thread::FileManagerThreadMsg;
use crate::http_status::HttpStatus;
use crate::mime_classifier::{ApacheBugFlag, MimeClassifier};
use crate::request::{PreloadId, Request, RequestBuilder};
use crate::response::{HttpsState, Response, ResponseInit};

pub mod blob_url_store;
pub mod filemanager_thread;
pub mod http_status;
pub mod image_cache;
pub mod mime_classifier;
pub mod policy_container;
pub mod pub_domains;
pub mod quality;
pub mod request;
pub mod response;

/// <https://fetch.spec.whatwg.org/#document-accept-header-value>
pub const DOCUMENT_ACCEPT_HEADER_VALUE: HeaderValue =
    HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8");

/// An implementation of the [Fetch specification](https://fetch.spec.whatwg.org/)
pub mod fetch {
    pub mod headers;
}

/// A loading context, for context-specific sniffing, as defined in
/// <https://mimesniff.spec.whatwg.org/#context-specific-sniffing>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum LoadContext {
    Browsing,
    Image,
    AudioVideo,
    Plugin,
    Style,
    Script,
    Font,
    TextTrack,
    CacheManifest,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct CustomResponse {
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    pub headers: HeaderMap,
    #[ignore_malloc_size_of = "Defined in hyper"]
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    pub raw_status: (StatusCode, String),
    pub body: Vec<u8>,
}

impl CustomResponse {
    pub fn new(
        headers: HeaderMap,
        raw_status: (StatusCode, String),
        body: Vec<u8>,
    ) -> CustomResponse {
        CustomResponse {
            headers,
            raw_status,
            body,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomResponseMediator {
    pub response_chan: IpcSender<Option<CustomResponse>>,
    pub load_url: ServoUrl,
}

/// [Policies](https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-states)
/// for providing a referrer header for a request
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ReferrerPolicy {
    /// ""
    EmptyString,
    /// "no-referrer"
    NoReferrer,
    /// "no-referrer-when-downgrade"
    NoReferrerWhenDowngrade,
    /// "origin"
    Origin,
    /// "same-origin"
    SameOrigin,
    /// "origin-when-cross-origin"
    OriginWhenCrossOrigin,
    /// "unsafe-url"
    UnsafeUrl,
    /// "strict-origin"
    StrictOrigin,
    /// "strict-origin-when-cross-origin"
    #[default]
    StrictOriginWhenCrossOrigin,
}

impl ReferrerPolicy {
    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    pub fn from_with_legacy(value: &str) -> Self {
        // Step 5. If value is one of the values given in the first column of the following table,
        // then set value to the value given in the second column:
        match value.to_ascii_lowercase().as_str() {
            "never" => ReferrerPolicy::NoReferrer,
            "default" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
            "always" => ReferrerPolicy::UnsafeUrl,
            "origin-when-crossorigin" => ReferrerPolicy::OriginWhenCrossOrigin,
            _ => ReferrerPolicy::from(value),
        }
    }

    /// <https://w3c.github.io/webappsec-referrer-policy/#parse-referrer-policy-from-header>
    pub fn parse_header_for_response(headers: &Option<Serde<HeaderMap>>) -> Self {
        // Step 4. Return policy.
        headers
            .as_ref()
            // Step 1. Let policy-tokens be the result of extracting header list values given `Referrer-Policy` and response’s header list.
            .and_then(|headers| headers.typed_get::<ReferrerPolicyHeader>())
            // Step 2-3.
            .into()
    }
}

impl From<&str> for ReferrerPolicy {
    /// <https://html.spec.whatwg.org/multipage/#referrer-policy-attribute>
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "no-referrer" => ReferrerPolicy::NoReferrer,
            "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
            "origin" => ReferrerPolicy::Origin,
            "same-origin" => ReferrerPolicy::SameOrigin,
            "strict-origin" => ReferrerPolicy::StrictOrigin,
            "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
            "origin-when-cross-origin" => ReferrerPolicy::OriginWhenCrossOrigin,
            "unsafe-url" => ReferrerPolicy::UnsafeUrl,
            _ => ReferrerPolicy::EmptyString,
        }
    }
}

impl Display for ReferrerPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ReferrerPolicy::EmptyString => "",
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
        };
        write!(formatter, "{string}")
    }
}

/// <https://w3c.github.io/webappsec-referrer-policy/#parse-referrer-policy-from-header>
impl From<Option<ReferrerPolicyHeader>> for ReferrerPolicy {
    fn from(header: Option<ReferrerPolicyHeader>) -> Self {
        // Step 2. Let policy be the empty string.
        // Step 3. For each token in policy-tokens, if token is a referrer policy and token is not the empty string, then set policy to token.
        header.map_or(ReferrerPolicy::EmptyString, |policy| match policy {
            ReferrerPolicyHeader::NO_REFERRER => ReferrerPolicy::NoReferrer,
            ReferrerPolicyHeader::NO_REFERRER_WHEN_DOWNGRADE => {
                ReferrerPolicy::NoReferrerWhenDowngrade
            },
            ReferrerPolicyHeader::SAME_ORIGIN => ReferrerPolicy::SameOrigin,
            ReferrerPolicyHeader::ORIGIN => ReferrerPolicy::Origin,
            ReferrerPolicyHeader::ORIGIN_WHEN_CROSS_ORIGIN => ReferrerPolicy::OriginWhenCrossOrigin,
            ReferrerPolicyHeader::UNSAFE_URL => ReferrerPolicy::UnsafeUrl,
            ReferrerPolicyHeader::STRICT_ORIGIN => ReferrerPolicy::StrictOrigin,
            ReferrerPolicyHeader::STRICT_ORIGIN_WHEN_CROSS_ORIGIN => {
                ReferrerPolicy::StrictOriginWhenCrossOrigin
            },
        })
    }
}

impl From<ReferrerPolicy> for ReferrerPolicyHeader {
    fn from(referrer_policy: ReferrerPolicy) -> Self {
        match referrer_policy {
            ReferrerPolicy::NoReferrer => ReferrerPolicyHeader::NO_REFERRER,
            ReferrerPolicy::NoReferrerWhenDowngrade => {
                ReferrerPolicyHeader::NO_REFERRER_WHEN_DOWNGRADE
            },
            ReferrerPolicy::SameOrigin => ReferrerPolicyHeader::SAME_ORIGIN,
            ReferrerPolicy::Origin => ReferrerPolicyHeader::ORIGIN,
            ReferrerPolicy::OriginWhenCrossOrigin => ReferrerPolicyHeader::ORIGIN_WHEN_CROSS_ORIGIN,
            ReferrerPolicy::UnsafeUrl => ReferrerPolicyHeader::UNSAFE_URL,
            ReferrerPolicy::StrictOrigin => ReferrerPolicyHeader::STRICT_ORIGIN,
            ReferrerPolicy::EmptyString | ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                ReferrerPolicyHeader::STRICT_ORIGIN_WHEN_CROSS_ORIGIN
            },
        }
    }
}

// FIXME: https://github.com/servo/servo/issues/34591
#[expect(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize)]
pub enum FetchResponseMsg {
    // todo: should have fields for transmitted/total bytes
    ProcessRequestBody(RequestId),
    ProcessRequestEOF(RequestId),
    // todo: send more info about the response (or perhaps the entire Response)
    ProcessResponse(RequestId, Result<FetchMetadata, NetworkError>),
    ProcessResponseChunk(RequestId, DebugVec),
    ProcessResponseEOF(RequestId, Result<(), NetworkError>, ResourceFetchTiming),
    ProcessCspViolations(RequestId, Vec<csp::Violation>),
}

#[derive(Deserialize, PartialEq, Serialize, MallocSizeOf)]
pub struct DebugVec(pub Vec<u8>);

impl From<Vec<u8>> for DebugVec {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl std::ops::Deref for DebugVec {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for DebugVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[...; {}]", self.0.len()))
    }
}

impl FetchResponseMsg {
    pub fn request_id(&self) -> RequestId {
        match self {
            FetchResponseMsg::ProcessRequestBody(id) |
            FetchResponseMsg::ProcessRequestEOF(id) |
            FetchResponseMsg::ProcessResponse(id, ..) |
            FetchResponseMsg::ProcessResponseChunk(id, ..) |
            FetchResponseMsg::ProcessResponseEOF(id, ..) |
            FetchResponseMsg::ProcessCspViolations(id, ..) => *id,
        }
    }
}

pub trait FetchTaskTarget {
    /// <https://fetch.spec.whatwg.org/#process-request-body>
    ///
    /// Fired when a chunk of the request body is transmitted
    fn process_request_body(&mut self, request: &Request);

    /// <https://fetch.spec.whatwg.org/#process-request-end-of-file>
    ///
    /// Fired when the entire request finishes being transmitted
    fn process_request_eof(&mut self, request: &Request);

    /// <https://fetch.spec.whatwg.org/#process-response>
    ///
    /// Fired when headers are received
    fn process_response(&mut self, request: &Request, response: &Response);

    /// Fired when a chunk of response content is received
    fn process_response_chunk(&mut self, request: &Request, chunk: Vec<u8>);

    /// <https://fetch.spec.whatwg.org/#process-response-end-of-file>
    ///
    /// Fired when the response is fully fetched
    fn process_response_eof(&mut self, request: &Request, response: &Response);

    fn process_csp_violations(&mut self, request: &Request, violations: Vec<csp::Violation>);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FilteredMetadata {
    Basic(Metadata),
    Cors(Metadata),
    Opaque,
    OpaqueRedirect(ServoUrl),
}

// FIXME: https://github.com/servo/servo/issues/34591
#[expect(clippy::large_enum_variant)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FetchMetadata {
    Unfiltered(Metadata),
    Filtered {
        filtered: FilteredMetadata,
        unsafe_: Metadata,
    },
}

impl FetchMetadata {
    pub fn metadata(&self) -> &Metadata {
        match self {
            Self::Unfiltered(metadata) => metadata,
            Self::Filtered { unsafe_, .. } => unsafe_,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#cors-cross-origin>
    pub fn is_cors_cross_origin(&self) -> bool {
        if let Self::Filtered { filtered, .. } = self {
            match filtered {
                FilteredMetadata::Basic(_) | FilteredMetadata::Cors(_) => false,
                FilteredMetadata::Opaque | FilteredMetadata::OpaqueRedirect(_) => true,
            }
        } else {
            false
        }
    }
}

impl FetchTaskTarget for IpcSender<FetchResponseMsg> {
    fn process_request_body(&mut self, request: &Request) {
        let _ = self.send(FetchResponseMsg::ProcessRequestBody(request.id));
    }

    fn process_request_eof(&mut self, request: &Request) {
        let _ = self.send(FetchResponseMsg::ProcessRequestEOF(request.id));
    }

    fn process_response(&mut self, request: &Request, response: &Response) {
        let _ = self.send(FetchResponseMsg::ProcessResponse(
            request.id,
            response.metadata(),
        ));
    }

    fn process_response_chunk(&mut self, request: &Request, chunk: Vec<u8>) {
        let _ = self.send(FetchResponseMsg::ProcessResponseChunk(
            request.id,
            chunk.into(),
        ));
    }

    fn process_response_eof(&mut self, request: &Request, response: &Response) {
        let result = response
            .get_network_error()
            .map_or_else(|| Ok(()), |network_error| Err(network_error.clone()));
        let timing = response.get_resource_timing().lock().clone();

        let _ = self.send(FetchResponseMsg::ProcessResponseEOF(
            request.id, result, timing,
        ));
    }

    fn process_csp_violations(&mut self, request: &Request, violations: Vec<csp::Violation>) {
        let _ = self.send(FetchResponseMsg::ProcessCspViolations(
            request.id, violations,
        ));
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TlsSecurityState {
    /// The connection used to fetch this resource was not secure.
    #[default]
    Insecure,
    /// This resource was transferred over a connection that used weak encryption.
    Weak,
    /// A security error prevented the resource from being loaded.
    Broken,
    /// The connection used to fetch this resource was secure.
    Secure,
}

impl Display for TlsSecurityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            TlsSecurityState::Insecure => "insecure",
            TlsSecurityState::Weak => "weak",
            TlsSecurityState::Broken => "broken",
            TlsSecurityState::Secure => "secure",
        };
        f.write_str(text)
    }
}

#[derive(Clone, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct TlsSecurityInfo {
    // "insecure", "weak", "broken", "secure".
    #[serde(default)]
    pub state: TlsSecurityState,
    // Reasons explaining why the negotiated parameters are considered weak.
    pub weakness_reasons: Vec<String>,
    // Negotiated TLS protocol version (e.g. "TLS 1.3").
    pub protocol_version: Option<String>,
    // Negotiated cipher suite identifier.
    pub cipher_suite: Option<String>,
    // Negotiated key exchange group.
    pub kea_group_name: Option<String>,
    // Signature scheme used for certificate verification.
    pub signature_scheme_name: Option<String>,
    // Negotiated ALPN protocol (e.g. "h2" for HTTP/2, "http/1.1" for HTTP/1.1).
    pub alpn_protocol: Option<String>,
    // Server certificate chain encoded as DER bytes, leaf first.
    pub certificate_chain_der: Vec<Vec<u8>>,
    // Certificate Transparency status, if provided.
    pub certificate_transparency: Option<String>,
    // HTTP Strict Transport Security flag.
    pub hsts: bool,
    // HTTP Public Key Pinning flag (always false, kept for parity).
    pub hpkp: bool,
    // Encrypted Client Hello usage flag.
    pub used_ech: bool,
    // Delegated credentials usage flag.
    pub used_delegated_credentials: bool,
    // OCSP stapling usage flag.
    pub used_ocsp: bool,
    // Private DNS usage flag.
    pub used_private_dns: bool,
}

impl FetchTaskTarget for IpcSender<WebSocketNetworkEvent> {
    fn process_request_body(&mut self, _: &Request) {}
    fn process_request_eof(&mut self, _: &Request) {}
    fn process_response(&mut self, _: &Request, response: &Response) {
        if response.is_network_error() {
            let _ = self.send(WebSocketNetworkEvent::Fail);
        }
    }
    fn process_response_chunk(&mut self, _: &Request, _: Vec<u8>) {}
    fn process_response_eof(&mut self, _: &Request, _: &Response) {}
    fn process_csp_violations(&mut self, _: &Request, violations: Vec<csp::Violation>) {
        let _ = self.send(WebSocketNetworkEvent::ReportCSPViolations(violations));
    }
}

/// A fetch task that discards all data it's sent,
/// useful when speculatively prefetching data that we don't need right
/// now, but might need in the future.
pub struct DiscardFetch;

impl FetchTaskTarget for DiscardFetch {
    fn process_request_body(&mut self, _: &Request) {}
    fn process_request_eof(&mut self, _: &Request) {}
    fn process_response(&mut self, _: &Request, _: &Response) {}
    fn process_response_chunk(&mut self, _: &Request, _: Vec<u8>) {}
    fn process_response_eof(&mut self, _: &Request, _: &Response) {}
    fn process_csp_violations(&mut self, _: &Request, _: Vec<csp::Violation>) {}
}

/// Handle to an async runtime,
/// only used to shut it down for now.
pub trait AsyncRuntime: Send {
    fn shutdown(&mut self);
}

/// Handle to a resource thread
pub type CoreResourceThread = GenericSender<CoreResourceMsg>;

// FIXME: Originally we will construct an Arc<ResourceThread> from ResourceThread
// in script_thread to avoid some performance pitfall. Now we decide to deal with
// the "Arc" hack implicitly in future.
// See discussion: http://logs.glob.uno/?c=mozilla%23servo&s=16+May+2016&e=16+May+2016#c430412
// See also: https://github.com/servo/servo/blob/735480/components/script/script_thread.rs#L313
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceThreads {
    pub core_thread: CoreResourceThread,
}

impl ResourceThreads {
    pub fn new(core_thread: CoreResourceThread) -> ResourceThreads {
        ResourceThreads { core_thread }
    }

    pub fn cache_entries(&self) -> Vec<CacheEntryDescriptor> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let _ = self
            .core_thread
            .send(CoreResourceMsg::GetCacheEntries(sender));
        receiver.recv().unwrap()
    }

    pub fn clear_cache(&self) {
        // NOTE: Messages used in these methods are currently handled
        // synchronously on the backend without consulting other threads, so
        // waiting for the response here cannot deadlock. If the backend
        // handling ever becomes asynchronous or involves sending messages
        // back to the originating thread, this code will need to be revisited
        // to avoid potential deadlocks.
        let (sender, receiver) = generic_channel::channel().unwrap();
        let _ = self
            .core_thread
            .send(CoreResourceMsg::ClearCache(Some(sender)));
        let _ = receiver.recv();
    }

    pub fn cookies(&self) -> Vec<SiteDescriptor> {
        let (sender, receiver) = generic_channel::channel().unwrap();
        let _ = self.core_thread.send(CoreResourceMsg::ListCookies(sender));
        receiver.recv().unwrap()
    }

    pub fn clear_cookies_for_sites(&self, sites: &[&str]) {
        let sites = sites.iter().map(|site| site.to_string()).collect();
        let (sender, receiver) = generic_channel::channel().unwrap();
        let _ = self
            .core_thread
            .send(CoreResourceMsg::DeleteCookiesForSites(sites, sender));
        let _ = receiver.recv();
    }

    pub fn clear_cookies(&self) {
        let (sender, receiver) = ipc::channel().unwrap();
        let _ = self
            .core_thread
            .send(CoreResourceMsg::DeleteCookies(None, Some(sender)));
        let _ = receiver.recv();
    }
}

impl GenericSend<CoreResourceMsg> for ResourceThreads {
    fn send(&self, msg: CoreResourceMsg) -> SendResult {
        self.core_thread.send(msg)
    }

    fn sender(&self) -> GenericSender<CoreResourceMsg> {
        self.core_thread.clone()
    }
}

// Ignore the sub-fields
malloc_size_of_is_0!(ResourceThreads);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum IncludeSubdomains {
    Included,
    NotIncluded,
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum MessageData {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebSocketDomAction {
    SendMessage(MessageData),
    Close(Option<u16>, Option<String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WebSocketNetworkEvent {
    ReportCSPViolations(Vec<csp::Violation>),
    ConnectionEstablished { protocol_in_use: Option<String> },
    MessageReceived(MessageData),
    Close(Option<u16>, String),
    Fail,
}

#[derive(Debug, Deserialize, Serialize)]
/// IPC channels to communicate with the script thread about network or DOM events.
pub enum FetchChannels {
    ResponseMsg(IpcSender<FetchResponseMsg>),
    WebSocket {
        event_sender: IpcSender<WebSocketNetworkEvent>,
        action_receiver: IpcReceiver<WebSocketDomAction>,
    },
    /// If the fetch is just being done to populate the cache,
    /// not because the data is needed now.
    Prefetch,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CoreResourceMsg {
    Fetch(RequestBuilder, FetchChannels),
    Cancel(Vec<RequestId>),
    /// Initiate a fetch in response to processing a redirection
    FetchRedirect(RequestBuilder, ResponseInit, IpcSender<FetchResponseMsg>),
    /// Store a cookie for a given originating URL
    SetCookieForUrl(ServoUrl, Serde<Cookie<'static>>, CookieSource),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(ServoUrl, Vec<Serde<Cookie<'static>>>, CookieSource),
    SetCookieForUrlAsync(
        CookieStoreId,
        ServoUrl,
        Serde<Cookie<'static>>,
        CookieSource,
    ),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(ServoUrl, IpcSender<Option<String>>, CookieSource),
    /// Get a cookie by name for a given originating URL
    GetCookiesDataForUrl(
        ServoUrl,
        IpcSender<Vec<Serde<Cookie<'static>>>>,
        CookieSource,
    ),
    GetCookieDataForUrlAsync(CookieStoreId, ServoUrl, Option<String>),
    GetAllCookieDataForUrlAsync(CookieStoreId, ServoUrl, Option<String>),
    DeleteCookiesForSites(Vec<String>, GenericSender<()>),
    /// This currently is used by unit tests and WebDriver only.
    /// When url is `None`, this clears cookies across all origins.
    DeleteCookies(Option<ServoUrl>, Option<IpcSender<()>>),
    DeleteCookie(ServoUrl, String),
    DeleteCookieAsync(CookieStoreId, ServoUrl, String),
    NewCookieListener(CookieStoreId, IpcSender<CookieAsyncResponse>, ServoUrl),
    RemoveCookieListener(CookieStoreId),
    ListCookies(GenericSender<Vec<SiteDescriptor>>),
    /// Get a history state by a given history state id
    GetHistoryState(HistoryStateId, IpcSender<Option<Vec<u8>>>),
    /// Set a history state for a given history state id
    SetHistoryState(HistoryStateId, Vec<u8>),
    /// Removes history states for the given ids
    RemoveHistoryStates(Vec<HistoryStateId>),
    /// Gets a list of origin descriptors derived from entries in the cache
    GetCacheEntries(GenericSender<Vec<CacheEntryDescriptor>>),
    /// Clear the network cache.
    ClearCache(Option<GenericSender<()>>),
    /// Send the service worker network mediator for an origin to CoreResourceThread
    NetworkMediator(IpcSender<CustomResponseMediator>, ImmutableOrigin),
    /// Message forwarded to file manager's handler
    ToFileManager(FileManagerThreadMsg),
    StorePreloadedResponse(PreloadId, Response),
    TotalSizeOfInFlightKeepAliveRecords(PipelineId, GenericSender<u64>),
    /// Break the load handler loop, send a reply when done cleaning up local resources
    /// and exit
    Exit(GenericOneshotSender<()>),
    CollectMemoryReport(ReportsChan),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SiteDescriptor {
    pub name: String,
}

impl SiteDescriptor {
    pub fn new(name: String) -> Self {
        SiteDescriptor { name }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheEntryDescriptor {
    pub key: String,
}

impl CacheEntryDescriptor {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

// FIXME: https://github.com/servo/servo/issues/34591
#[expect(clippy::large_enum_variant)]
enum ToFetchThreadMessage {
    Cancel(Vec<RequestId>, CoreResourceThread),
    StartFetch(
        /* request_builder */ RequestBuilder,
        /* response_init */ Option<ResponseInit>,
        /* callback  */ BoxedFetchCallback,
        /* core resource thread channel */ CoreResourceThread,
    ),
    FetchResponse(FetchResponseMsg),
    /// Stop the background thread.
    Exit,
}

pub type BoxedFetchCallback = Box<dyn FnMut(FetchResponseMsg) + Send + 'static>;

/// A thread to handle fetches in a Servo process. This thread is responsible for
/// listening for new fetch requests as well as updates on those operations and forwarding
/// them to crossbeam channels.
struct FetchThread {
    /// A list of active fetches. A fetch is no longer active once the
    /// [`FetchResponseMsg::ProcessResponseEOF`] is received.
    active_fetches: FxHashMap<RequestId, BoxedFetchCallback>,
    /// A crossbeam receiver attached to the router proxy which converts incoming fetch
    /// updates from IPC messages to crossbeam messages as well as another sender which
    /// handles requests from clients wanting to do fetches.
    receiver: Receiver<ToFetchThreadMessage>,
    /// An [`IpcSender`] that's sent with every fetch request and leads back to our
    /// router proxy.
    to_fetch_sender: IpcSender<FetchResponseMsg>,
}

impl FetchThread {
    fn spawn() -> (Sender<ToFetchThreadMessage>, JoinHandle<()>) {
        let (sender, receiver) = unbounded();
        let (to_fetch_sender, from_fetch_sender) = ipc::channel().unwrap();

        let sender_clone = sender.clone();
        ROUTER.add_typed_route(
            from_fetch_sender,
            Box::new(move |message| {
                let message: FetchResponseMsg = message.unwrap();
                let _ = sender_clone.send(ToFetchThreadMessage::FetchResponse(message));
            }),
        );
        let join_handle = thread::Builder::new()
            .name("FetchThread".to_owned())
            .spawn(move || {
                let mut fetch_thread = FetchThread {
                    active_fetches: FxHashMap::default(),
                    receiver,
                    to_fetch_sender,
                };
                fetch_thread.run();
            })
            .expect("Thread spawning failed");
        (sender, join_handle)
    }

    fn run(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                ToFetchThreadMessage::StartFetch(
                    request_builder,
                    response_init,
                    callback,
                    core_resource_thread,
                ) => {
                    let request_builder_id = request_builder.id;

                    // Only redirects have a `response_init` field.
                    let message = match response_init {
                        Some(response_init) => CoreResourceMsg::FetchRedirect(
                            request_builder,
                            response_init,
                            self.to_fetch_sender.clone(),
                        ),
                        None => CoreResourceMsg::Fetch(
                            request_builder,
                            FetchChannels::ResponseMsg(self.to_fetch_sender.clone()),
                        ),
                    };

                    core_resource_thread.send(message).unwrap();

                    let preexisting_fetch =
                        self.active_fetches.insert(request_builder_id, callback);
                    // When we terminate a fetch group, all deferred fetches are processed.
                    // In case we were already processing a deferred fetch, we should not
                    // process the second call. This should be handled by [`DeferredFetchRecord::process`]
                    assert!(preexisting_fetch.is_none());
                },
                ToFetchThreadMessage::FetchResponse(fetch_response_msg) => {
                    let request_id = fetch_response_msg.request_id();
                    let fetch_finished =
                        matches!(fetch_response_msg, FetchResponseMsg::ProcessResponseEOF(..));

                    self.active_fetches
                        .get_mut(&request_id)
                        .expect("Got fetch response for unknown fetch")(
                        fetch_response_msg
                    );

                    if fetch_finished {
                        self.active_fetches.remove(&request_id);
                    }
                },
                ToFetchThreadMessage::Cancel(request_ids, core_resource_thread) => {
                    // Errors are ignored here, because Servo sends many cancellation requests when shutting down.
                    // At this point the networking task might be shut down completely, so just ignore errors
                    // during this time.
                    let _ = core_resource_thread.send(CoreResourceMsg::Cancel(request_ids));
                },
                ToFetchThreadMessage::Exit => break,
            }
        }
    }
}

static FETCH_THREAD: OnceLock<Sender<ToFetchThreadMessage>> = OnceLock::new();

/// Start the fetch thread,
/// and returns the join handle to the background thread.
pub fn start_fetch_thread() -> JoinHandle<()> {
    let (sender, join_handle) = FetchThread::spawn();
    FETCH_THREAD
        .set(sender)
        .expect("Fetch thread should be set only once on start-up");
    join_handle
}

/// Send the exit message to the background thread,
/// after which the caller can,
/// and should,
/// join on the thread.
pub fn exit_fetch_thread() {
    let _ = FETCH_THREAD
        .get()
        .expect("Fetch thread should always be initialized on start-up")
        .send(ToFetchThreadMessage::Exit);
}

/// Instruct the resource thread to make a new fetch request.
pub fn fetch_async(
    core_resource_thread: &CoreResourceThread,
    request: RequestBuilder,
    response_init: Option<ResponseInit>,
    callback: BoxedFetchCallback,
) {
    let _ = FETCH_THREAD
        .get()
        .expect("Fetch thread should always be initialized on start-up")
        .send(ToFetchThreadMessage::StartFetch(
            request,
            response_init,
            callback,
            core_resource_thread.clone(),
        ));
}

/// Instruct the resource thread to cancel an existing request. Does nothing if the
/// request has already completed or has not been fetched yet.
pub fn cancel_async_fetch(request_ids: Vec<RequestId>, core_resource_thread: &CoreResourceThread) {
    let _ = FETCH_THREAD
        .get()
        .expect("Fetch thread should always be initialized on start-up")
        .send(ToFetchThreadMessage::Cancel(
            request_ids,
            core_resource_thread.clone(),
        ));
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ResourceCorsData {
    /// CORS Preflight flag
    pub preflight: bool,
    /// Origin of CORS Request
    pub origin: ServoUrl,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ResourceFetchTiming {
    pub domain_lookup_start: Option<CrossProcessInstant>,
    pub timing_check_passed: bool,
    pub timing_type: ResourceTimingType,
    /// Number of redirects until final resource (currently limited to 20)
    pub redirect_count: u16,
    pub request_start: Option<CrossProcessInstant>,
    pub secure_connection_start: Option<CrossProcessInstant>,
    pub response_start: Option<CrossProcessInstant>,
    pub fetch_start: Option<CrossProcessInstant>,
    pub response_end: Option<CrossProcessInstant>,
    pub redirect_start: Option<CrossProcessInstant>,
    pub redirect_end: Option<CrossProcessInstant>,
    pub connect_start: Option<CrossProcessInstant>,
    pub connect_end: Option<CrossProcessInstant>,
    pub start_time: Option<CrossProcessInstant>,
    pub preloaded: bool,
}

pub enum RedirectStartValue {
    Zero,
    FetchStart,
}

pub enum RedirectEndValue {
    Zero,
    ResponseEnd,
}

// TODO: refactor existing code to use this enum for setting time attributes
// suggest using this with all time attributes in the future
pub enum ResourceTimeValue {
    Zero,
    Now,
    FetchStart,
    RedirectStart,
}

pub enum ResourceAttribute {
    RedirectCount(u16),
    DomainLookupStart,
    RequestStart,
    ResponseStart,
    RedirectStart(RedirectStartValue),
    RedirectEnd(RedirectEndValue),
    FetchStart,
    ConnectStart(CrossProcessInstant),
    ConnectEnd(CrossProcessInstant),
    SecureConnectionStart,
    ResponseEnd,
    StartTime(ResourceTimeValue),
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ResourceTimingType {
    Resource,
    Navigation,
    Error,
    None,
}

impl ResourceFetchTiming {
    pub fn new(timing_type: ResourceTimingType) -> ResourceFetchTiming {
        ResourceFetchTiming {
            timing_type,
            timing_check_passed: true,
            domain_lookup_start: None,
            redirect_count: 0,
            secure_connection_start: None,
            request_start: None,
            response_start: None,
            fetch_start: None,
            redirect_start: None,
            redirect_end: None,
            connect_start: None,
            connect_end: None,
            response_end: None,
            start_time: None,
            preloaded: false,
        }
    }

    // TODO currently this is being set with precise time ns when it should be time since
    // time origin (as described in Performance::now)
    pub fn set_attribute(&mut self, attribute: ResourceAttribute) {
        let should_attribute_always_be_updated = matches!(
            attribute,
            ResourceAttribute::FetchStart |
                ResourceAttribute::ResponseEnd |
                ResourceAttribute::StartTime(_)
        );
        if !self.timing_check_passed && !should_attribute_always_be_updated {
            return;
        }
        let now = Some(CrossProcessInstant::now());
        match attribute {
            ResourceAttribute::DomainLookupStart => self.domain_lookup_start = now,
            ResourceAttribute::RedirectCount(count) => self.redirect_count = count,
            ResourceAttribute::RequestStart => self.request_start = now,
            ResourceAttribute::ResponseStart => self.response_start = now,
            ResourceAttribute::RedirectStart(val) => match val {
                RedirectStartValue::Zero => self.redirect_start = None,
                RedirectStartValue::FetchStart => {
                    if self.redirect_start.is_none() {
                        self.redirect_start = self.fetch_start
                    }
                },
            },
            ResourceAttribute::RedirectEnd(val) => match val {
                RedirectEndValue::Zero => self.redirect_end = None,
                RedirectEndValue::ResponseEnd => self.redirect_end = self.response_end,
            },
            ResourceAttribute::FetchStart => self.fetch_start = now,
            ResourceAttribute::ConnectStart(instant) => self.connect_start = Some(instant),
            ResourceAttribute::ConnectEnd(instant) => self.connect_end = Some(instant),
            ResourceAttribute::SecureConnectionStart => self.secure_connection_start = now,
            ResourceAttribute::ResponseEnd => self.response_end = now,
            ResourceAttribute::StartTime(val) => match val {
                ResourceTimeValue::RedirectStart
                    if self.redirect_start.is_none() || !self.timing_check_passed => {},
                _ => self.start_time = self.get_time_value(val),
            },
        }
    }

    fn get_time_value(&self, time: ResourceTimeValue) -> Option<CrossProcessInstant> {
        match time {
            ResourceTimeValue::Zero => None,
            ResourceTimeValue::Now => Some(CrossProcessInstant::now()),
            ResourceTimeValue::FetchStart => self.fetch_start,
            ResourceTimeValue::RedirectStart => self.redirect_start,
        }
    }

    pub fn mark_timing_check_failed(&mut self) {
        self.timing_check_passed = false;
        self.domain_lookup_start = None;
        self.redirect_count = 0;
        self.request_start = None;
        self.response_start = None;
        self.redirect_start = None;
        self.connect_start = None;
        self.connect_end = None;
    }
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: ServoUrl,

    /// Location URL from the response headers.
    pub location_url: Option<Result<ServoUrl, String>>,

    #[ignore_malloc_size_of = "Defined in hyper"]
    /// MIME type / subtype.
    pub content_type: Option<Serde<ContentType>>,

    /// Character set.
    pub charset: Option<String>,

    #[ignore_malloc_size_of = "Defined in hyper"]
    /// Headers
    pub headers: Option<Serde<HeaderMap>>,

    /// HTTP Status
    pub status: HttpStatus,

    /// Is successful HTTPS connection
    pub https_state: HttpsState,

    /// Referrer Url
    pub referrer: Option<ServoUrl>,

    /// Referrer Policy of the Request used to obtain Response
    pub referrer_policy: ReferrerPolicy,
    /// Performance information for navigation events
    pub timing: Option<ResourceFetchTiming>,
    /// True if the request comes from a redirection
    pub redirected: bool,
    /// Detailed TLS metadata associated with the response, if any.
    pub tls_security_info: Option<TlsSecurityInfo>,
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: ServoUrl) -> Self {
        Metadata {
            final_url: url,
            location_url: None,
            content_type: None,
            charset: None,
            headers: None,
            status: HttpStatus::default(),
            https_state: HttpsState::None,
            referrer: None,
            referrer_policy: ReferrerPolicy::EmptyString,
            timing: None,
            redirected: false,
            tls_security_info: None,
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        if self.headers.is_none() {
            self.headers = Some(Serde(HeaderMap::new()));
        }

        if let Some(mime) = content_type {
            self.headers
                .as_mut()
                .unwrap()
                .typed_insert(ContentType::from(mime.clone()));
            if let Some(charset) = mime.get_param(mime::CHARSET) {
                self.charset = Some(charset.to_string());
            }
            self.content_type = Some(Serde(ContentType::from(mime.clone())));
        }
    }

    /// Set the referrer policy associated with the loaded resource.
    pub fn set_referrer_policy(&mut self, referrer_policy: ReferrerPolicy) {
        if referrer_policy == ReferrerPolicy::EmptyString {
            return;
        }

        if self.headers.is_none() {
            self.headers = Some(Serde(HeaderMap::new()));
        }

        self.referrer_policy = referrer_policy;

        self.headers
            .as_mut()
            .unwrap()
            .typed_insert::<ReferrerPolicyHeader>(referrer_policy.into());
    }

    /// <https://html.spec.whatwg.org/multipage/#content-type>
    pub fn resource_content_type_metadata(&self, load_context: LoadContext, data: &[u8]) -> Mime {
        // The Content-Type metadata of a resource must be obtained and interpreted in a manner consistent with the requirements of MIME Sniffing. [MIMESNIFF]
        let no_sniff = self
            .headers
            .as_deref()
            .is_some_and(determine_nosniff)
            .into();
        let mime = self
            .content_type
            .clone()
            .map(|content_type| content_type.into_inner().into());
        MimeClassifier::default().classify(
            load_context,
            no_sniff,
            ApacheBugFlag::from_content_type(mime.as_ref()),
            &mime,
            data,
        )
    }
}

/// The creator of a given cookie
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum CookieSource {
    /// An HTTP API
    HTTP,
    /// A non-HTTP API
    NonHTTP,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CookieChange {
    changed: Vec<Serde<Cookie<'static>>>,
    deleted: Vec<Serde<Cookie<'static>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CookieData {
    Change(CookieChange),
    Get(Option<Serde<Cookie<'static>>>),
    GetAll(Vec<Serde<Cookie<'static>>>),
    Set(Result<(), ()>),
    Delete(Result<(), ()>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CookieAsyncResponse {
    pub data: CookieData,
}

/// Network errors that have to be exported out of the loaders
#[derive(Clone, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum NetworkError {
    LoadCancelled,
    /// SSL validation error, to be converted to Resource::BadCertHTML in the HTML parser.
    SslValidation(String, Vec<u8>),
    /// Crash error, to be converted to Resource::Crash in the HTML parser.
    Crash(String),
    UnsupportedScheme,
    CorsGeneral,
    CrossOriginResponse,
    CorsCredentials,
    CorsAllowMethods,
    CorsAllowHeaders,
    CorsMethod,
    CorsAuthorization,
    CorsHeaders,
    ConnectionFailure,
    RedirectError,
    TooManyRedirects,
    TooManyInFlightKeepAliveRequests,
    InvalidMethod,
    ResourceLoadError(String),
    ContentSecurityPolicy,
    Nosniff,
    MimeType(String),
    SubresourceIntegrity,
    MixedContent,
    CacheError,
    InvalidPort,
    WebsocketConnectionFailure(String),
    LocalDirectoryError,
    PartialResponseToNonRangeRequestError,
    ProtocolHandlerSubstitutionError,
    BlobURLStoreError(String),
    HttpError(String),
    DecompressionError,
}

impl fmt::Debug for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::UnsupportedScheme => write!(f, "Unsupported scheme"),
            NetworkError::CorsGeneral => write!(f, "CORS check failed"),
            NetworkError::CrossOriginResponse => write!(f, "Cross-origin response"),
            NetworkError::CorsCredentials => write!(f, "Cross-origin credentials check failed"),
            NetworkError::CorsAllowMethods => write!(f, "CORS ACAM check failed"),
            NetworkError::CorsAllowHeaders => write!(f, "CORS ACAH check failed"),
            NetworkError::CorsMethod => write!(f, "CORS method check failed"),
            NetworkError::CorsAuthorization => write!(f, "CORS authorization check failed"),
            NetworkError::CorsHeaders => write!(f, "CORS headers check failed"),
            NetworkError::ConnectionFailure => write!(f, "Request failed"),
            NetworkError::RedirectError => write!(f, "Redirect failed"),
            NetworkError::TooManyRedirects => write!(f, "Too many redirects"),
            NetworkError::TooManyInFlightKeepAliveRequests => {
                write!(f, "Too many in flight keep-alive requests")
            },
            NetworkError::InvalidMethod => write!(f, "Unexpected method"),
            NetworkError::ResourceLoadError(s) => write!(f, "{}", s),
            NetworkError::ContentSecurityPolicy => write!(f, "Blocked by Content-Security-Policy"),
            NetworkError::Nosniff => write!(f, "Blocked by nosniff"),
            NetworkError::MimeType(s) => write!(f, "{}", s),
            NetworkError::SubresourceIntegrity => {
                write!(f, "Subresource integrity validation failed")
            },
            NetworkError::MixedContent => write!(f, "Blocked as mixed content"),
            NetworkError::CacheError => write!(f, "Couldn't find response in cache"),
            NetworkError::InvalidPort => write!(f, "Request attempted on bad port"),
            NetworkError::LocalDirectoryError => write!(f, "Local directory access failed"),
            NetworkError::LoadCancelled => write!(f, "Load cancelled"),
            NetworkError::SslValidation(s, _) => write!(f, "SSL validation error: {}", s),
            NetworkError::Crash(s) => write!(f, "Crash: {}", s),
            NetworkError::PartialResponseToNonRangeRequestError => write!(
                f,
                "Refusing to provide partial response from earlier ranged request to API that did not make a range request"
            ),
            NetworkError::ProtocolHandlerSubstitutionError => {
                write!(f, "Failed to parse substituted protocol handler url")
            },
            NetworkError::BlobURLStoreError(s) => write!(f, "Blob URL store error: {}", s),
            NetworkError::WebsocketConnectionFailure(s) => {
                write!(f, "Websocket connection failure: {}", s)
            },
            NetworkError::HttpError(s) => write!(f, "HTTP failure: {}", s),
            NetworkError::DecompressionError => write!(f, "Decompression error"),
        }
    }
}

impl NetworkError {
    pub fn is_permanent_failure(&self) -> bool {
        matches!(
            self,
            NetworkError::ContentSecurityPolicy |
                NetworkError::MixedContent |
                NetworkError::SubresourceIntegrity |
                NetworkError::Nosniff |
                NetworkError::InvalidPort |
                NetworkError::CorsGeneral |
                NetworkError::CrossOriginResponse |
                NetworkError::CorsCredentials |
                NetworkError::CorsAllowMethods |
                NetworkError::CorsAllowHeaders |
                NetworkError::CorsMethod |
                NetworkError::CorsAuthorization |
                NetworkError::CorsHeaders |
                NetworkError::UnsupportedScheme
        )
    }

    pub fn from_hyper_error(error: &HyperError, certificate: Option<CertificateDer>) -> Self {
        let error_string = error.to_string();
        match certificate {
            Some(certificate) => NetworkError::SslValidation(error_string, certificate.to_vec()),
            _ => NetworkError::HttpError(error_string),
        }
    }
}

/// Normalize `slice`, as defined by
/// [the Fetch Spec](https://fetch.spec.whatwg.org/#concept-header-value-normalize).
pub fn trim_http_whitespace(mut slice: &[u8]) -> &[u8] {
    const HTTP_WS_BYTES: &[u8] = b"\x09\x0A\x0D\x20";

    loop {
        match slice.split_first() {
            Some((first, remainder)) if HTTP_WS_BYTES.contains(first) => slice = remainder,
            _ => break,
        }
    }

    loop {
        match slice.split_last() {
            Some((last, remainder)) if HTTP_WS_BYTES.contains(last) => slice = remainder,
            _ => break,
        }
    }

    slice
}

pub fn http_percent_encode(bytes: &[u8]) -> String {
    // This encode set is used for HTTP header values and is defined at
    // https://tools.ietf.org/html/rfc5987#section-3.2
    const HTTP_VALUE: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'%')
        .add(b'\'')
        .add(b'(')
        .add(b')')
        .add(b'*')
        .add(b',')
        .add(b'/')
        .add(b':')
        .add(b';')
        .add(b'<')
        .add(b'-')
        .add(b'>')
        .add(b'?')
        .add(b'[')
        .add(b'\\')
        .add(b']')
        .add(b'{')
        .add(b'}');

    percent_encoding::percent_encode(bytes, HTTP_VALUE).to_string()
}

/// Returns the cached current system locale, or en-US by default.
pub fn get_current_locale() -> &'static (String, HeaderValue) {
    static CURRENT_LOCALE: OnceLock<(String, HeaderValue)> = OnceLock::new();

    CURRENT_LOCALE.get_or_init(|| {
        let locale_override = servo_config::pref!(intl_locale_override);
        let locale = if locale_override.is_empty() {
            sys_locale::get_locale().unwrap_or_else(|| "en-US".into())
        } else {
            locale_override
        };
        let header_value = HeaderValue::from_str(&locale)
            .ok()
            .unwrap_or_else(|| HeaderValue::from_static("en-US"));
        (locale, header_value)
    })
}

/// Step 12 of <https://fetch.spec.whatwg.org/#concept-fetch>
pub fn set_default_accept_language(headers: &mut HeaderMap) {
    // If request’s header list does not contain `Accept-Language`,
    // then user agents should append (`Accept-Language, an appropriate header value) to request’s header list.
    if headers.contains_key(header::ACCEPT_LANGUAGE) {
        return;
    }

    // To reduce fingerprinting we set only a single language.
    headers.insert(header::ACCEPT_LANGUAGE, get_current_locale().1.clone());
}

pub static PRIVILEGED_SECRET: LazyLock<u32> = LazyLock::new(|| rng().next_u32());
