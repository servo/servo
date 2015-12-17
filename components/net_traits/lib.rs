/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![feature(plugin)]
#![feature(slice_patterns)]
#![feature(step_by)]
#![feature(custom_attribute)]
#![plugin(serde_macros, plugins)]

extern crate euclid;
extern crate hyper;
extern crate image as piston_image;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate serde;
extern crate stb_image;
extern crate url;
extern crate util;
extern crate websocket;

use hyper::header::{ContentType, Headers};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Attr, Mime};
use hyper::status::StatusCode;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use msg::constellation_msg::{PipelineId};
use serde::{Deserializer, Serializer};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use url::Url;
use util::mem::HeapSizeOf;

pub mod hosts;
pub mod image_cache_task;
pub mod net_error_list;
pub mod storage_task;

/// [Response type](https://fetch.spec.whatwg.org/#concept-response-type)
#[derive(Clone, PartialEq, Copy)]
pub enum ResponseType {
    Basic,
    CORS,
    Default,
    Error,
    Opaque,
    OpaqueRedirect
}

/// [Response termination reason](https://fetch.spec.whatwg.org/#concept-response-termination-reason)
#[derive(Clone, Copy)]
pub enum TerminationReason {
    EndUserAbort,
    Fatal,
    Timeout
}

/// The response body can still be pushed to after fetch
/// This provides a way to store unfinished response bodies
#[derive(Clone)]
pub enum ResponseBody {
    Empty, // XXXManishearth is this necessary, or is Done(vec![]) enough?
    Receiving(Vec<u8>),
    Done(Vec<u8>),
}

pub enum ResponseMsg {
    Chunk(Vec<u8>),
    Finished,
    Errored
}

/// A [Response](https://fetch.spec.whatwg.org/#concept-response) as defined by the Fetch spec
#[derive(Clone)]
pub struct Response {
    pub response_type: ResponseType,
    pub termination_reason: Option<TerminationReason>,
    pub url: Option<Url>,
    pub url_list: Vec<Url>,
    /// `None` can be considered a StatusCode of `0`.
    pub status: Option<StatusCode>,
    pub headers: Headers,
    pub body: ResponseBody,
    /// [Internal response](https://fetch.spec.whatwg.org/#concept-internal-response), only used if the Response
    /// is a filtered response
    pub internal_response: Option<Rc<RefCell<Response>>>,
}

impl Response {
    pub fn network_error() -> Response {
        Response {
            response_type: ResponseType::Error,
            termination_reason: None,
            url: None,
            url_list: vec![],
            status: None,
            headers: Headers::new(),
            body: ResponseBody::Empty,
            internal_response: None
        }
    }

    pub fn is_network_error(&self) -> bool {
        match self.response_type {
            ResponseType::Error => true,
            _ => false
        }
    }
}

