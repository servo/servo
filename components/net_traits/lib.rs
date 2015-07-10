/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(custom_derive)]
#![feature(plugin)]
#![feature(slice_patterns)]
#![feature(step_by)]
#![feature(vec_push_all)]
#![plugin(serde_macros)]

extern crate serde;
extern crate euclid;
extern crate hyper;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate png;
extern crate serde;
extern crate stb_image;
extern crate url;
extern crate util;
extern crate msg;

use hyper::header::{ContentType, Header, Headers, HeadersItems};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, Attr};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use msg::constellation_msg::{PipelineId};
use serde::de;
use serde::ser;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::thread;

pub mod image_cache_task;
pub mod storage_task;

/// Image handling.
///
/// It may be surprising that this goes in the network crate as opposed to the graphics crate.
/// However, image handling is generally very integrated with the network stack (especially where
/// caching is involved) and as a result it must live in here.
pub mod image {
    pub mod base;
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LoadData {
    pub url: SerializableUrl,
    pub method: SerializableMethod,
    /// Headers that will apply to the initial request only
    pub headers: SerializableHeaders,
    /// Headers that will apply to the initial request and any redirects
    pub preserved_headers: SerializableHeaders,
    pub data: Option<Vec<u8>>,
    pub cors: Option<ResourceCORSData>,
    pub pipeline_id: Option<PipelineId>,
}

impl LoadData {
    pub fn new(url: Url, id: Option<PipelineId>) -> LoadData {
        LoadData {
            url: SerializableUrl(url),
            method: SerializableMethod(Method::Get),
            headers: SerializableHeaders(Headers::new()),
            preserved_headers: SerializableHeaders(Headers::new()),
            data: None,
            cors: None,
            pipeline_id: id,
        }
    }
}

/// A listener for asynchronous network events. Cancelling the underlying request is unsupported.
pub trait AsyncResponseListener {
    /// The response headers for a request have been received.
    fn headers_available(&self, metadata: Metadata);
    /// A portion of the response body has been received. This data is unavailable after
    /// this method returned, and must be stored accordingly.
    fn data_available(&self, payload: Vec<u8>);
    /// The response is complete. If the provided status is an Err value, there is no guarantee
    /// that the response body was completely read.
    fn response_complete(&self, status: Result<(), String>);
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
    ResponseComplete(SerializableStringResult)
}

impl ResponseAction {
    /// Execute the default action on a provided listener.
    pub fn process(self, listener: &AsyncResponseListener) {
        match self {
            ResponseAction::HeadersAvailable(m) => listener.headers_available(m),
            ResponseAction::DataAvailable(d) => listener.data_available(d),
            ResponseAction::ResponseComplete(r) => listener.response_complete(r.0),
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

#[derive(Deserialize, Serialize)]
pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(LoadData, LoadConsumer),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(SerializableUrl, String, CookieSource),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(SerializableUrl, IpcSender<Option<String>>, CookieSource),
    Exit
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

    /// Initiate the network request associated with this pending load.
    pub fn load(mut self) -> IpcReceiver<LoadResponse> {
        self.guard.neuter();
        let load_data = LoadData::new(self.url, self.pipeline);
        let (sender, receiver) = ipc::channel().unwrap();
        let consumer = LoadConsumer::Channel(sender);
        self.resource_task.send(ControlMsg::Load(load_data, consumer)).unwrap();
        receiver
    }

    /// Initiate the network request associated with this pending load, using the provided target.
    pub fn load_async(mut self, listener: AsyncResponseTarget) {
        self.guard.neuter();
        let load_data = LoadData::new(self.url, self.pipeline);
        let consumer = LoadConsumer::Listener(listener);
        self.resource_task.send(ControlMsg::Load(load_data, consumer)).unwrap();
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

#[derive(Clone, Deserialize, Serialize)]
pub struct ResourceCORSData {
    /// CORS Preflight flag
    pub preflight: bool,
    /// Origin of CORS Request
    pub origin: SerializableUrl,
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone, Deserialize, Serialize)]
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: SerializableUrl,

    /// MIME type / subtype.
    pub content_type: Option<(SerializableContentType)>,

    /// Character set.
    pub charset: Option<String>,

    /// Headers
    pub headers: Option<SerializableHeaders>,

    /// HTTP Status
    pub status: Option<SerializableRawStatus>,
}

impl Metadata {
    /// Metadata with defaults for everything optional.
    pub fn default(url: Url) -> Self {
        Metadata {
            final_url:    SerializableUrl(url),
            content_type: None,
            charset:      None,
            headers: None,
            // https://fetch.spec.whatwg.org/#concept-response-status-message
            status: Some(SerializableRawStatus(RawStatus(200, "OK".into()))),
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        match content_type {
            None => (),
            Some(mime) => {
                self.content_type = Some(SerializableContentType(ContentType(mime.clone())));
                let &Mime(_, _, ref parameters) = mime;
                for &(ref k, ref v) in parameters.iter() {
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
    Done(SerializableStringResult)
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(resource_task: &ResourceTask, url: Url)
        -> Result<(Metadata, Vec<u8>), String> {
    let (start_chan, start_port) = ipc::channel().unwrap();
    resource_task.send(ControlMsg::Load(LoadData::new(url, None),
                       LoadConsumer::Channel(start_chan))).unwrap();
    let response = start_port.recv().unwrap();

    let mut buf = vec!();
    loop {
        match response.progress_port.recv().unwrap() {
            ProgressMsg::Payload(data) => buf.push_all(&data),
            ProgressMsg::Done(SerializableStringResult(Ok(()))) => {
                return Ok((response.metadata, buf))
            }
            ProgressMsg::Done(SerializableStringResult(Err(e))) => return Err(e)
        }
    }
}

/// Load a URL asynchronously and iterate over chunks of bytes from the response.
pub fn load_bytes_iter(pending: PendingAsyncLoad) -> (Metadata, ProgressMsgPortIterator) {
    let input_port = pending.load();
    let response = input_port.recv().unwrap();
    let iter = ProgressMsgPortIterator {
        progress_port: response.progress_port
    };
    (response.metadata, iter)
}

/// Iterator that reads chunks of bytes from a ProgressMsg port
pub struct ProgressMsgPortIterator {
    progress_port: IpcReceiver<ProgressMsg>,
}

impl Iterator for ProgressMsgPortIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        match self.progress_port.recv().unwrap() {
            ProgressMsg::Payload(data) => Some(data),
            ProgressMsg::Done(SerializableStringResult(Ok(())))  => None,
            ProgressMsg::Done(SerializableStringResult(Err(e)))  => {
                error!("error receiving bytes: {}", e);
                None
            }
        }
    }
}

#[derive(Clone)]
pub struct SerializableMethod(pub Method);

impl Deref for SerializableMethod {
    type Target = Method;

    fn deref(&self) -> &Method {
        &self.0
    }
}

impl Serialize for SerializableMethod {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        format!("{}", self.0).serialize(serializer)
    }
}

impl Deserialize for SerializableMethod {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableMethod, D::Error>
                      where D: Deserializer {
        let string_representation: String = try!(Deserialize::deserialize(deserializer));
        Ok(SerializableMethod(FromStr::from_str(&string_representation[..]).unwrap()))
    }
}

#[derive(Clone)]
pub struct SerializableHeaders(pub Headers);

impl Deref for SerializableHeaders {
    type Target = Headers;

    fn deref(&self) -> &Headers {
        &self.0
    }
}

impl DerefMut for SerializableHeaders {
    fn deref_mut(&mut self) -> &mut Headers {
        &mut self.0
    }
}

impl Serialize for SerializableHeaders {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        struct HeadersVisitor<'a> {
            iter: HeadersItems<'a>,
            len: usize,
        }

        impl<'a> ser::MapVisitor for HeadersVisitor<'a> {
            fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                        where S: Serializer {
                match self.iter.next() {
                    Some(header_item) => {
                        try!(serializer.visit_map_elt(header_item.name(),
                                                      header_item.value_string()));
                        Ok(Some(()))
                    }
                    None => Ok(None),
                }
            }

            fn len(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        serializer.visit_map(HeadersVisitor {
            iter: self.iter(),
            len: self.len(),
        })
    }
}

impl Deserialize for SerializableHeaders {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableHeaders, D::Error>
                      where D: Deserializer {
        struct HeadersVisitor;

        impl de::Visitor for HeadersVisitor {
            type Value = SerializableHeaders;

            fn visit_map<V>(&mut self, mut visitor: V) -> Result<SerializableHeaders, V::Error>
                            where V: de::MapVisitor {
                let mut result = Headers::new();
                while let Some((key, value)) = try!(visitor.visit()) {
                    let (key, value): (String, String) = (key, value);
                    result.set_raw(key, vec![value.into_bytes()]);
                }
                try!(visitor.end());
                Ok(SerializableHeaders(result))
            }
        }

        let result = SerializableHeaders(Headers::new());
        try!(deserializer.visit_map(HeadersVisitor));
        Ok(result)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SerializableUrl(pub Url);

impl Deref for SerializableUrl {
    type Target = Url;

    fn deref(&self) -> &Url {
        &self.0
    }
}

impl DerefMut for SerializableUrl {
    fn deref_mut(&mut self) -> &mut Url {
        &mut self.0
    }
}

impl Serialize for SerializableUrl {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        format!("{}", self.0).serialize(serializer)
    }
}

impl Deserialize for SerializableUrl {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableUrl, D::Error>
                      where D: Deserializer {
        let string_representation: String = try!(Deserialize::deserialize(deserializer));
        Ok(SerializableUrl(FromStr::from_str(&string_representation[..]).unwrap()))
    }
}

#[derive(Clone, PartialEq)]
pub struct SerializableContentType(pub ContentType);

impl Deref for SerializableContentType {
    type Target = ContentType;

    fn deref(&self) -> &ContentType {
        &self.0
    }
}

impl Serialize for SerializableContentType {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        format!("{}", self.0).serialize(serializer)
    }
}

impl Deserialize for SerializableContentType {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableContentType, D::Error>
                      where D: Deserializer {
        let string_representation: String = try!(Deserialize::deserialize(deserializer));
        Ok(SerializableContentType(Header::parse_header(
                    &[string_representation.into_bytes()]).unwrap()))
    }
}

#[derive(Clone, PartialEq)]
pub struct SerializableRawStatus(pub RawStatus);

impl Deref for SerializableRawStatus {
    type Target = RawStatus;

    fn deref(&self) -> &RawStatus {
        &self.0
    }
}

impl Serialize for SerializableRawStatus {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        ((self.0).0, (self.0).1.clone().into_owned()).serialize(serializer)
    }
}

impl Deserialize for SerializableRawStatus {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableRawStatus, D::Error>
                      where D: Deserializer {
        let representation: (u16, String) = try!(Deserialize::deserialize(deserializer));
        Ok(SerializableRawStatus(RawStatus(representation.0, Cow::Owned(representation.1))))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SerializableStringResult(pub Result<(),String>);

#[derive(Deserialize, Serialize)]
enum SerializableStringResultInternal {
    Ok(()),
    Err(String),
}

impl Deref for SerializableStringResult {
    type Target = Result<(),String>;

    fn deref(&self) -> &Result<(),String> {
        &self.0
    }
}

impl Serialize for SerializableStringResult {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        let result = match **self {
            Ok(ref value) => SerializableStringResultInternal::Ok(*value),
            Err(ref value) => SerializableStringResultInternal::Err((*value).clone()),
        };
        result.serialize(serializer)
    }
}

impl Deserialize for SerializableStringResult {
    fn deserialize<D>(deserializer: &mut D) -> Result<SerializableStringResult, D::Error>
                      where D: Deserializer {
        let result: SerializableStringResultInternal =
            try!(Deserialize::deserialize(deserializer));
        match result {
            SerializableStringResultInternal::Ok(value) => Ok(SerializableStringResult(Ok(value))),
            SerializableStringResultInternal::Err(value) => {
                Ok(SerializableStringResult(Err(value)))
            }
        }
    }
}

