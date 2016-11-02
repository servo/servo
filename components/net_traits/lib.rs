/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(plugin)]
#![feature(proc_macro)]
#![feature(rustc_attrs)]
#![feature(slice_patterns)]
#![feature(step_by)]
#![feature(structural_match)]
#![plugin(heapsize_plugin)]

#![deny(unsafe_code)]

extern crate cookie as cookie_rs;
extern crate heapsize;
extern crate hyper;
extern crate hyper_serde;
extern crate image as piston_image;
extern crate ipc_channel;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate msg;
extern crate num_traits;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;
extern crate util;
extern crate uuid;
extern crate websocket;

use cookie_rs::Cookie;
use filemanager_thread::FileManagerThreadMsg;
use heapsize::HeapSizeOf;
use hyper::header::{ContentType, Headers};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Attr, Mime};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use msg::constellation_msg::{PipelineId, ReferrerPolicy};
use request::{Request, RequestInit};
use response::{HttpsState, Response};
use std::io::Error as IOError;
use storage_thread::StorageThreadMsg;
use url::Url;
use websocket::header;

pub mod blob_url_store;
pub mod bluetooth_blacklist;
pub mod bluetooth_scanfilter;
pub mod bluetooth_thread;
pub mod filemanager_thread;
pub mod hosts;
pub mod image_cache_thread;
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
    pub body: Vec<u8>
}

