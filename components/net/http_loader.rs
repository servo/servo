/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, NetworkEvent};
use hsts::{HSTSList, secure_url};
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::hosts::replace_hosts;
use net_traits::{ControlMsg, CookieSource, LoadData, Metadata, LoadConsumer, IncludeSubdomains};
use resource_task::{start_sending_opt, start_sending_sniffed_opt};
use hsts::{HSTSList, secure_url};
use file_loader;

use file_loader;
use ipc_channel::ipc::{self, IpcSender};
use log;
use std::collections::HashSet;
use flate2::read::{DeflateDecoder, GzDecoder};
use hyper::Error as HttpError;
use hyper::client::Request;
use hyper::header::StrictTransportSecurity;
use hyper::header::{AcceptEncoding, Accept, ContentLength, ContentType, Host, Location, qitem, Quality, QualityItem};
use hyper::client::{Request, Response};
use hyper::header::{AcceptEncoding, Accept, ContentLength, ContentType, Host, Location, qitem, StrictTransportSecurity};
use hyper::header::{Quality, QualityItem, Headers};
use hyper::Error as HttpError;
use hyper::method::Method;
use hyper::http::RawStatus;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::net::{Fresh, HttpsConnector, Openssl};
use hyper::status::{StatusCode, StatusClass};
use ipc_channel::ipc::{self, IpcSender};
use log;
use openssl::ssl::{SslContext, SslMethod, SSL_VERIFY_PEER};
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
use util::task::spawn_named;
use util::resource_files::resources_dir_path;
use url::{Url, UrlParser};
use util::opts;
use util::resource_files::resources_dir_path;
use util::task::spawn_named;

use std::borrow::ToOwned;
use std::boxed::FnBox;
use uuid;
use std::fs::File;

pub fn factory(resource_mgr_chan: IpcSender<ControlMsg>,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               hsts_list: Arc<Mutex<HSTSList>>)
               -> Box<FnBox(LoadData, LoadConsumer, Arc<MIMEClassifier>) + Send> {
    box move |load_data, senders, classifier| {
        spawn_named("http_loader".to_owned(),
                    move || load_for_consumer(load_data, senders, classifier, resource_mgr_chan, devtools_chan, hsts_list))
    }
}

fn send_error(url: Url, err: String, start_chan: LoadConsumer) {
    let mut metadata: Metadata = Metadata::default(url);
    metadata.status = None;

    match start_sending_opt(start_chan, metadata) {
        Ok(p) => p.send(Done(Err(err))).unwrap(),
        _ => {}
    };
}

enum ReadResult {
    Payload(Vec<u8>),
    EOF,
}

fn read_block<R: Read>(reader: &mut R) -> Result<ReadResult, ()> {
    let mut buf = vec![0; 1024];

    match reader.read(&mut buf) {
        Ok(len) if len > 0 => {
            unsafe { buf.set_len(len); }
            Ok(ReadResult::Payload(buf))
        }
        Ok(_) => Ok(ReadResult::EOF),
        Err(_) => Err(()),
    }
}

fn request_must_be_secured(hsts_list: &HSTSList, url: &Url) -> bool {
    match url.domain() {
        Some(ref h) => {
            hsts_list.is_host_secure(h)
        },
        _ => false
    }
}

fn inner_url(url: &Url) -> Url {
    let inner_url = url.non_relative_scheme_data().unwrap();
    Url::parse(inner_url).unwrap()
}

