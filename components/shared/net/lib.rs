/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

use base::id::HistoryStateId;
use cookie::Cookie;
use headers::{ContentType, HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use http::{Error as HttpError, HeaderMap, StatusCode};
use hyper::Error as HyperError;
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use ipc_channel::Error as IpcError;
use lazy_static::lazy_static;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use mime::Mime;
use num_traits::Zero;
use rustls::Certificate;
use serde::{Deserialize, Serialize};
use servo_rand::RngCore;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::filemanager_thread::FileManagerThreadMsg;
use crate::request::{Request, RequestBuilder};
use crate::response::{HttpsState, Response, ResponseInit};
use crate::storage_thread::StorageThreadMsg;

pub mod blob_url_store;
pub mod filemanager_thread;
pub mod image_cache;
pub mod pub_domains;
pub mod quality;
pub mod request;
pub mod response;
pub mod storage_thread;

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
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum ReferrerPolicy {
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
    StrictOriginWhenCrossOrigin,
}

impl Display for ReferrerPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
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

impl From<ReferrerPolicyHeader> for ReferrerPolicy {
    fn from(policy: ReferrerPolicyHeader) -> Self {
        match policy {
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
        }
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
            ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                ReferrerPolicyHeader::STRICT_ORIGIN_WHEN_CROSS_ORIGIN
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum FetchResponseMsg {
    // todo: should have fields for transmitted/total bytes
    ProcessRequestBody,
    ProcessRequestEOF,
    // todo: send more info about the response (or perhaps the entire Response)
    ProcessResponse(Result<FetchMetadata, NetworkError>),
    ProcessResponseChunk(Vec<u8>),
    ProcessResponseEOF(Result<ResourceFetchTiming, NetworkError>),
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
    fn process_response(&mut self, response: &Response);

    /// Fired when a chunk of response content is received
    fn process_response_chunk(&mut self, chunk: Vec<u8>);

    /// <https://fetch.spec.whatwg.org/#process-response-end-of-file>
    ///
    /// Fired when the response is fully fetched
    fn process_response_eof(&mut self, response: &Response);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FilteredMetadata {
    Basic(Metadata),
    Cors(Metadata),
    Opaque,
    OpaqueRedirect(ServoUrl),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FetchMetadata {
    Unfiltered(Metadata),
    Filtered {
        filtered: FilteredMetadata,
        unsafe_: Metadata,
    },
}

pub trait FetchResponseListener {
    fn process_request_body(&mut self);
    fn process_request_eof(&mut self);
    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>);
    fn process_response_chunk(&mut self, chunk: Vec<u8>);
    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>);
    fn resource_timing(&self) -> &ResourceFetchTiming;
    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming;
    fn submit_resource_timing(&mut self);
}

impl FetchTaskTarget for IpcSender<FetchResponseMsg> {
    fn process_request_body(&mut self, _: &Request) {
        let _ = self.send(FetchResponseMsg::ProcessRequestBody);
    }

    fn process_request_eof(&mut self, _: &Request) {
        let _ = self.send(FetchResponseMsg::ProcessRequestEOF);
    }

    fn process_response(&mut self, response: &Response) {
        let _ = self.send(FetchResponseMsg::ProcessResponse(response.metadata()));
    }

    fn process_response_chunk(&mut self, chunk: Vec<u8>) {
        let _ = self.send(FetchResponseMsg::ProcessResponseChunk(chunk));
    }

    fn process_response_eof(&mut self, response: &Response) {
        if let Some(e) = response.get_network_error() {
            let _ = self.send(FetchResponseMsg::ProcessResponseEOF(Err(e.clone())));
        } else {
            let _ = self.send(FetchResponseMsg::ProcessResponseEOF(Ok(response
                .get_resource_timing()
                .lock()
                .unwrap()
                .clone())));
        }
    }
}

/// A fetch task that discards all data it's sent,
/// useful when speculatively prefetching data that we don't need right
/// now, but might need in the future.
pub struct DiscardFetch;

impl FetchTaskTarget for DiscardFetch {
    fn process_request_body(&mut self, _: &Request) {}

    fn process_request_eof(&mut self, _: &Request) {}

    fn process_response(&mut self, _: &Response) {}

    fn process_response_chunk(&mut self, _: Vec<u8>) {}

    fn process_response_eof(&mut self, _: &Response) {}
}

pub trait Action<Listener> {
    fn process(self, listener: &mut Listener);
}

impl<T: FetchResponseListener> Action<T> for FetchResponseMsg {
    /// Execute the default action on a provided listener.
    fn process(self, listener: &mut T) {
        match self {
            FetchResponseMsg::ProcessRequestBody => listener.process_request_body(),
            FetchResponseMsg::ProcessRequestEOF => listener.process_request_eof(),
            FetchResponseMsg::ProcessResponse(meta) => listener.process_response(meta),
            FetchResponseMsg::ProcessResponseChunk(data) => listener.process_response_chunk(data),
            FetchResponseMsg::ProcessResponseEOF(data) => {
                match data {
                    Ok(ref response_resource_timing) => {
                        // update listener with values from response
                        *listener.resource_timing_mut() = response_resource_timing.clone();
                        listener.process_response_eof(Ok(response_resource_timing.clone()));
                        // TODO timing check https://w3c.github.io/resource-timing/#dfn-timing-allow-check

                        listener.submit_resource_timing();
                    },
                    // TODO Resources for which the fetch was initiated, but was later aborted
                    // (e.g. due to a network error) MAY be included as PerformanceResourceTiming
                    // objects in the Performance Timeline and MUST contain initialized attribute
                    // values for processed substeps of the processing model.
                    Err(e) => listener.process_response_eof(Err(e)),
                }
            },
        }
    }
}

/// Handle to a resource thread
pub type CoreResourceThread = IpcSender<CoreResourceMsg>;

pub type IpcSendResult = Result<(), IpcError>;

/// Abstraction of the ability to send a particular type of message,
/// used by net_traits::ResourceThreads to ease the use its IpcSender sub-fields
/// XXX: If this trait will be used more in future, some auto derive might be appealing
pub trait IpcSend<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// send message T
    fn send(&self, _: T) -> IpcSendResult;
    /// get underlying sender
    fn sender(&self) -> IpcSender<T>;
}

// FIXME: Originally we will construct an Arc<ResourceThread> from ResourceThread
// in script_thread to avoid some performance pitfall. Now we decide to deal with
// the "Arc" hack implicitly in future.
// See discussion: http://logs.glob.uno/?c=mozilla%23servo&s=16+May+2016&e=16+May+2016#c430412
// See also: https://github.com/servo/servo/blob/735480/components/script/script_thread.rs#L313
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceThreads {
    pub core_thread: CoreResourceThread,
    storage_thread: IpcSender<StorageThreadMsg>,
}

impl ResourceThreads {
    pub fn new(c: CoreResourceThread, s: IpcSender<StorageThreadMsg>) -> ResourceThreads {
        ResourceThreads {
            core_thread: c,
            storage_thread: s,
        }
    }

    pub fn clear_cache(&self) {
        let _ = self.core_thread.send(CoreResourceMsg::ClearCache);
    }
}

impl IpcSend<CoreResourceMsg> for ResourceThreads {
    fn send(&self, msg: CoreResourceMsg) -> IpcSendResult {
        self.core_thread.send(msg)
    }

    fn sender(&self) -> IpcSender<CoreResourceMsg> {
        self.core_thread.clone()
    }
}

impl IpcSend<StorageThreadMsg> for ResourceThreads {
    fn send(&self, msg: StorageThreadMsg) -> IpcSendResult {
        self.storage_thread.send(msg)
    }

    fn sender(&self) -> IpcSender<StorageThreadMsg> {
        self.storage_thread.clone()
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
    ConnectionEstablished { protocol_in_use: Option<String> },
    MessageReceived(MessageData),
    Close(Option<u16>, String),
    Fail,
}

#[derive(Debug, Deserialize, Serialize)]
/// IPC channels to communicate with the script thread about network or DOM events.
pub enum FetchChannels {
    ResponseMsg(
        IpcSender<FetchResponseMsg>,
        /* cancel_chan */ Option<IpcReceiver<()>>,
    ),
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
    /// Initiate a fetch in response to processing a redirection
    FetchRedirect(
        RequestBuilder,
        ResponseInit,
        IpcSender<FetchResponseMsg>,
        /* cancel_chan */ Option<IpcReceiver<()>>,
    ),
    /// Store a cookie for a given originating URL
    SetCookieForUrl(ServoUrl, Serde<Cookie<'static>>, CookieSource),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(ServoUrl, Vec<Serde<Cookie<'static>>>, CookieSource),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(ServoUrl, IpcSender<Option<String>>, CookieSource),
    /// Get a cookie by name for a given originating URL
    GetCookiesDataForUrl(
        ServoUrl,
        IpcSender<Vec<Serde<Cookie<'static>>>>,
        CookieSource,
    ),
    DeleteCookies(ServoUrl),
    /// Get a history state by a given history state id
    GetHistoryState(HistoryStateId, IpcSender<Option<Vec<u8>>>),
    /// Set a history state for a given history state id
    SetHistoryState(HistoryStateId, Vec<u8>),
    /// Removes history states for the given ids
    RemoveHistoryStates(Vec<HistoryStateId>),
    /// Synchronization message solely for knowing the state of the ResourceChannelManager loop
    Synchronize(IpcSender<()>),
    /// Clear the network cache.
    ClearCache,
    /// Send the service worker network mediator for an origin to CoreResourceThread
    NetworkMediator(IpcSender<CustomResponseMediator>, ImmutableOrigin),
    /// Message forwarded to file manager's handler
    ToFileManager(FileManagerThreadMsg),
    /// Break the load handler loop, send a reply when done cleaning up local resources
    /// and exit
    Exit(IpcSender<()>),
}

/// Instruct the resource thread to make a new request.
pub fn fetch_async<F>(request: RequestBuilder, core_resource_thread: &CoreResourceThread, f: F)
where
    F: Fn(FetchResponseMsg) + Send + 'static,
{
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    ROUTER.add_route(
        action_receiver.to_opaque(),
        Box::new(move |message| f(message.to().unwrap())),
    );
    core_resource_thread
        .send(CoreResourceMsg::Fetch(
            request,
            FetchChannels::ResponseMsg(action_sender, None),
        ))
        .unwrap();
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
    pub domain_lookup_start: u64,
    pub timing_check_passed: bool,
    pub timing_type: ResourceTimingType,
    /// Number of redirects until final resource (currently limited to 20)
    pub redirect_count: u16,
    pub request_start: u64,
    pub secure_connection_start: u64,
    pub response_start: u64,
    pub fetch_start: u64,
    pub response_end: u64,
    pub redirect_start: u64,
    pub redirect_end: u64,
    pub connect_start: u64,
    pub connect_end: u64,
    pub start_time: u64,
}

pub enum RedirectStartValue {
    #[allow(dead_code)]
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
    ConnectStart(u64),
    ConnectEnd(u64),
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
            domain_lookup_start: 0,
            redirect_count: 0,
            secure_connection_start: 0,
            request_start: 0,
            response_start: 0,
            fetch_start: 0,
            redirect_start: 0,
            redirect_end: 0,
            connect_start: 0,
            connect_end: 0,
            response_end: 0,
            start_time: 0,
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
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        match attribute {
            ResourceAttribute::DomainLookupStart => self.domain_lookup_start = now,
            ResourceAttribute::RedirectCount(count) => self.redirect_count = count,
            ResourceAttribute::RequestStart => self.request_start = now,
            ResourceAttribute::ResponseStart => self.response_start = now,
            ResourceAttribute::RedirectStart(val) => match val {
                RedirectStartValue::Zero => self.redirect_start = 0,
                RedirectStartValue::FetchStart => {
                    if self.redirect_start.is_zero() {
                        self.redirect_start = self.fetch_start
                    }
                },
            },
            ResourceAttribute::RedirectEnd(val) => match val {
                RedirectEndValue::Zero => self.redirect_end = 0,
                RedirectEndValue::ResponseEnd => self.redirect_end = self.response_end,
            },
            ResourceAttribute::FetchStart => self.fetch_start = now,
            ResourceAttribute::ConnectStart(val) => self.connect_start = val,
            ResourceAttribute::ConnectEnd(val) => self.connect_end = val,
            ResourceAttribute::SecureConnectionStart => self.secure_connection_start = now,
            ResourceAttribute::ResponseEnd => self.response_end = now,
            ResourceAttribute::StartTime(val) => match val {
                ResourceTimeValue::RedirectStart
                    if self.redirect_start.is_zero() || !self.timing_check_passed => {},
                _ => self.start_time = self.get_time_value(val),
            },
        }
    }

    fn get_time_value(&self, time: ResourceTimeValue) -> u64 {
        match time {
            ResourceTimeValue::Zero => 0,
            ResourceTimeValue::Now => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            ResourceTimeValue::FetchStart => self.fetch_start,
            ResourceTimeValue::RedirectStart => self.redirect_start,
        }
    }

    pub fn mark_timing_check_failed(&mut self) {
        self.timing_check_passed = false;
        self.domain_lookup_start = 0;
        self.redirect_count = 0;
        self.request_start = 0;
        self.response_start = 0;
        self.redirect_start = 0;
        self.connect_start = 0;
        self.connect_end = 0;
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
    pub status: Option<(u16, Vec<u8>)>,

    /// Is successful HTTPS connection
    pub https_state: HttpsState,

    /// Referrer Url
    pub referrer: Option<ServoUrl>,

    /// Referrer Policy of the Request used to obtain Response
    pub referrer_policy: Option<ReferrerPolicy>,
    /// Performance information for navigation events
    pub timing: Option<ResourceFetchTiming>,
    /// True if the request comes from a redirection
    pub redirected: bool,
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
            // https://fetch.spec.whatwg.org/#concept-response-status-message
            status: Some((200, b"".to_vec())),
            https_state: HttpsState::None,
            referrer: None,
            referrer_policy: None,
            timing: None,
            redirected: false,
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
    pub fn set_referrer_policy(&mut self, referrer_policy: Option<ReferrerPolicy>) {
        if self.headers.is_none() {
            self.headers = Some(Serde(HeaderMap::new()));
        }

        self.referrer_policy = referrer_policy;
        if let Some(referrer_policy) = referrer_policy {
            self.headers
                .as_mut()
                .unwrap()
                .typed_insert::<ReferrerPolicyHeader>(referrer_policy.into());
        }
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

/// Network errors that have to be exported out of the loaders
#[derive(Clone, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum NetworkError {
    /// Could be any of the internal errors, like unsupported scheme, connection errors, etc.
    Internal(String),
    LoadCancelled,
    /// SSL validation error, to be converted to Resource::BadCertHTML in the HTML parser.
    SslValidation(String, Vec<u8>),
    /// Crash error, to be converted to Resource::Crash in the HTML parser.
    Crash(String),
}

impl NetworkError {
    pub fn from_hyper_error(error: &HyperError, certificate: Option<Certificate>) -> Self {
        let error_string = error.to_string();
        match certificate {
            Some(certificate) => NetworkError::SslValidation(error_string, certificate.0),
            _ => NetworkError::Internal(error_string),
        }
    }

    pub fn from_http_error(error: &HttpError) -> Self {
        NetworkError::Internal(error.to_string())
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

lazy_static! {
    pub static ref PRIVILEGED_SECRET: u32 = servo_rand::ServoRng::default().next_u32();
}
