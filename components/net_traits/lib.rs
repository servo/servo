/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(iterator_step_by)]

#![deny(unsafe_code)]

extern crate cookie as cookie_rs;
extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
extern crate hyper;
extern crate hyper_serde;
extern crate image as piston_image;
extern crate ipc_channel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate msg;
extern crate num_traits;
#[macro_use] extern crate serde;
extern crate servo_config;
extern crate servo_url;
extern crate url;
extern crate uuid;
extern crate webrender_traits;

use cookie_rs::Cookie;
use filemanager_thread::FileManagerThreadMsg;
use heapsize::HeapSizeOf;
use hyper::Error as HyperError;
use hyper::header::{ContentType, Headers, ReferrerPolicy as ReferrerPolicyHeader};
use hyper::http::RawStatus;
use hyper::mime::{Attr, Mime};
use hyper_serde::Serde;
use ipc_channel::Error as IpcError;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use request::{Request, RequestInit};
use response::{HttpsState, Response, ResponseInit};
use servo_url::ServoUrl;
use std::error::Error;
use storage_thread::StorageThreadMsg;

pub mod blob_url_store;
pub mod filemanager_thread;
pub mod image_cache;
pub mod net_error_list;
pub mod pub_domains;
pub mod request;
pub mod response;
pub mod storage_thread;

/// Image handling.
///
/// It may be surprising that this goes in the network crate as opposed to the graphics crate.
/// However, image handling is generally very integrated with the network stack (especially where
/// caching is involved) and as a result it must live in here.
pub mod image {
    pub mod base;
}

/// A loading context, for context-specific sniffing, as defined in
/// https://mimesniff.spec.whatwg.org/#context-specific-sniffing
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
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

#[derive(Clone, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct CustomResponse {
    #[ignore_heap_size_of = "Defined in hyper"]
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub headers: Headers,
    #[ignore_heap_size_of = "Defined in hyper"]
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub raw_status: RawStatus,
    pub body: Vec<u8>,
}