impl CustomResponse {
    pub fn new(headers: Headers, raw_status: RawStatus, body: Vec<u8>) -> CustomResponse {
        CustomResponse { headers: headers, raw_status: raw_status, body: body }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CustomResponseMediator {
    pub response_chan: IpcSender<Option<CustomResponse>>,
    pub load_url: Url
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct LoadData {
    pub url: Url,
    #[ignore_heap_size_of = "Defined in hyper"]
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub method: Method,
    #[ignore_heap_size_of = "Defined in hyper"]
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    /// Headers that will apply to the initial request only
    pub headers: Headers,
    #[ignore_heap_size_of = "Defined in hyper"]
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    /// Headers that will apply to the initial request and any redirects
    /// Unused in fetch
    pub preserved_headers: Headers,
    pub data: Option<Vec<u8>>,
    pub cors: Option<ResourceCORSData>,
    pub pipeline_id: Option<PipelineId>,
    // https://fetch.spec.whatwg.org/#concept-http-fetch step 4.3
    pub credentials_flag: bool,
    pub context: LoadContext,
    /// The policy and referring URL for the originator of this request
    pub referrer_policy: Option<ReferrerPolicy>,
    pub referrer_url: Option<Url>
}

impl LoadData {
    pub fn new(context: LoadContext,
               url: Url,
               load_origin: &LoadOrigin) -> LoadData {
        LoadData {
            url: url,
            method: Method::Get,
            headers: Headers::new(),
            preserved_headers: Headers::new(),
            data: None,
            cors: None,
            pipeline_id: load_origin.pipeline_id(),
            credentials_flag: true,
            context: context,
            referrer_policy: load_origin.referrer_policy(),
            referrer_url: load_origin.referrer_url().clone(),
        }
    }
}

pub trait LoadOrigin {
    fn referrer_url(&self) -> Option<Url>;
    fn referrer_policy(&self) -> Option<ReferrerPolicy>;
    fn pipeline_id(&self) -> Option<PipelineId>;
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

#[derive(Serialize, Deserialize)]
pub enum FilteredMetadata {
    Opaque,
    Transparent(Metadata)
}

#[derive(Serialize, Deserialize)]
pub enum FetchMetadata {
    Unfiltered(Metadata),
    Filtered {
        filtered: FilteredMetadata,
        unsafe_: Metadata
    }
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
            let _ = self.send(FetchResponseMsg::ProcessResponseEOF(
                              Err(NetworkError::Internal("Network error".into()))));
        } else {
            let _ = self.send(FetchResponseMsg::ProcessResponseEOF(Ok(())));
        }
    }
}


pub trait Action<Listener> {
    fn process(self, listener: &mut Listener);
}

/// A listener for asynchronous network events. Cancelling the underlying request is unsupported.
pub trait AsyncResponseListener {
    /// The response headers for a request have been received.
    fn headers_available(&mut self, metadata: Result<Metadata, NetworkError>);
    /// A portion of the response body has been received. This data is unavailable after
    /// this method returned, and must be stored accordingly.
    fn data_available(&mut self, payload: Vec<u8>);
    /// The response is complete. If the provided status is an Err value, there is no guarantee
    /// that the response body was completely read.
    fn response_complete(&mut self, status: Result<(), NetworkError>);
}

/// Data for passing between threads/processes to indicate a particular action to
/// take on a provided network listener.
#[derive(Deserialize, Serialize)]
pub enum ResponseAction {
    /// Invoke headers_available
    HeadersAvailable(Result<Metadata, NetworkError>),
    /// Invoke data_available
    DataAvailable(Vec<u8>),
    /// Invoke response_complete
    ResponseComplete(Result<(), NetworkError>)
}

impl<T: AsyncResponseListener> Action<T> for ResponseAction {
    /// Execute the default action on a provided listener.
    fn process(self, listener: &mut T) {
        match self {
            ResponseAction::HeadersAvailable(m) => listener.headers_available(m),
            ResponseAction::DataAvailable(d) => listener.data_available(d),
            ResponseAction::ResponseComplete(r) => listener.response_complete(r),
        }
    }
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

/// A target for async networking events. Commonly used to dispatch a runnable event to another
/// thread storing the wrapped closure for later execution.
#[derive(Deserialize, Serialize)]
pub struct AsyncResponseTarget {
    pub sender: IpcSender<ResponseAction>,
}

impl AsyncResponseTarget {
    pub fn invoke_with_listener(&self, action: ResponseAction) {
        self.sender.send(action).unwrap()
    }
}

/// A wrapper for a network load that can either be channel or event-based.
#[derive(Deserialize, Serialize)]
pub enum LoadConsumer {
    Channel(IpcSender<LoadResponse>),
    Listener(AsyncResponseTarget),
}

/// Handle to a resource thread
pub type CoreResourceThread = IpcSender<CoreResourceMsg>;

pub type IpcSendResult = Result<(), IOError>;

/// Abstraction of the ability to send a particular type of message,
/// used by net_traits::ResourceThreads to ease the use its IpcSender sub-fields
/// XXX: If this trait will be used more in future, some auto derive might be appealing
pub trait IpcSend<T> where T: serde::Serialize + serde::Deserialize {
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
    pub fn new(c: CoreResourceThread,
               s: IpcSender<StorageThreadMsg>) -> ResourceThreads {
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
    fn heap_size_of_children(&self) -> usize { 0 }
}

#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum IncludeSubdomains {
    Included,
    NotIncluded
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
    ConnectionEstablished(
        #[serde(deserialize_with = "::hyper_serde::deserialize",
                serialize_with = "::hyper_serde::serialize")]
        header::Headers,
        Vec<String>
    ),
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
    pub resource_url: Url,
    pub origin: String,
    pub protocols: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub enum CoreResourceMsg {
    /// Request the data associated with a particular URL
    Load(LoadData, LoadConsumer, Option<IpcSender<ResourceId>>),
    Fetch(RequestInit, IpcSender<FetchResponseMsg>),
    /// Try to make a websocket connection to a URL.
    WebsocketConnect(WebSocketCommunicate, WebSocketConnectData),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(Url, String, CookieSource),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrlWithData(
        Url,
        #[serde(deserialize_with = "::hyper_serde::deserialize",
                serialize_with = "::hyper_serde::serialize")]
        Cookie,
        CookieSource
    ),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(Url, IpcSender<Option<String>>, CookieSource),
    /// Get a cookie by name for a given originating URL
    GetCookiesDataForUrl(Url, IpcSender<Vec<Serde<Cookie>>>, CookieSource),
    /// Cancel a network request corresponding to a given `ResourceId`
    Cancel(ResourceId),
    /// Synchronization message solely for knowing the state of the ResourceChannelManager loop
    Synchronize(IpcSender<()>),
    /// Send the network sender in constellation to CoreResourceThread
    NetworkMediator(IpcSender<CustomResponseMediator>),
    /// Message forwarded to file manager's handler
    ToFileManager(FileManagerThreadMsg),
    /// Break the load handler loop, send a reply when done cleaning up local resources
    //  and exit
    Exit(IpcSender<()>),
}

struct LoadOriginData {
    pipeline: Option<PipelineId>,
    referrer_policy: Option<ReferrerPolicy>,
    referrer_url: Option<Url>
}

impl LoadOrigin for LoadOriginData {
    fn referrer_url(&self) -> Option<Url> {
        self.referrer_url.clone()
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        self.referrer_policy.clone()
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        self.pipeline
    }
}

/// Instruct the resource thread to make a new request.
pub fn load_async(context: LoadContext,
                  core_resource_thread: CoreResourceThread,
                  url: Url,
                  pipeline: Option<PipelineId>,
                  referrer_policy: Option<ReferrerPolicy>,
                  referrer_url: Option<Url>,
                  listener: AsyncResponseTarget) {
    let load = LoadOriginData {
        pipeline: pipeline,
        referrer_policy: referrer_policy,
        referrer_url: referrer_url
    };
    let load_data = LoadData::new(context, url, &load);
    let consumer = LoadConsumer::Listener(listener);
    core_resource_thread.send(CoreResourceMsg::Load(load_data, consumer, None)).unwrap();
}