fn load_for_consumer(load_data: LoadData,
        start_chan: LoadConsumer,
        classifier: Arc<MIMEClassifier>,
        resource_mgr_chan: IpcSender<ControlMsg>,
        devtools_chan: Option<Sender<DevtoolsControlMsg>>,
        hsts_list: Arc<Mutex<HSTSList>>) {

    match load::<WrappedHttpRequest>(load_data, resource_mgr_chan, devtools_chan, hsts_list, &NetworkHttpRequestFactory) {
        Err(LoadError::UnsupportedScheme(url)) => {
            let s = format!("{} request, but we don't support that scheme", &*url.scheme);
            send_error(url, s, start_chan)
        }
        Err(LoadError::Connection(url, e)) => {
            send_error(url, e, start_chan)
        }
        Err(LoadError::MaxRedirects(url)) => {
            send_error(url, "too many redirects".to_string(), start_chan)
        }
        Err(LoadError::Cors(url, msg)) |
        Err(LoadError::InvalidRedirect(url, msg)) |
        Err(LoadError::Decoding(url, msg)) => {
            send_error(url, msg, start_chan)
        }
        Err(LoadError::Ssl(url, msg)) => {
            info!("ssl validation error {}, '{}'", url.serialize(), msg);

            let mut image = resources_dir_path();
            image.push("badcert.html");
            let load_data = LoadData::new(Url::from_file_path(&*image).unwrap(), None);

            file_loader::factory(load_data, start_chan, classifier)

        }
        Ok((mut response_reader, metadata)) => {
            send_data(&mut response_reader, start_chan, metadata, classifier)
        }
    }
}

pub trait HttpResponse: Read {
    fn headers(&self) -> &Headers;
    fn status(&self) -> StatusCode;
    fn status_raw(&self) -> &RawStatus;
}

struct WrappedHttpResponse {
    response: Response
}

impl Read for WrappedHttpResponse {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.response.read(buf)
    }
}

impl HttpResponse for WrappedHttpResponse {
    fn headers(&self) -> &Headers {
        &self.response.headers
    }

    fn status(&self) -> StatusCode {
        self.response.status
    }

    fn status_raw(&self) -> &RawStatus {
        self.response.status_raw()
    }
}

pub trait HttpRequestFactory {
    type R: HttpRequest;

    fn create(&self, url: Url, method: Method) -> Result<Self::R, LoadError>;
}

struct NetworkHttpRequestFactory;

impl HttpRequestFactory for NetworkHttpRequestFactory {
    type R = WrappedHttpRequest;

    fn create(&self, url: Url, method: Method) -> Result<WrappedHttpRequest, LoadError> {
        let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
        context.set_verify(SSL_VERIFY_PEER, None);
        context.set_CA_file(&resources_dir_path().join("certs")).unwrap();

        let connector = HttpsConnector::new(Openssl { context: Arc::new(context) });
        let connection = Request::with_connector(method.clone(), url.clone(), &connector);

        let ssl_err_string = "Some(OpenSslErrors([UnknownError { library: \"SSL routines\", \
    function: \"SSL3_GET_SERVER_CERTIFICATE\", \
    reason: \"certificate verify failed\" }]))";

        let request = match connection {
            Ok(req) => req,

            Err(HttpError::Io(ref io_error)) if (
                io_error.kind() == io::ErrorKind::Other &&
                io_error.description() == "Error in OpenSSL" &&
                // FIXME: This incredibly hacky. Make it more robust, and at least test it.
                format!("{:?}", io_error.cause()) == ssl_err_string
            ) => {
                return Err(
                    LoadError::Ssl(
                        url.clone(),
                        format!("ssl error {:?}: {:?} {:?}", io_error.kind(), io_error.description(), io_error.cause())
                    )
                )
            },
            Err(e) => {
                 return Err(LoadError::Connection(url.clone(), e.description().to_string()))
            }
        };

        Ok(WrappedHttpRequest { request: request })
    }
}

pub trait HttpRequest {
    type R: HttpResponse + 'static;

    fn headers_mut(&mut self) -> &mut Headers;
    fn send(self, body: &Option<Vec<u8>>) -> Result<Self::R, LoadError>;
}

struct WrappedHttpRequest {
    request: Request<Fresh>
}

impl HttpRequest for WrappedHttpRequest {
    type R = WrappedHttpResponse;

    fn headers_mut(&mut self) -> &mut Headers {
        self.request.headers_mut()
    }