impl CustomResponse {
    pub fn new(headers: Headers, raw_status: RawStatus, body: Vec<u8>) -> CustomResponse {
        CustomResponse {
            headers: headers,
            raw_status: raw_status,
            body: body,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CustomResponseMediator {
    pub response_chan: IpcSender<Option<CustomResponse>>,
    pub load_url: ServoUrl,
}

/// [Policies](https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-states)
/// for providing a referrer header for a request
#[derive(Clone, Copy, Debug, Deserialize, HeapSizeOf, Serialize)]
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

impl<'a> From<&'a ReferrerPolicyHeader> for ReferrerPolicy {
    fn from(policy: &'a ReferrerPolicyHeader) -> Self {
        match *policy {
            ReferrerPolicyHeader::NoReferrer =>
                ReferrerPolicy::NoReferrer,
            ReferrerPolicyHeader::NoReferrerWhenDowngrade =>
                ReferrerPolicy::NoReferrerWhenDowngrade,
            ReferrerPolicyHeader::SameOrigin =>
                ReferrerPolicy::SameOrigin,
            ReferrerPolicyHeader::Origin =>
                ReferrerPolicy::Origin,
            ReferrerPolicyHeader::OriginWhenCrossOrigin =>
                ReferrerPolicy::OriginWhenCrossOrigin,
            ReferrerPolicyHeader::UnsafeUrl =>
                ReferrerPolicy::UnsafeUrl,
            ReferrerPolicyHeader::StrictOrigin =>
                ReferrerPolicy::StrictOrigin,
            ReferrerPolicyHeader::StrictOriginWhenCrossOrigin =>
                ReferrerPolicy::StrictOriginWhenCrossOrigin,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum FetchResponseMsg {
    // todo: should have fields for transmitted/total bytes
    ProcessRequestBody,
    ProcessRequestEOF,
    // todo: send more info about the response (or perhaps the entire Response)
    ProcessResponse(Result<FetchMetadata, NetworkError>),
    ProcessResponseChunk(Vec<u8>),
    ProcessResponseEOF(Result<(), NetworkError>),
}

pub trait FetchTaskTarget {
    /// https://fetch.spec.whatwg.org/#process-request-body
    ///
    /// Fired when a chunk of the request body is transmitted
    fn process_request_body(&mut self, request: &Request);

    /// https://fetch.spec.whatwg.org/#process-request-end-of-file
    ///
    /// Fired when the entire request finishes being transmitted
    fn process_request_eof(&mut self, request: &Request);

    /// https://fetch.spec.whatwg.org/#process-response
    ///
    /// Fired when headers are received
    fn process_response(&mut self, response: &Response);

    /// Fired when a chunk of response content is received
    fn process_response_chunk(&mut self, chunk: Vec<u8>);

    /// https://fetch.spec.whatwg.org/#process-response-end-of-file
    ///
    /// Fired when the response is fully fetched
    fn process_response_eof(&mut self, response: &Response);
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FilteredMetadata {
    Basic(Metadata),
    Cors(Metadata),
    Opaque,
    OpaqueRedirect
}

#[derive(Clone, Serialize, Deserialize)]
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
    fn process_response_eof(&mut self, response: Result<(), NetworkError>);
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
        if response.is_network_error() {
            // todo: finer grained errors
            let _ =
                self.send(FetchResponseMsg::ProcessResponseEOF(Err(NetworkError::Internal("Network error".into()))));
        } else {
            let _ = self.send(FetchResponseMsg::ProcessResponseEOF(Ok(())));
        }
    }
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
            FetchResponseMsg::ProcessResponseEOF(data) => listener.process_response_eof(data),
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
    where T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    /// send message T
    fn send(&self, T) -> IpcSendResult;
    /// get underlying sender
    fn sender(&self) -> IpcSender<T>;
}

// FIXME: Originally we will construct an Arc<ResourceThread> from ResourceThread
// in script_thread to avoid some performance pitfall. Now we decide to deal with
// the "Arc" hack implicitly in future.
// See discussion: http://logs.glob.uno/?c=mozilla%23servo&s=16+May+2016&e=16+May+2016#c430412
// See also: https://github.com/servo/servo/blob/735480/components/script/script_thread.rs#L313
#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceThreads {
    core_thread: CoreResourceThread,
    storage_thread: IpcSender<StorageThreadMsg>,
}

impl ResourceThreads {
    pub fn new(c: CoreResourceThread, s: IpcSender<StorageThreadMsg>) -> ResourceThreads {
        ResourceThreads {
            core_thread: c,
            storage_thread: s,
        }
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
impl HeapSizeOf for ResourceThreads {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum IncludeSubdomains {
    Included,
    NotIncluded,
}

#[derive(HeapSizeOf, Deserialize, Serialize)]
pub enum MessageData {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Deserialize, Serialize)]
pub enum WebSocketDomAction {
    SendMessage(MessageData),
    Close(Option<u16>, Option<String>),
}

#[derive(Deserialize, Serialize)]
pub enum WebSocketNetworkEvent {
    ConnectionEstablished {
        protocol_in_use: Option<String>,
    },
    MessageReceived(MessageData),
    Close(Option<u16>, String),
    Fail,
}

#[derive(Deserialize, Serialize)]
pub struct WebSocketCommunicate {
    pub event_sender: IpcSender<WebSocketNetworkEvent>,
    pub action_receiver: IpcReceiver<WebSocketDomAction>,
}

#[derive(Deserialize, Serialize)]
pub struct WebSocketConnectData {
    pub resource_url: ServoUrl,
    pub origin: String,
    pub protocols: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub enum CoreResourceMsg {
    Fetch(RequestInit, IpcSender<FetchResponseMsg>),
    /// Initiate a fetch in response to processing a redirection
    FetchRedirect(RequestInit, ResponseInit, IpcSender<FetchResponseMsg>),
    /// Try to make a websocket connection to a URL.
    WebsocketConnect(WebSocketCommunicate, WebSocketConnectData),
    /// Store a cookie for a given originating URL
    SetCookieForUrl(ServoUrl, Serde<Cookie<'static>>, CookieSource),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(ServoUrl, Vec<Serde<Cookie<'static>>>, CookieSource),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(ServoUrl, IpcSender<Option<String>>, CookieSource),
    /// Get a cookie by name for a given originating URL
    GetCookiesDataForUrl(ServoUrl, IpcSender<Vec<Serde<Cookie<'static>>>>, CookieSource),
    /// Cancel a network request corresponding to a given `ResourceId`
    Cancel(ResourceId),
    /// Synchronization message solely for knowing the state of the ResourceChannelManager loop
    Synchronize(IpcSender<()>),
    /// Send the network sender in constellation to CoreResourceThread
    NetworkMediator(IpcSender<CustomResponseMediator>),
    /// Message forwarded to file manager's handler
    ToFileManager(FileManagerThreadMsg),
    /// Break the load handler loop, send a reply when done cleaning up local resources
    /// and exit
    Exit(IpcSender<()>),
}

/// Instruct the resource thread to make a new request.
pub fn fetch_async<F>(request: RequestInit, core_resource_thread: &CoreResourceThread, f: F)
    where F: Fn(FetchResponseMsg) + Send + 'static,
{
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    ROUTER.add_route(action_receiver.to_opaque(),
                     box move |message| f(message.to().unwrap()));
    core_resource_thread.send(CoreResourceMsg::Fetch(request, action_sender)).unwrap();
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct ResourceCorsData {
    /// CORS Preflight flag
    pub preflight: bool,
    /// Origin of CORS Request
    pub origin: ServoUrl,
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: ServoUrl,

    #[ignore_heap_size_of = "Defined in hyper"]
    /// MIME type / subtype.
    pub content_type: Option<Serde<ContentType>>,

    /// Character set.
    pub charset: Option<String>,

    #[ignore_heap_size_of = "Defined in hyper"]
    /// Headers
    pub headers: Option<Serde<Headers>>,

    /// HTTP Status
    pub status: Option<(u16, Vec<u8>)>,

    /// Is successful HTTPS connection
    pub https_state: HttpsState,

    /// Referrer Url
    pub referrer: Option<ServoUrl>,

    /// Referrer Policy of the Request used to obtain Response
    pub referrer_policy: Option<ReferrerPolicy>,
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: ServoUrl) -> Self {
        Metadata {
            final_url: url,
            content_type: None,
            charset: None,
            headers: None,
            // https://fetch.spec.whatwg.org/#concept-response-status-message
            status: Some((200, b"OK".to_vec())),
            https_state: HttpsState::None,
            referrer: None,
            referrer_policy: None,
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        if self.headers.is_none() {
            self.headers = Some(Serde(Headers::new()));
        }

        if let Some(mime) = content_type {
            self.headers.as_mut().unwrap().set(ContentType(mime.clone()));
            self.content_type = Some(Serde(ContentType(mime.clone())));
            let Mime(_, _, ref parameters) = *mime;
            for &(ref k, ref v) in parameters {
                if Attr::Charset == *k {
                    self.charset = Some(v.to_string());
                }
            }
        }
    }
}

/// The creator of a given cookie
#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum CookieSource {
    /// An HTTP API
    HTTP,
    /// A non-HTTP API
    NonHTTP,
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(request: RequestInit,
                           core_resource_thread: &CoreResourceThread)
                           -> Result<(Metadata, Vec<u8>), NetworkError> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    core_resource_thread.send(CoreResourceMsg::Fetch(request, action_sender)).unwrap();

    let mut buf = vec![];
    let mut metadata = None;
    loop {
        match action_receiver.recv().unwrap() {
            FetchResponseMsg::ProcessRequestBody |
            FetchResponseMsg::ProcessRequestEOF => (),
            FetchResponseMsg::ProcessResponse(Ok(m)) => {
                metadata = Some(match m {
                    FetchMetadata::Unfiltered(m) => m,
                    FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                })
            },
            FetchResponseMsg::ProcessResponseChunk(data) => buf.extend_from_slice(&data),
            FetchResponseMsg::ProcessResponseEOF(Ok(())) => return Ok((metadata.unwrap(), buf)),
            FetchResponseMsg::ProcessResponse(Err(e)) |
            FetchResponseMsg::ProcessResponseEOF(Err(e)) => return Err(e),
        }
    }
}

/// An unique identifier to keep track of each load message in the resource handler
#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct ResourceId(pub u32);

/// Network errors that have to be exported out of the loaders
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, HeapSizeOf)]
pub enum NetworkError {
    /// Could be any of the internal errors, like unsupported scheme, connection errors, etc.
    Internal(String),
    LoadCancelled,
    /// SSL validation error that has to be handled in the HTML parser
    SslValidation(ServoUrl, String),
}

impl NetworkError {
    pub fn from_hyper_error(url: &ServoUrl, error: HyperError) -> Self {
        if let HyperError::Ssl(ref ssl_error) = error {
            return NetworkError::from_ssl_error(url, &**ssl_error);
        }
        NetworkError::Internal(error.description().to_owned())
    }

    pub fn from_ssl_error(url: &ServoUrl, error: &Error) -> Self {
        NetworkError::SslValidation(url.clone(), error.description().to_owned())
    }
}

/// Normalize `slice`, as defined by
/// [the Fetch Spec](https://fetch.spec.whatwg.org/#concept-header-value-normalize).
pub fn trim_http_whitespace(mut slice: &[u8]) -> &[u8] {
    const HTTP_WS_BYTES: &'static [u8] = b"\x09\x0A\x0D\x20";

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