/// Message sent in response to `Load`.  Contains metadata, and a port
/// for receiving the data.
///
/// Even if loading fails immediately, we send one of these and the
/// progress_port will provide the error.
#[derive(Serialize, Deserialize)]
pub struct LoadResponse {
    /// Metadata, such as from HTTP headers.
    pub metadata: Metadata,
    /// Port for reading data.
    pub progress_port: IpcReceiver<ProgressMsg>,
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct ResourceCORSData {
    /// CORS Preflight flag
    pub preflight: bool,
    /// Origin of CORS Request
    pub origin: Url,
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: Url,

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
    pub referrer: Option<Url>,
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: Url) -> Self {
        Metadata {
            final_url:    url,
            content_type: None,
            charset:      None,
            headers: None,
            // https://fetch.spec.whatwg.org/#concept-response-status-message
            status: Some((200, b"OK".to_vec())),
            https_state: HttpsState::None,
            referrer: None,
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        match self.headers {
            None => self.headers = Some(Serde(Headers::new())),
            Some(_) => (),
        }

        match content_type {
            None => (),
            Some(mime) => {
                if let Some(headers) = self.headers.as_mut() {
                    headers.set(ContentType(mime.clone()));
                }

                self.content_type = Some(Serde(ContentType(mime.clone())));
                let &Mime(_, _, ref parameters) = mime;
                for &(ref k, ref v) in parameters {
                    if &Attr::Charset == k {
                        self.charset = Some(v.to_string());
                    }
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

/// Messages sent in response to a `Load` message
#[derive(PartialEq, Debug, Deserialize, Serialize)]
pub enum ProgressMsg {
    /// Binary data - there may be multiple of these
    Payload(Vec<u8>),
    /// Indicates loading is complete, either successfully or not
    Done(Result<(), NetworkError>),
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(context: LoadContext,
                           core_resource_thread: &CoreResourceThread,
                           url: Url,
                           load_origin: &LoadOrigin)
        -> Result<(Metadata, Vec<u8>), NetworkError> {
    let (start_chan, start_port) = ipc::channel().unwrap();
    let load_data = LoadData::new(context, url, load_origin);
    core_resource_thread.send(CoreResourceMsg::Load(load_data, LoadConsumer::Channel(start_chan), None)).unwrap();
    let response = start_port.recv().unwrap();

    let mut buf = vec!();
    loop {
        match response.progress_port.recv().unwrap() {
            ProgressMsg::Payload(data) => buf.extend_from_slice(&data),
            ProgressMsg::Done(Ok(())) => return Ok((response.metadata, buf)),
            ProgressMsg::Done(Err(e)) => return Err(e)
        }
    }
}

/// Defensively unwraps the protocol string from the response object's protocol
pub fn unwrap_websocket_protocol(wsp: Option<&header::WebSocketProtocol>) -> Option<&str> {
    wsp.and_then(|protocol_list| protocol_list.get(0).map(|protocol| protocol.as_ref()))
}

/// An unique identifier to keep track of each load message in the resource handler
#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct ResourceId(pub u32);

#[derive(Deserialize, Serialize)]
pub enum ConstellationMsg {
    /// Queries whether a pipeline or its ancestors are private
    IsPrivate(PipelineId, IpcSender<bool>),
}

/// Network errors that have to be exported out of the loaders
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, HeapSizeOf)]
pub enum NetworkError {
    /// Could be any of the internal errors, like unsupported scheme, connection errors, etc.
    Internal(String),
    LoadCancelled,
    /// SSL validation error that has to be handled in the HTML parser
    SslValidation(Url, String),
}

/// Normalize `slice`, as defined by
/// [the Fetch Spec](https://fetch.spec.whatwg.org/#concept-header-value-normalize).
pub fn trim_http_whitespace(mut slice: &[u8]) -> &[u8] {
    const HTTP_WS_BYTES: &'static [u8] = b"\x09\x0A\x0D\x20";

    loop {
        match slice.split_first() {
            Some((first, remainder)) if HTTP_WS_BYTES.contains(first) =>
                slice = remainder,
            _ => break,
        }
    }

    loop {
        match slice.split_last() {
            Some((last, remainder)) if HTTP_WS_BYTES.contains(last) =>
                slice = remainder,
            _ => break,
        }
    }

    slice
}
