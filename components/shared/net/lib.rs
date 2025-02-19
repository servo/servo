/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{LazyLock, OnceLock};
use std::thread;

use base::cross_process_instant::CrossProcessInstant;
use base::id::HistoryStateId;
use cookie::Cookie;
use crossbeam_channel::{unbounded, Receiver, Sender};
use headers::{ContentType, HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use http::{header, Error as HttpError, HeaderMap, HeaderValue, StatusCode};
use hyper_serde::Serde;
use hyper_util::client::legacy::Error as HyperError;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use ipc_channel::Error as IpcError;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use mime::Mime;
use request::RequestId;
use rustls_pki_types::CertificateDer;
use serde::{Deserialize, Serialize};
use servo_rand::RngCore;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::filemanager_thread::FileManagerThreadMsg;
use crate::http_status::HttpStatus;
use crate::request::{Request, RequestBuilder};
use crate::response::{HttpsState, Response, ResponseInit};
use crate::storage_thread::StorageThreadMsg;

pub mod blob_url_store;
pub mod filemanager_thread;
pub mod http_status;
pub mod image_cache;
pub mod policy_container;
pub mod pub_domains;
pub mod quality;
pub mod request;
pub mod response;
pub mod storage_thread;

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

impl From<Option<ReferrerPolicyHeader>> for ReferrerPolicy {
    fn from(header: Option<ReferrerPolicyHeader>) -> Self {
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
    ProcessResponseChunk(RequestId, Vec<u8>),
    ProcessResponseEOF(RequestId, Result<ResourceFetchTiming, NetworkError>),
}

impl FetchResponseMsg {
    pub fn request_id(&self) -> RequestId {
        match self {
            FetchResponseMsg::ProcessRequestBody(id) |
            FetchResponseMsg::ProcessRequestEOF(id) |
            FetchResponseMsg::ProcessResponse(id, ..) |
            FetchResponseMsg::ProcessResponseChunk(id, ..) |
            FetchResponseMsg::ProcessResponseEOF(id, ..) => *id,
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
}

pub trait FetchResponseListener {
    fn process_request_body(&mut self, request_id: RequestId);
    fn process_request_eof(&mut self, request_id: RequestId);
    fn process_response(
        &mut self,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    );
    fn process_response_chunk(&mut self, request_id: RequestId, chunk: Vec<u8>);
    fn process_response_eof(
        &mut self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    );
    fn resource_timing(&self) -> &ResourceFetchTiming;
    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming;
    fn submit_resource_timing(&mut self);
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
        let _ = self.send(FetchResponseMsg::ProcessResponseChunk(request.id, chunk));
    }

    fn process_response_eof(&mut self, request: &Request, response: &Response) {
        let payload = if let Some(network_error) = response.get_network_error() {
            Err(network_error.clone())
        } else {
            Ok(response.get_resource_timing().lock().unwrap().clone())
        };

        let _ = self.send(FetchResponseMsg::ProcessResponseEOF(request.id, payload));
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
}

pub trait Action<Listener> {
    fn process(self, listener: &mut Listener);
}

impl<T: FetchResponseListener> Action<T> for FetchResponseMsg {
    /// Execute the default action on a provided listener.
    fn process(self, listener: &mut T) {
        match self {
            FetchResponseMsg::ProcessRequestBody(request_id) => {
                listener.process_request_body(request_id)
            },
            FetchResponseMsg::ProcessRequestEOF(request_id) => {
                listener.process_request_eof(request_id)
            },
            FetchResponseMsg::ProcessResponse(request_id, meta) => {
                listener.process_response(request_id, meta)
            },
            FetchResponseMsg::ProcessResponseChunk(request_id, data) => {
                listener.process_response_chunk(request_id, data)
            },
            FetchResponseMsg::ProcessResponseEOF(request_id, data) => {
                match data {
                    Ok(ref response_resource_timing) => {
                        // update listener with values from response
                        *listener.resource_timing_mut() = response_resource_timing.clone();
                        listener
                            .process_response_eof(request_id, Ok(response_resource_timing.clone()));
                        // TODO timing check https://w3c.github.io/resource-timing/#dfn-timing-allow-check

                        listener.submit_resource_timing();
                    },
                    // TODO Resources for which the fetch was initiated, but was later aborted
                    // (e.g. due to a network error) MAY be included as PerformanceResourceTiming
                    // objects in the Performance Timeline and MUST contain initialized attribute
                    // values for processed substeps of the processing model.
                    Err(e) => listener.process_response_eof(request_id, Err(e)),
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

// FIXME: https://github.com/servo/servo/issues/34591
#[expect(clippy::large_enum_variant)]
enum ToFetchThreadMessage {
    Cancel(Vec<RequestId>),
    StartFetch(
        /* request_builder */ RequestBuilder,
        /* response_init */ Option<ResponseInit>,
        /* callback  */ BoxedFetchCallback,
    ),
    FetchResponse(FetchResponseMsg),
}

pub type BoxedFetchCallback = Box<dyn FnMut(FetchResponseMsg) + Send + 'static>;

/// A thread to handle fetches in a Servo process. This thread is responsible for
/// listening for new fetch requests as well as updates on those operations and forwarding
/// them to crossbeam channels.
struct FetchThread {
    /// A list of active fetches. A fetch is no longer active once the
    /// [`FetchResponseMsg::ProcessResponseEOF`] is received.
    active_fetches: HashMap<RequestId, BoxedFetchCallback>,
    /// A reference to the [`CoreResourceThread`] used to kick off fetch requests.
    core_resource_thread: CoreResourceThread,
    /// A crossbeam receiver attached to the router proxy which converts incoming fetch
    /// updates from IPC messages to crossbeam messages as well as another sender which
    /// handles requests from clients wanting to do fetches.
    receiver: Receiver<ToFetchThreadMessage>,
    /// An [`IpcSender`] that's sent with every fetch request and leads back to our
    /// router proxy.
    to_fetch_sender: IpcSender<FetchResponseMsg>,
}

impl FetchThread {
    fn spawn(core_resource_thread: &CoreResourceThread) -> Sender<ToFetchThreadMessage> {
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

        let core_resource_thread = core_resource_thread.clone();
        thread::Builder::new()
            .name("FetchThread".to_owned())
            .spawn(move || {
                let mut fetch_thread = FetchThread {
                    active_fetches: HashMap::new(),
                    core_resource_thread,
                    receiver,
                    to_fetch_sender,
                };
                fetch_thread.run();
            })
            .expect("Thread spawning failed");
        sender
    }

    fn run(&mut self) {
        loop {
            match self.receiver.recv().unwrap() {
                ToFetchThreadMessage::StartFetch(request_builder, response_init, callback) => {
                    self.active_fetches.insert(request_builder.id, callback);

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

                    self.core_resource_thread.send(message).unwrap();
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
                ToFetchThreadMessage::Cancel(request_ids) => {
                    // Errors are ignored here, because Servo sends many cancellation requests when shutting down.
                    // At this point the networking task might be shut down completely, so just ignore errors
                    // during this time.
                    let _ = self
                        .core_resource_thread
                        .send(CoreResourceMsg::Cancel(request_ids));
                },
            }
        }
    }
}

static FETCH_THREAD: OnceLock<Sender<ToFetchThreadMessage>> = OnceLock::new();

/// Instruct the resource thread to make a new fetch request.
pub fn fetch_async(
    core_resource_thread: &CoreResourceThread,
    request: RequestBuilder,
    response_init: Option<ResponseInit>,
    callback: BoxedFetchCallback,
) {
    let _ = FETCH_THREAD
        .get_or_init(|| FetchThread::spawn(core_resource_thread))
        .send(ToFetchThreadMessage::StartFetch(
            request,
            response_init,
            callback,
        ));
}

/// Instruct the resource thread to cancel an existing request. Does nothing if the
/// request has already completed or has not been fetched yet.
pub fn cancel_async_fetch(request_ids: Vec<RequestId>) {
    let _ = FETCH_THREAD
        .get()
        .expect("Tried to cancel request in process that hasn't started one.")
        .send(ToFetchThreadMessage::Cancel(request_ids));
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
    pub fn from_hyper_error(error: &HyperError, certificate: Option<CertificateDer>) -> Self {
        let error_string = error.to_string();
        match certificate {
            Some(certificate) => NetworkError::SslValidation(error_string, certificate.to_vec()),
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

pub fn set_default_accept_language(headers: &mut HeaderMap) {
    if headers.contains_key(header::ACCEPT_LANGUAGE) {
        return;
    }

    // TODO(eijebong): Change this once typed headers are done
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.5"),
    );
}

pub static PRIVILEGED_SECRET: LazyLock<u32> =
    LazyLock::new(|| servo_rand::ServoRng::default().next_u32());
