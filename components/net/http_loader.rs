/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, NetworkEvent};
use file_loader;
use flate2::read::{DeflateDecoder, GzDecoder};
use hsts::secure_url;
use hyper::Error as HttpError;
use hyper::client::{Request, Response, Pool};
use hyper::header::{AcceptEncoding, Accept, ContentLength, ContentType, Host};
use hyper::header::{Location, qitem, StrictTransportSecurity, UserAgent};
use hyper::header::{Quality, QualityItem, Headers, ContentEncoding, Encoding};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::net::{Fresh, HttpsConnector, Openssl};
use hyper::status::{StatusCode, StatusClass};
use ipc_channel::ipc::{self, IpcSender};
use log;
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::hosts::replace_hosts;
use net_traits::{ControlMsg, CookieSource, LoadData, Metadata, LoadConsumer, IncludeSubdomains};
use openssl::ssl::{SslContext, SslMethod, SSL_VERIFY_PEER};
use resource_task::{start_sending_opt, start_sending_sniffed_opt};
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use url::{Url, UrlParser};
use util::resource_files::resources_dir_path;
use util::task::spawn_named;
use uuid;

pub type Connector = HttpsConnector<Openssl>;

pub fn create_http_connector() -> Arc<Pool<Connector>> {
    let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
    context.set_verify(SSL_VERIFY_PEER, None);
    context.set_CA_file(&resources_dir_path().join("certs")).unwrap();
    let connector = HttpsConnector::new(Openssl {
        context: Arc::new(context)
    });

    Arc::new(Pool::with_connector(Default::default(), connector))
}

pub fn factory(resource_mgr_chan: IpcSender<ControlMsg>,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               connector: Arc<Pool<Connector>>)
               -> Box<FnBox(LoadData, LoadConsumer, Arc<MIMEClassifier>, String) + Send> {
    box move |load_data: LoadData, senders, classifier, user_agent| {
        spawn_named(format!("http_loader for {}", load_data.url.serialize()), move || {
            load_for_consumer(load_data, senders, classifier, connector, resource_mgr_chan, devtools_chan, user_agent)
        })
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
            buf.truncate(len);
            Ok(ReadResult::Payload(buf))
        }
        Ok(_) => Ok(ReadResult::EOF),
        Err(_) => Err(()),
    }
}

fn inner_url(url: &Url) -> Url {
    let inner_url = url.non_relative_scheme_data().unwrap();
    Url::parse(inner_url).unwrap()
}

fn load_for_consumer(load_data: LoadData,
                     start_chan: LoadConsumer,
                     classifier: Arc<MIMEClassifier>,
                     connector: Arc<Pool<Connector>>,
                     resource_mgr_chan: IpcSender<ControlMsg>,
                     devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                     user_agent: String) {

    let factory = NetworkHttpRequestFactory {
        connector: connector,
    };
    match load::<WrappedHttpRequest>(load_data, resource_mgr_chan, devtools_chan, &factory, user_agent) {
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
        Err(LoadError::ConnectionAborted(_)) => unreachable!(),
        Ok(mut load_response) => {
            let metadata = load_response.metadata.clone();
            send_data(&mut load_response, start_chan, metadata, classifier)
        }
    }
}

pub trait HttpResponse: Read {
    fn headers(&self) -> &Headers;
    fn status(&self) -> StatusCode;
    fn status_raw(&self) -> &RawStatus;

    fn content_encoding(&self) -> Option<Encoding> {
        self.headers().get::<ContentEncoding>().and_then(|h| {
            match h {
                &ContentEncoding(ref encodings) => {
                    if encodings.contains(&Encoding::Gzip) {
                        Some(Encoding::Gzip)
                    } else if encodings.contains(&Encoding::Deflate) {
                        Some(Encoding::Deflate)
                    } else {
                        // TODO: Is this the correct behaviour?
                        None
                    }
                }
            }
        })
    }
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

struct NetworkHttpRequestFactory {
    connector: Arc<Pool<Connector>>,
}

impl HttpRequestFactory for NetworkHttpRequestFactory {
    type R = WrappedHttpRequest;