    fn send(self, body: &Option<Vec<u8>>) -> Result<WrappedHttpResponse, LoadError> {
        let mut request_writer = match self.request.start() {
            Ok(streaming) => streaming,
            Err(e) => return Err(LoadError::Connection(Url::parse("http://example.com").unwrap(), e.description().to_string()))
        };

        if let Some(ref data) = *body {
            match request_writer.write_all(&data) {
                Err(e) => {
                    return Err(LoadError::Connection(Url::parse("http://example.com").unwrap(), e.description().to_string()))
                }
                _ => {}
            }
        }

        let response = match request_writer.send() {
            Ok(w) => w,
            Err(e) => return Err(LoadError::Connection(Url::parse("http://example.com").unwrap(), e.description().to_string()))
        };

        Ok(WrappedHttpResponse { response: response })
    }
}

#[derive(Debug)]
pub enum LoadError {
    UnsupportedScheme(Url),
    Connection(Url, String),
    Cors(Url, String),
    Ssl(Url, String),
    InvalidRedirect(Url, String),
    Decoding(Url, String),
    MaxRedirects(Url)
}

#[inline(always)]
fn set_default_accept_encoding(headers: &mut Headers) {
    if !headers.has::<AcceptEncoding>() {
        headers.set_raw("Accept-Encoding".to_owned(), vec![b"gzip, deflate".to_vec()]);
    }
}

#[inline(always)]
fn set_default_accept(headers: &mut Headers) {
    if !headers.has::<Accept>() {
        let accept = Accept(vec![
                            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                            qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_string()), vec![])),
                            QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), Quality(900u16)),
                            QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(800u16)),
                            ]);
        headers.set(accept);
    }
}

