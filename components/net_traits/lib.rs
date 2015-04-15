/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(collections)]
#![feature(core)]
#![feature(rustc_private)]
#![feature(std_misc)]

extern crate geom;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate png;
extern crate profile;
extern crate stb_image;
extern crate url;
extern crate util;

use hyper::header::{ContentType, Headers};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, Attr};
use url::Url;

use std::borrow::IntoCow;
use std::sync::mpsc::{channel, Receiver, Sender};

pub mod image_cache_task;
pub mod local_image_cache;
pub mod storage_task;

/// Image handling.
///
/// It may be surprising that this goes in the network crate as opposed to the graphics crate.
/// However, image handling is generally very integrated with the network stack (especially where
/// caching is involved) and as a result it must live in here.
pub mod image {
    pub mod base;
    pub mod holder;
}

#[derive(Clone)]
pub struct LoadData {
    pub url: Url,
    pub method: Method,
    /// Headers that will apply to the initial request only
    pub headers: Headers,
    /// Headers that will apply to the initial request and any redirects
    pub preserved_headers: Headers,
    pub data: Option<Vec<u8>>,
    pub cors: Option<ResourceCORSData>,
    pub consumer: Sender<ProgressMsg>,
}

impl LoadData {
    pub fn new(url: Url, consumer: Sender<ProgressMsg>) -> LoadData {
        LoadData {
            url: url,
            method: Method::Get,
            headers: Headers::new(),
            preserved_headers: Headers::new(),
            data: None,
            cors: None,
            consumer: consumer,
        }
    }
}

/// Handle to a resource task
#[derive(Clone)]
pub struct ResourceTask(Sender<ControlMsg>);

impl ResourceTask {
    pub fn new(sender: Sender<ControlMsg>) -> ResourceTask {
        ResourceTask(sender)
    }

    pub fn load_url(&self, url: Url, consumer: Sender<ProgressMsg>) {
        let ResourceTask(ref sender) = *self;
        sender.send(ControlMsg::Load(LoadData::new(url.clone(), consumer))).unwrap();
    }

    pub fn load(&self, load_data: LoadData) {
        let ResourceTask(ref sender) = *self;
        sender.send(ControlMsg::Load(load_data)).unwrap();
    }

    pub fn set_cookies_for_url(&self, url: Url, cookie: String, source: CookieSource) {
        let ResourceTask(ref sender) = *self;
        sender.send(ControlMsg::SetCookiesForUrl(url, cookie, source)).unwrap();
    }

    pub fn get_cookies_for_url(&self,
                               url: Url,
                               consumer: Sender<Option<String>>,
                               source: CookieSource) {
        let ResourceTask(ref sender) = *self;
        sender.send(ControlMsg::GetCookiesForUrl(url, consumer, source)).unwrap();
    }

    pub fn exit(&self) {
        let ResourceTask(ref sender) = *self;
        sender.send(ControlMsg::Exit).unwrap();
    }
}

pub enum ControlMsg {
    /// Request the data associated with a particular URL
    Load(LoadData),
    /// Store a set of cookies for a given originating URL
    SetCookiesForUrl(Url, String, CookieSource),
    /// Retrieve the stored cookies for a given URL
    GetCookiesForUrl(Url, Sender<Option<String>>, CookieSource),
    Exit
}

/// Message sent by the resource task during an
/// asynchronous resource load. Headers is always
/// the first message type sent. This is followed
/// by zero or more Payloads, and then a Done message
/// indicating success or failure.
#[derive(Debug)]
pub enum ProgressMsg {
    Headers(Metadata),
    Payload(Vec<u8>),
    Done(Result<(), String>)
}

#[derive(Clone)]
pub struct ResourceCORSData {
    /// CORS Preflight flag
    pub preflight: bool,
    /// Origin of CORS Request
    pub origin: Url
}

/// Metadata about a loaded resource, such as is obtained from HTTP headers.
#[derive(Clone, Debug)]
pub struct Metadata {
    /// Final URL after redirects.
    pub final_url: Url,

    /// MIME type / subtype.
    pub content_type: Option<(ContentType)>,

    /// Character set.
    pub charset: Option<String>,

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
            status: Some(RawStatus(200, "OK".into_cow())),
        }
    }

    /// Extract the parts of a Mime that we care about.
    pub fn set_content_type(&mut self, content_type: Option<&Mime>) {
        match content_type {
            None => (),
            Some(mime) => {
                self.content_type = Some(ContentType(mime.clone()));
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
#[derive(PartialEq, Copy)]
pub enum CookieSource {
    /// An HTTP API
    HTTP,
    /// A non-HTTP API
    NonHTTP,
}

pub struct LoadInfo {
    pub metadata: Metadata,
    pub consumer: Receiver<ProgressMsg>,
}

/// Convenience function for synchronously loading a whole resource.
pub fn load_whole_resource(resource_task: &ResourceTask, url: Url)
        -> Result<(Metadata, Vec<u8>), String> {
    let (sender, receiver) = channel();
    resource_task.load_url(url, sender);

    let mut metadata = None;
    let mut buf = vec!();
    loop {
        match receiver.recv().unwrap() {
            ProgressMsg::Headers(data) => metadata = Some(data),
            ProgressMsg::Payload(data) => buf.push_all(&data),
            ProgressMsg::Done(Ok(()))  => return Ok((metadata.unwrap(), buf)),
            ProgressMsg::Done(Err(e))  => return Err(e)
        }
    }
}

/// Load a URL asynchronously and iterate over chunks of bytes from the response.
pub fn load_bytes_iter(resource_task: &ResourceTask, url: Url) -> (Metadata, ProgressMsgPortIterator) {
    let (input_chan, input_port) = channel();
    resource_task.load_url(url, input_chan);

    let response = input_port.recv().unwrap();
    match response {
        ProgressMsg::Headers(metadata) => {
            let iter = ProgressMsgPortIterator { progress_port: input_port };
            (metadata, iter)
        }
        _ => unreachable!(),
    }
}

/// Iterator that reads chunks of bytes from a ProgressMsg port
pub struct ProgressMsgPortIterator {
    progress_port: Receiver<ProgressMsg>
}

impl Iterator for ProgressMsgPortIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {
        match self.progress_port.recv().unwrap() {
            ProgressMsg::Headers(..) => unreachable!(),
            ProgressMsg::Payload(data) => Some(data),
            ProgressMsg::Done(Ok(()))  => None,
            ProgressMsg::Done(Err(e))  => {
                error!("error receiving bytes: {}", e);
                None
            }
        }
    }
}