    fn create(&self, url: Url, method: Method) -> Result<WrappedHttpRequest, LoadError> {
        let connection = Request::with_connector(method, url.clone(), &*self.connector);

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
                        url,
                        format!("ssl error {:?}: {:?} {:?}",
                                io_error.kind(),
                                io_error.description(),
                                io_error.cause())
                    )
                )
            },
            Err(e) => {
                 return Err(LoadError::Connection(url, e.description().to_string()))
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
        let url = self.request.url.clone();
        let mut request_writer = match self.request.start() {
            Ok(streaming) => streaming,
            Err(e) => return Err(LoadError::Connection(url, e.description().to_string()))
        };

        if let Some(ref data) = *body {
            match request_writer.write_all(&data) {
                Err(e) => {
                    return Err(LoadError::Connection(url, e.description().to_string()))
                }
                _ => {}
            }
        }

        let response = match request_writer.send() {
            Ok(w) => w,
            Err(HttpError::Io(ref io_error)) if io_error.kind() == io::ErrorKind::ConnectionAborted => {
                return Err(LoadError::ConnectionAborted(io_error.description().to_string()));
            },
            Err(e) => return Err(LoadError::Connection(url, e.description().to_string()))
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
    MaxRedirects(Url),
    ConnectionAborted(String),
}

fn set_default_accept_encoding(headers: &mut Headers) {
    if headers.has::<AcceptEncoding>() {
        return
    }

    headers.set_raw("Accept-Encoding".to_owned(), vec![b"gzip, deflate".to_vec()]);
}

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

fn set_request_cookies(url: Url, headers: &mut Headers, resource_mgr_chan: &IpcSender<ControlMsg>) {
    let (tx, rx) = ipc::channel().unwrap();
    resource_mgr_chan.send(ControlMsg::GetCookiesForUrl(url, tx, CookieSource::HTTP)).unwrap();
    if let Some(cookie_list) = rx.recv().unwrap() {
        let mut v = Vec::new();
        v.push(cookie_list.into_bytes());
        headers.set_raw("Cookie".to_owned(), v);
    }
}

fn set_cookies_from_response(url: Url, response: &HttpResponse, resource_mgr_chan: &IpcSender<ControlMsg>) {
    if let Some(cookies) = response.headers().get_raw("set-cookie") {
        for cookie in cookies.iter() {
            if let Ok(cookies) = String::from_utf8(cookie.clone()) {
                resource_mgr_chan.send(ControlMsg::SetCookiesForUrl(url.clone(),
                                                                    cookies,
                                                                    CookieSource::HTTP)).unwrap();
            }
        }
    }
}

fn request_must_be_secured(url: &Url, resource_mgr_chan: &IpcSender<ControlMsg>) -> bool {
    let (tx, rx) = ipc::channel().unwrap();
    resource_mgr_chan.send(
        ControlMsg::GetHostMustBeSecured(url.domain().unwrap().to_string(), tx)
    ).unwrap();

    rx.recv().unwrap()
}

fn update_sts_list_from_response(url: &Url, response: &HttpResponse, resource_mgr_chan: &IpcSender<ControlMsg>) {
    if url.scheme != "https" {
        return;
    }

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

            let msg = ControlMsg::SetHSTSEntryForHost(
                host.to_string(),
                include_subdomains,
                header.max_age
            );

            resource_mgr_chan.send(msg).unwrap();
        }
    }
}

pub struct StreamedResponse<R: HttpResponse> {
    decoder: Decoder<R>,
    pub metadata: Metadata
}


impl<R: HttpResponse> Read for StreamedResponse<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.decoder {
            Decoder::Gzip(ref mut d) => d.read(buf),
            Decoder::Deflate(ref mut d) => d.read(buf),
            Decoder::Plain(ref mut d) => d.read(buf)
        }
    }
}

impl<R: HttpResponse> StreamedResponse<R> {
    fn new(m: Metadata, d: Decoder<R>) -> StreamedResponse<R> {
        StreamedResponse { metadata: m, decoder: d }
    }

    fn from_http_response(response: R, m: Metadata) -> Result<StreamedResponse<R>, LoadError> {
        match response.content_encoding() {
            Some(Encoding::Gzip) => {
                let result = GzDecoder::new(response);
                match result {
                    Ok(response_decoding) => {
                        return Ok(StreamedResponse::new(m, Decoder::Gzip(response_decoding)));
                    }
                    Err(err) => {
                        return Err(LoadError::Decoding(m.final_url, err.to_string()));
                    }
                }
            }
            Some(Encoding::Deflate) => {
                let response_decoding = DeflateDecoder::new(response);
                return Ok(StreamedResponse::new(m, Decoder::Deflate(response_decoding)));
            }
            _ => {
                return Ok(StreamedResponse::new(m, Decoder::Plain(response)));
            }
        }
    }
}

enum Decoder<R: Read> {
    Gzip(GzDecoder<R>),
    Deflate(DeflateDecoder<R>),
    Plain(R)
}

fn send_request_to_devtools(devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                            request_id: String,
                            url: Url,
                            method: Method,
                            headers: Headers,
                            body: Option<Vec<u8>>) {

    if let Some(ref chan) = devtools_chan {
        let net_event = NetworkEvent::HttpRequest(url, method, headers, body);

        let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event);
        chan.send(DevtoolsControlMsg::FromChrome(msg)).unwrap();
    }
}