pub fn load<A>(mut load_data: LoadData,
            resource_mgr_chan: IpcSender<ControlMsg>,
            devtools_chan: Option<Sender<DevtoolsControlMsg>>,
            hsts_list: Arc<Mutex<HSTSList>>,
            request_factory: &HttpRequestFactory<R=A>)
    -> Result<(Box<Read>, Metadata), LoadError> where A: HttpRequest + 'static {
    // FIXME: At the time of writing this FIXME, servo didn't have any central
    //        location for configuration. If you're reading this and such a
    //        repository DOES exist, please update this constant to use it.
    let max_redirects = 50;
    let mut iters = 0;
    // URL of the document being loaded, as seen by all the higher-level code.
    let mut doc_url = load_data.url.clone();
    // URL that we actually fetch from the network, after applying the replacements
    // specified in the hosts file.
    let mut url = replace_hosts(&load_data.url);
    let mut redirected_to = HashSet::new();

    // If the URL is a view-source scheme then the scheme data contains the
    // real URL that should be used for which the source is to be viewed.
    // Change our existing URL to that and keep note that we are viewing
    // the source rather than rendering the contents of the URL.
    let viewing_source = url.scheme == "view-source";
    if viewing_source {
        url = inner_url(&load_data.url);
        doc_url = url.clone();
    }

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if &*url.scheme != "https" && request_must_be_secured(&hsts_list.lock().unwrap(), &url) {
            info!("{} is in the strict transport security list, requesting secure host", url);
            url = secure_url(&url);
        }

        if iters > max_redirects {
            return Err(LoadError::MaxRedirects(url));
        }

        if &*url.scheme != "http" && &*url.scheme != "https" {
            return Err(LoadError::UnsupportedScheme(url));
        }

        info!("requesting {}", url.serialize());

        // Ensure that the host header is set from the original url
        let host = Host {
            hostname: doc_url.serialize_host().unwrap(),
            port: doc_url.port_or_default()
        };

        // Avoid automatically preserving request headers when redirects occur.
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=401564 and
        // https://bugzilla.mozilla.org/show_bug.cgi?id=216828 .
        // Only preserve ones which have been explicitly marked as such.
        let mut request_headers = if iters == 1 {
            let mut combined_headers = load_data.headers.clone();
            combined_headers.extend(load_data.preserved_headers.iter());
            combined_headers
        } else {
            load_data.preserved_headers.clone()
        };

        request_headers.set(host);

        set_default_accept(&mut request_headers);
        set_default_accept_encoding(&mut request_headers);

        // --- Fetch cookies
        let (tx, rx) = ipc::channel().unwrap();
        resource_mgr_chan.send(ControlMsg::GetCookiesForUrl(doc_url.clone(),
                                                            tx,
                                                            CookieSource::HTTP)).unwrap();
        if let Some(cookie_list) = rx.recv().unwrap() {
            let mut v = Vec::new();
            v.push(cookie_list.into_bytes());
            request_headers.set_raw("Cookie".to_owned(), v);
        }

        // --- Send the request
        let mut req = try!(request_factory.create(url.clone(), load_data.method.clone()));
        *req.headers_mut() = request_headers;

        if log_enabled!(log::LogLevel::Info) {
            info!("{}", load_data.method);
            for header in req.headers_mut().iter() {
                info!(" - {}", header);
            }
            info!("{:?}", load_data.data);
        }

        // TODO: Avoid automatically sending request body if a redirect has occurred.
        if let Some(ref data) = load_data.data {
            req.headers_mut().set(ContentLength(data.len() as u64));
        }

        let response = try!(req.send(&load_data.data));

        // --- Tell devtools we've made a request
        // Send an HttpRequest message to devtools with a unique request_id
        // TODO: Do this only if load_data has some pipeline_id, and send the pipeline_id in the message
        let request_id = uuid::Uuid::new_v4().to_simple_string();
        if let Some(ref chan) = devtools_chan {
            let net_event = NetworkEvent::HttpRequest(load_data.url.clone(),
                                                      load_data.method.clone(),
                                                      load_data.headers.clone(),
                                                      load_data.data.clone());
            chan.send(DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::NetworkEvent(request_id.clone(),
                                                             net_event))).unwrap();
        }

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {}, headers:", response.status());
        if log_enabled!(log::LogLevel::Info) {
            for header in response.headers().iter() {
                info!(" - {}", header);
            }
        }

        // --- Update the resource manager that we've gotten a cookie
        if let Some(cookies) = response.headers().get_raw("set-cookie") {
            for cookie in cookies.iter() {
                if let Ok(cookies) = String::from_utf8(cookie.clone()) {
                    resource_mgr_chan.send(ControlMsg::SetCookiesForUrl(doc_url.clone(),
                                                                        cookies,
                                                                        CookieSource::HTTP)).unwrap();
                }
            }
        }

        if url.scheme == "https" {
            if let Some(header) = response.headers().get::<StrictTransportSecurity>() {
                if let Some(host) = url.domain() {
                    info!("adding host {} to the strict transport security list", host);
                    info!("- max-age {}", header.max_age);

                    let include_subdomains = if header.include_subdomains {
                        info!("- includeSubdomains");
                        IncludeSubdomains::Included
                    } else {
                        IncludeSubdomains::NotIncluded
                    };

                    resource_mgr_chan.send(
                        ControlMsg::SetHSTSEntryForHost(
                            host.to_string(), include_subdomains, header.max_age
                        )
                    ).unwrap();
                }
            }
        }

        // --- Loop if there's a redirect
        if response.status().class() == StatusClass::Redirection {
            match response.headers().get::<Location>() {
                Some(&Location(ref new_url)) => {
                    // CORS (https://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                    match load_data.cors {
                        Some(ref c) => {
                            if c.preflight {
                                return Err(LoadError::Cors(url, "Preflight fetch inconsistent with main fetch".to_string()));
                            } else {
                                // XXXManishearth There are some CORS-related steps here,
                                // but they don't seem necessary until credentials are implemented
                            }
                        }
                        _ => {}
                    }
                    let new_doc_url = match UrlParser::new().base_url(&doc_url).parse(&new_url) {
                        Ok(u) => u,
                        Err(e) => {
                            return Err(LoadError::InvalidRedirect(doc_url, e.to_string()));
                        }
                    };
                    info!("redirecting to {}", new_doc_url);
                    url = replace_hosts(&new_doc_url);
                    doc_url = new_doc_url;

                    // According to https://tools.ietf.org/html/rfc7231#section-6.4.2,
                    // historically UAs have rewritten POST->GET on 301 and 302 responses.
                    if load_data.method == Method::Post &&
                        (response.status() == StatusCode::MovedPermanently ||
                         response.status() == StatusCode::Found) {
                        load_data.method = Method::Get;
                    }

                    if redirected_to.contains(&url) {
                        return Err(LoadError::InvalidRedirect(doc_url, "redirect loop".to_string()));
                    }

                    redirected_to.insert(doc_url.clone());
                    continue;
                }
                None => ()
            }
        }

        let mut adjusted_headers = response.headers().clone();

        if viewing_source {
            adjusted_headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));
        }
        let mut metadata: Metadata = Metadata::default(doc_url.clone());

        metadata.set_content_type(match adjusted_headers.get() {
            Some(&ContentType(ref mime)) => Some(mime),
            None => None
        });
        metadata.headers = Some(adjusted_headers);
        metadata.status = Some(response.status_raw().clone());

        let mut encoding_str: Option<String> = None;

        //TODO: This is now in hyper, just need to implement
        //FIXME: Implement Content-Encoding Header https://github.com/hyperium/hyper/issues/391
        if let Some(encodings) = response.headers().get_raw("content-encoding") {
            for encoding in encodings.iter() {
                if let Ok(encodings) = String::from_utf8(encoding.clone()) {
                    if encodings == "gzip" || encodings == "deflate" {
                        encoding_str = Some(encodings);
                        break;
                    }
                }
            }
        }

        // --- Tell devtools that we got a response
        // Send an HttpResponse message to devtools with the corresponding request_id
        // TODO: Send this message only if load_data has a pipeline_id that is not None
        if let Some(ref chan) = devtools_chan {
            let net_event_response =
                NetworkEvent::HttpResponse(metadata.headers.clone(),
                                           metadata.status.clone(),
                                           None);
            chan.send(DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::NetworkEvent(request_id,
                                                             net_event_response))).unwrap();
        }

        // --- Stream the results depending on the encoding type.
        match encoding_str {
            Some(encoding) => {
                if encoding == "gzip" {
                    let result = GzDecoder::new(response);
                    match result {
                        Ok(response_decoding) => {
                            return Ok((Box::new(response_decoding), metadata));
                        }
                        Err(err) => {
                            return Err(LoadError::Decoding(metadata.final_url, err.to_string()));
                        }
                    }
                } else if encoding == "deflate" {
                    let response_decoding = DeflateDecoder::new(response);
                    return Ok((Box::new(response_decoding), metadata));
                }
            },
            None => {
                return Ok((Box::new(response), metadata));
            }
        }
    }
}

fn send_data<R: Read>(reader: &mut R,
                      start_chan: LoadConsumer,
                      metadata: Metadata,
                      classifier: Arc<MIMEClassifier>) {
    let (progress_chan, mut chunk) = {
        let buf = match read_block(reader) {
            Ok(ReadResult::Payload(buf)) => buf,
            _ => vec!(),
        };
        let p = match start_sending_sniffed_opt(start_chan, metadata, classifier, &buf) {
            Ok(p) => p,
            _ => return
        };
        (p, buf)
    };

    loop {
        if progress_chan.send(Payload(chunk)).is_err() {
            // The send errors when the receiver is out of scope,
            // which will happen if the fetch has timed out (or has been aborted)
            // so we don't need to continue with the loading of the file here.
            return;
        }

        chunk = match read_block(reader) {
            Ok(ReadResult::Payload(buf)) => buf,
            Ok(ReadResult::EOF) | Err(_) => break,
        };
    }

    let _ = progress_chan.send(Done(Ok(())));
}