/// Image handling.
///
/// It may be surprising that this goes in the network crate as opposed to the graphics crate.
/// However, image handling is generally very integrated with the network stack (especially where
/// caching is involved) and as a result it must live in here.
pub mod image {
    pub mod base;
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct LoadData {
    pub url: Url,
    pub method: Method,
    #[ignore_heap_size_of = "Defined in hyper"]
    /// Headers that will apply to the initial request only
    pub headers: Headers,
    #[ignore_heap_size_of = "Defined in hyper"]
    /// Headers that will apply to the initial request and any redirects
    pub preserved_headers: Headers,
    pub data: Option<Vec<u8>>,
    pub cors: Option<ResourceCORSData>,
    pub pipeline_id: Option<PipelineId>,
    // https://fetch.spec.whatwg.org/#concept-http-fetch step 4.3
    pub credentials_flag: bool,
}

impl LoadData {
    pub fn new(url: Url, id: Option<PipelineId>) -> LoadData {
        LoadData {
            url: url,
            method: Method::Get,
            headers: Headers::new(),
            preserved_headers: Headers::new(),
            data: None,
            cors: None,
            pipeline_id: id,
            credentials_flag: true,
        }
    }
}

/// Interface for observing the final response for an asynchronous fetch operation.
pub trait AsyncFetchListener {
    fn response_available(&self, response: Response);
}

/// A listener for asynchronous network events. Cancelling the underlying request is unsupported.
pub trait AsyncResponseListener {
    /// The response headers for a request have been received.
    fn headers_available(&mut self, metadata: Metadata);
    /// A portion of the response body has been received. This data is unavailable after
    /// this method returned, and must be stored accordingly.
    fn data_available(&mut self, payload: Vec<u8>);
    /// The response is complete. If the provided status is an Err value, there is no guarantee
    /// that the response body was completely read.
    fn response_complete(&mut self, status: Result<(), String>);
}

/// Data for passing between threads/processes to indicate a particular action to
/// take on a provided network listener.
#[derive(Deserialize, Serialize)]
pub enum ResponseAction {
    /// Invoke headers_available
    HeadersAvailable(Metadata),
    /// Invoke data_available
    DataAvailable(Vec<u8>),
    /// Invoke response_complete
    ResponseComplete(Result<(), String>)
}

impl ResponseAction {
    /// Execute the default action on a provided listener.
    pub fn process(self, listener: &mut AsyncResponseListener) {
        match self {
            ResponseAction::HeadersAvailable(m) => listener.headers_available(m),
            ResponseAction::DataAvailable(d) => listener.data_available(d),
            ResponseAction::ResponseComplete(r) => listener.response_complete(r),
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

/// Handle to a resource task
pub type ResourceTask = IpcSender<ControlMsg>;

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
    Close(u16, String),
}

#[derive(Deserialize, Serialize)]
pub enum WebSocketNetworkEvent {
    ConnectionEstablished,
    MessageReceived(MessageData),
    Fail,
    Close,
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
}

#[derive(Deserialize, Serialize)]
pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(LoadData, LoadConsumer, Option<IpcSender<ResourceId>>),
    /// Try to make a websocket connection to a URL.
    WebsocketConnect(WebSocketCommunicate, WebSocketConnectData),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(Url, String, CookieSource),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(Url, IpcSender<Option<String>>, CookieSource),
    /// Cancel a network request corresponding to a given `ResourceId`
    Cancel(ResourceId),
    /// Synchronization message solely for knowing the state of the ResourceChannelManager loop
    Synchronize(IpcSender<()>),
    /// Break the load handler loop and exit
    Exit,
}

/// Initialized but unsent request. Encapsulates everything necessary to instruct
/// the resource task to make a new request. The `load` method *must* be called before
/// destruction or the task will panic.
pub struct PendingAsyncLoad {
    resource_task: ResourceTask,
    url: Url,
    pipeline: Option<PipelineId>,
    guard: PendingLoadGuard,
}

struct PendingLoadGuard {
    loaded: bool,
}

impl PendingLoadGuard {
    fn neuter(&mut self) {
        self.loaded = true;
    }
}

impl Drop for PendingLoadGuard {
    fn drop(&mut self) {
        if !thread::panicking() {
            assert!(self.loaded)
        }
    }
}

impl PendingAsyncLoad {
    pub fn new(resource_task: ResourceTask, url: Url, pipeline: Option<PipelineId>)
               -> PendingAsyncLoad {
        PendingAsyncLoad {
            resource_task: resource_task,
            url: url,
            pipeline: pipeline,
            guard: PendingLoadGuard { loaded: false, },
        }
    }

    /// Initiate the network request associated with this pending load, using the provided target.
    pub fn load_async(mut self, listener: AsyncResponseTarget) {
        self.guard.neuter();
        let load_data = LoadData::new(self.url, self.pipeline);
        let consumer = LoadConsumer::Listener(listener);
        self.resource_task.send(ControlMsg::Load(load_data, consumer, None)).unwrap();
    }
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

    /// MIME type / subtype.
    pub content_type: Option<(ContentType)>,

    /// Character set.
    pub charset: Option<String>,

    #[ignore_heap_size_of = "Defined in hyper"]
    /// Headers
    pub headers: Option<Headers>,

    /// HTTP Status
    pub status: Option<RawStatus>,
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
            status: Some(RawStatus(200, "OK".into())),
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        match content_type {
            None => (),
            Some(mime) => {
                self.content_type = Some(ContentType(mime.clone()));
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
    Done(Result<(), String>)
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(resource_task: &ResourceTask, url: Url, pipeline_id: Option<PipelineId>)
        -> Result<(Metadata, Vec<u8>), String> {
    let (start_chan, start_port) = ipc::channel().unwrap();
    resource_task.send(ControlMsg::Load(LoadData::new(url, pipeline_id),
                       LoadConsumer::Channel(start_chan), None)).unwrap();
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

/// An unique identifier to keep track of each load message in the resource handler
#[derive(Clone, PartialEq, Eq, Copy, Hash, Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct ResourceId(pub u32);