fn send_response_to_devtools(devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                             request_id: String,
                             headers: Option<Headers>,
                             status: Option<RawStatus>) {
    if let Some(ref chan) = devtools_chan {
        let net_event_response = NetworkEvent::HttpResponse(headers, status, None);

        let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event_response);
        chan.send(DevtoolsControlMsg::FromChrome(msg)).unwrap();
    }
}

pub fn load<A>(load_data: LoadData,
               resource_mgr_chan: IpcSender<ControlMsg>,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               request_factory: &HttpRequestFactory<R=A>,
               user_agent: String)
               -> Result<StreamedResponse<A::R>, LoadError> where A: HttpRequest + 'static {
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
    let mut method = load_data.method.clone();

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

        if &*url.scheme == "http" && request_must_be_secured(&url, &resource_mgr_chan) {
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

        request_headers.set(UserAgent(user_agent.clone()));

        set_default_accept(&mut request_headers);
        set_default_accept_encoding(&mut request_headers);
        set_request_cookies(doc_url.clone(), &mut request_headers, &resource_mgr_chan);

        let request_id = uuid::Uuid::new_v4().to_simple_string();

        let response;

        // loop trying connections in connection pool
        // they may have grown stale (disconnected), in which case we'll get
        // a ConnectionAborted error. this loop tries again with a new
        // connection.
        loop {
            let mut req = try!(request_factory.create(url.clone(), method.clone()));
            *req.headers_mut() = request_headers.clone();

            if log_enabled!(log::LogLevel::Info) {
                info!("{}", method);
                for header in req.headers_mut().iter() {
                    info!(" - {}", header);
                }
                info!("{:?}", load_data.data);
            }

            // Avoid automatically sending request body if a redirect has occurred.
            //
            // TODO - This is the wrong behaviour according to the RFC. However, I'm not
            // sure how much "correctness" vs. real-world is important in this case.
            //
            // https://tools.ietf.org/html/rfc7231#section-6.4
            let is_redirected_request = iters != 1;
            let maybe_response = match load_data.data {
                Some(ref data) if !is_redirected_request => {
                    req.headers_mut().set(ContentLength(data.len() as u64));

                    // TODO: Do this only if load_data has some pipeline_id, and send the pipeline_id
                    // in the message
                    send_request_to_devtools(
                        devtools_chan.clone(), request_id.clone(), url.clone(),
                        method.clone(), load_data.headers.clone(),
                        load_data.data.clone()
                    );

                    req.send(&load_data.data)
                }
                _ => {
                    if load_data.method != Method::Get && load_data.method != Method::Head {
                        req.headers_mut().set(ContentLength(0))
                    }

                    send_request_to_devtools(
                        devtools_chan.clone(), request_id.clone(), url.clone(),
                        method.clone(), load_data.headers.clone(),
                        None
                    );

                    req.send(&None)
                }
            };

            response = match maybe_response {
                Ok(r) => r,
                Err(LoadError::ConnectionAborted(reason)) => {
                    debug!("connection aborted ({:?}), possibly stale, trying new connection", reason);
                    continue;
                }
                Err(e) => return Err(e),
            };

            // if no ConnectionAborted, break the loop
            break;
        }

        info!("got HTTP response {}, headers:", response.status());
        if log_enabled!(log::LogLevel::Info) {
            for header in response.headers().iter() {
                info!(" - {}", header);
            }
        }

        set_cookies_from_response(doc_url.clone(), &response, &resource_mgr_chan);
        update_sts_list_from_response(&url, &response, &resource_mgr_chan);

        // --- Loop if there's a redirect
        if response.status().class() == StatusClass::Redirection {
            match response.headers().get::<Location>() {
                Some(&Location(ref new_url)) => {
                    // CORS (https://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                    match load_data.cors {
                        Some(ref c) => {
                            if c.preflight {
                                return Err(
                                    LoadError::Cors(
                                        url,
                                        "Preflight fetch inconsistent with main fetch".to_string()));
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
                    if method == Method::Post &&
                        (response.status() == StatusCode::MovedPermanently ||
                         response.status() == StatusCode::Found) {
                        method = Method::Get;
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

        // --- Tell devtools that we got a response
        // Send an HttpResponse message to devtools with the corresponding request_id
        // TODO: Send this message only if load_data has a pipeline_id that is not None
        // TODO: Send this message even when the load fails?
        send_response_to_devtools(
            devtools_chan, request_id,
            metadata.headers.clone(), metadata.status.clone()
        );

        return StreamedResponse::from_http_response(response, metadata)
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
