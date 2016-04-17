/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use brotli::Decompressor;
use cookie;
use cookie_storage::CookieStorage;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest};
use devtools_traits::{HttpResponse as DevtoolsHttpResponse, NetworkEvent};
use file_loader;
use flate2::read::{DeflateDecoder, GzDecoder};
use hsts::{HstsEntry, HstsList, secure_url};
use hyper::Error as HttpError;
use hyper::client::{Pool, Request, Response};
use hyper::header::{Accept, AcceptEncoding, ContentLength, ContentType, Host};
use hyper::header::{Authorization, Basic};
use hyper::header::{ContentEncoding, Encoding, Header, Headers, Quality, QualityItem};
use hyper::header::{Location, SetCookie, StrictTransportSecurity, UserAgent, qitem};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::net::{Fresh, HttpsConnector, Openssl};
use hyper::status::{StatusClass, StatusCode};
use log;
use mime_classifier::MIMEClassifier;
use msg::constellation_msg::{PipelineId};
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::hosts::replace_hosts;
use net_traits::response::HttpsState;
use net_traits::{CookieSource, IncludeSubdomains, LoadConsumer, LoadContext, LoadData, Metadata};
use openssl::ssl::error::{SslError, OpensslError};
use openssl::ssl::{SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3, SSL_VERIFY_PEER, SslContext, SslMethod};
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt, AuthCacheEntry};
use std::borrow::ToOwned;
use std::boxed::FnBox;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Read, Write};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use time;
use time::Tm;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use tinyfiledialogs;
use url::Url;
use util::prefs;
use util::resource_files::resources_dir_path;
use util::thread::spawn_named;
use uuid;

pub type Connector = HttpsConnector<Openssl>;

// The basic logic here is to prefer ciphers with ECDSA certificates, Forward
// Secrecy, AES GCM ciphers, AES ciphers, and finally 3DES ciphers.
// A complete discussion of the issues involved in TLS configuration can be found here:
// https://wiki.mozilla.org/Security/Server_Side_TLS
const DEFAULT_CIPHERS: &'static str = concat!(
    "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:",
    "ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:",
    "DHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-SHA256:",
    "ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA384:",
    "ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA:",
    "ECDHE-RSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:",
    "DHE-RSA-AES256-SHA256:DHE-RSA-AES256-SHA:ECDHE-RSA-DES-CBC3-SHA:",
    "ECDHE-ECDSA-DES-CBC3-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:",
    "AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA"
);

pub fn create_http_connector() -> Arc<Pool<Connector>> {
    let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
    context.set_verify(SSL_VERIFY_PEER, None);
    context.set_CA_file(&resources_dir_path().join("certs")).unwrap();
    context.set_cipher_list(DEFAULT_CIPHERS).unwrap();
    context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3);
    let connector = HttpsConnector::new(Openssl {
        context: Arc::new(context)
    });

    Arc::new(Pool::with_connector(Default::default(), connector))
}

pub fn factory(user_agent: String,
               http_state: HttpState,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               connector: Arc<Pool<Connector>>)
               -> Box<FnBox(LoadData,
                            LoadConsumer,
                            Arc<MIMEClassifier>,
                            CancellationListener) + Send> {
    box move |load_data: LoadData, senders, classifier, cancel_listener| {
        spawn_named(format!("http_loader for {}", load_data.url.serialize()), move || {
            load_for_consumer(load_data,
                              senders,
                              classifier,
                              connector,
                              http_state,
                              devtools_chan,
                              cancel_listener,
                              user_agent)
        })
    }
}

pub enum ReadResult {
    Payload(Vec<u8>),
    EOF,
}

pub fn read_block<R: Read>(reader: &mut R) -> Result<ReadResult, ()> {
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

pub struct HttpState {
    pub hsts_list: Arc<RwLock<HstsList>>,
    pub cookie_jar: Arc<RwLock<CookieStorage>>,
    pub auth_cache: Arc<RwLock<HashMap<Url, AuthCacheEntry>>>,
}

impl HttpState {
    pub fn new() -> HttpState {
        HttpState {
            hsts_list: Arc::new(RwLock::new(HstsList::new())),
            cookie_jar: Arc::new(RwLock::new(CookieStorage::new())),
            auth_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

fn load_for_consumer(load_data: LoadData,
                     start_chan: LoadConsumer,
                     classifier: Arc<MIMEClassifier>,
                     connector: Arc<Pool<Connector>>,
                     http_state: HttpState,
                     devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                     cancel_listener: CancellationListener,
                     user_agent: String) {

    let factory = NetworkHttpRequestFactory {
        connector: connector,
    };

    let ui_provider = TFDProvider;
    let context = load_data.context.clone();
    match load(load_data, &ui_provider, &http_state,
               devtools_chan, &factory,
               user_agent, &cancel_listener) {
        Err(LoadError::UnsupportedScheme(url)) => {
            let s = format!("{} request, but we don't support that scheme", &*url.scheme);
            send_error(url, s, start_chan)
        }
        Err(LoadError::Connection(url, e)) => {
            send_error(url, e, start_chan)
        }
        Err(LoadError::MaxRedirects(url, _)) => {
            send_error(url, "too many redirects".to_owned(), start_chan)
        }
        Err(LoadError::Cors(url, msg)) |
        Err(LoadError::Cancelled(url, msg)) |
        Err(LoadError::InvalidRedirect(url, msg)) |
        Err(LoadError::Decoding(url, msg)) => {
            send_error(url, msg, start_chan)
        }
        Err(LoadError::Ssl(url, msg)) => {
            info!("ssl validation error {}, '{}'", url.serialize(), msg);

            let mut image = resources_dir_path();
            image.push("badcert.html");
            let load_data = LoadData::new(context, Url::from_file_path(&*image).unwrap(), None);

            file_loader::factory(load_data, start_chan, classifier, cancel_listener)
        }
        Err(LoadError::ConnectionAborted(_)) => unreachable!(),
        Ok(mut load_response) => {
            let metadata = load_response.metadata.clone();
            send_data(context, &mut load_response, start_chan, metadata, classifier, &cancel_listener)
        }
    }
}

pub trait HttpResponse: Read {
    fn headers(&self) -> &Headers;
    fn status(&self) -> StatusCode;
    fn status_raw(&self) -> &RawStatus;
    fn http_version(&self) -> String {
        "HTTP/1.1".to_owned()
    }
    fn content_encoding(&self) -> Option<Encoding> {
        let encodings = match self.headers().get::<ContentEncoding>() {
            Some(&ContentEncoding(ref encodings)) => encodings,
            None => return None,
        };
        if encodings.contains(&Encoding::Gzip) {
            Some(Encoding::Gzip)
        } else if encodings.contains(&Encoding::Deflate) {
            Some(Encoding::Deflate)
        } else if encodings.contains(&Encoding::EncodingExt("br".to_owned())) {
            Some(Encoding::EncodingExt("br".to_owned()))
        } else {
            None
        }
    }
}


pub struct WrappedHttpResponse {
    pub response: Response
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

    fn http_version(&self) -> String {
        self.response.version.to_string()
    }
}

pub trait HttpRequestFactory {
    type R: HttpRequest;

    fn create(&self, url: Url, method: Method, headers: Headers) -> Result<Self::R, LoadError>;
}

pub struct NetworkHttpRequestFactory {
    pub connector: Arc<Pool<Connector>>,
}

impl HttpRequestFactory for NetworkHttpRequestFactory {
    type R = WrappedHttpRequest;

    fn create(&self, url: Url, method: Method, headers: Headers)
              -> Result<WrappedHttpRequest, LoadError> {
        let connection = Request::with_connector(method, url.clone(), &*self.connector);

        if let Err(HttpError::Ssl(ref error)) = connection {
            let error: &(Error + Send + 'static) = &**error;
            if let Some(&SslError::OpenSslErrors(ref errors)) = error.downcast_ref::<SslError>() {
                if errors.iter().any(is_cert_verify_error) {
                    return Err(
                        LoadError::Ssl(url, format!("ssl error: {:?} {:?}",
                                                    error.description(),
                                                    error.cause())));
                }
            }
        }

        let mut request = match connection {
            Ok(req) => req,

            Err(e) => {
                 return Err(LoadError::Connection(url, e.description().to_owned()))
            }
        };
        *request.headers_mut() = headers;

        Ok(WrappedHttpRequest { request: request })
    }
}

pub trait HttpRequest {
    type R: HttpResponse + 'static;

    fn send(self, body: &Option<Vec<u8>>) -> Result<Self::R, LoadError>;
}

pub struct WrappedHttpRequest {
    request: Request<Fresh>
}

impl HttpRequest for WrappedHttpRequest {
    type R = WrappedHttpResponse;

    fn send(self, body: &Option<Vec<u8>>) -> Result<WrappedHttpResponse, LoadError> {
        let url = self.request.url.clone();
        let mut request_writer = match self.request.start() {
            Ok(streaming) => streaming,
            Err(e) => return Err(LoadError::Connection(url, e.description().to_owned()))
        };

        if let Some(ref data) = *body {
            if let Err(e) = request_writer.write_all(&data) {
                return Err(LoadError::Connection(url, e.description().to_owned()))
            }
        }

        let response = match request_writer.send() {
            Ok(w) => w,
            Err(HttpError::Io(ref io_error)) if io_error.kind() == io::ErrorKind::ConnectionAborted => {
                return Err(LoadError::ConnectionAborted(io_error.description().to_owned()));
            },
            Err(e) => return Err(LoadError::Connection(url, e.description().to_owned()))
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
    MaxRedirects(Url, u32),  // u32 indicates number of redirects that occurred
    ConnectionAborted(String),
    Cancelled(Url, String),
}

fn set_default_accept_encoding(headers: &mut Headers) {
    if headers.has::<AcceptEncoding>() {
        return
    }

    headers.set(AcceptEncoding(vec![
        qitem(Encoding::Gzip),
        qitem(Encoding::Deflate),
        qitem(Encoding::EncodingExt("br".to_owned()))
    ]));
}

fn set_default_accept(headers: &mut Headers) {
    if !headers.has::<Accept>() {
        let accept = Accept(vec![
                            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                            qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_owned()), vec![])),
                            QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), Quality(900u16)),
                            QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(800u16)),
                            ]);
        headers.set(accept);
    }
}

pub fn set_request_cookies(url: Url, headers: &mut Headers, cookie_jar: &Arc<RwLock<CookieStorage>>) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    if let Some(cookie_list) = cookie_jar.cookies_for_url(&url, CookieSource::HTTP) {
        let mut v = Vec::new();
        v.push(cookie_list.into_bytes());
        headers.set_raw("Cookie".to_owned(), v);
    }
}

fn set_cookie_for_url(cookie_jar: &Arc<RwLock<CookieStorage>>,
                      request: Url,
                      cookie_val: String) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    let source = CookieSource::HTTP;
    let header = Header::parse_header(&[cookie_val.into_bytes()]);

    if let Ok(SetCookie(cookies)) = header {
        for bare_cookie in cookies {
            if let Some(cookie) = cookie::Cookie::new_wrapped(bare_cookie, &request, source) {
                cookie_jar.push(cookie, source);
            }
        }
    }
}

fn set_cookies_from_response(url: Url, response: &HttpResponse, cookie_jar: &Arc<RwLock<CookieStorage>>) {
    if let Some(cookies) = response.headers().get_raw("set-cookie") {
        for cookie in cookies.iter() {
            if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                set_cookie_for_url(&cookie_jar,
                                   url.clone(),
                                   cookie_value);
            }
        }
    }
}

fn update_sts_list_from_response(url: &Url, response: &HttpResponse, hsts_list: &Arc<RwLock<HstsList>>) {
    if url.scheme != "https" {
        return;
    }

    if let Some(header) = response.headers().get::<StrictTransportSecurity>() {
        if let Some(host) = url.domain() {
            let mut hsts_list = hsts_list.write().unwrap();
            let include_subdomains = if header.include_subdomains {
                IncludeSubdomains::Included
            } else {
                IncludeSubdomains::NotIncluded
            };

            if let Some(entry) = HstsEntry::new(host.to_owned(), include_subdomains, Some(header.max_age)) {
                info!("adding host {} to the strict transport security list", host);
                info!("- max-age {}", header.max_age);
                if header.include_subdomains {
                    info!("- includeSubdomains");
                }

                hsts_list.push(entry);
            }
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
            Decoder::Brotli(ref mut d) => d.read(buf),
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
                        Ok(StreamedResponse::new(m, Decoder::Gzip(response_decoding)))
                    }
                    Err(err) => {
                        Err(LoadError::Decoding(m.final_url, err.to_string()))
                    }
                }
            }
            Some(Encoding::Deflate) => {
                let response_decoding = DeflateDecoder::new(response);
                Ok(StreamedResponse::new(m, Decoder::Deflate(response_decoding)))
            }
            Some(Encoding::EncodingExt(ref ext)) if ext == "br" => {
                let response_decoding = Decompressor::new(response);
                Ok(StreamedResponse::new(m, Decoder::Brotli(response_decoding)))
            }
            _ => {
                Ok(StreamedResponse::new(m, Decoder::Plain(response)))
            }
        }
    }
}

enum Decoder<R: Read> {
    Gzip(GzDecoder<R>),
    Deflate(DeflateDecoder<R>),
    Brotli(Decompressor<R>),
    Plain(R)
}

fn send_request_to_devtools(devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                            request_id: String,
                            url: Url,
                            method: Method,
                            headers: Headers,
                            body: Option<Vec<u8>>,
                            pipeline_id: PipelineId, now: Tm) {

    if let Some(ref chan) = devtools_chan {
        let request = DevtoolsHttpRequest {
            url: url, method: method, headers: headers, body: body, pipeline_id: pipeline_id, startedDateTime: now };
        let net_event = NetworkEvent::HttpRequest(request);

        let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event);
        chan.send(DevtoolsControlMsg::FromChrome(msg)).unwrap();
    }
}

fn send_response_to_devtools(devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                             request_id: String,
                             headers: Option<Headers>,
                             status: Option<RawStatus>,
                             pipeline_id: PipelineId) {
    if let Some(ref chan) = devtools_chan {
        let response = DevtoolsHttpResponse { headers: headers, status: status, body: None, pipeline_id: pipeline_id };
        let net_event_response = NetworkEvent::HttpResponse(response);

        let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event_response);
        chan.send(DevtoolsControlMsg::FromChrome(msg)).unwrap();
    }
}

fn request_must_be_secured(url: &Url, hsts_list: &Arc<RwLock<HstsList>>) -> bool {
    match url.domain() {
        Some(domain) => hsts_list.read().unwrap().is_host_secure(domain),
        None => false
    }
}

pub fn modify_request_headers(headers: &mut Headers,
                              url: &Url,
                              user_agent: &str,
                              cookie_jar: &Arc<RwLock<CookieStorage>>,
                              auth_cache: &Arc<RwLock<HashMap<Url, AuthCacheEntry>>>,
                              load_data: &LoadData) {
    // Ensure that the host header is set from the original url
    let host = Host {
        hostname: url.serialize_host().unwrap(),
        port: url.port_or_default()
    };
    headers.set(host);

    // If the user-agent has not already been set, then use the
    // browser's default user-agent or the user-agent override
    // from the command line. If the user-agent is set, don't
    // modify it, as setting of the user-agent by the user is
    // allowed.
    // https://fetch.spec.whatwg.org/#concept-http-network-or-cache-fetch step 8
    if !headers.has::<UserAgent>() {
        headers.set(UserAgent(user_agent.to_owned()));
    }

    set_default_accept(headers);
    set_default_accept_encoding(headers);

    // https://fetch.spec.whatwg.org/#concept-http-network-or-cache-fetch step 11
    if load_data.credentials_flag {
        set_request_cookies(url.clone(), headers, cookie_jar);

        // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch step 12
        set_auth_header(headers, url, auth_cache);
    }
}

fn set_auth_header(headers: &mut Headers,
                   url: &Url,
                   auth_cache: &Arc<RwLock<HashMap<Url, AuthCacheEntry>>>) {

    if !headers.has::<Authorization<Basic>>() {
        if let Some(auth) = auth_from_url(url) {
            headers.set(auth);
        } else {
            if let Some(ref auth_entry) = auth_cache.read().unwrap().get(url) {
                auth_from_entry(&auth_entry, headers);
            }
        }
    }
}

fn auth_from_entry(auth_entry: &AuthCacheEntry, headers: &mut Headers) {
    let user_name = auth_entry.user_name.clone();
    let password  = Some(auth_entry.password.clone());

    headers.set(Authorization(Basic { username: user_name, password: password }));
}

fn auth_from_url(doc_url: &Url) -> Option<Authorization<Basic>> {
    match doc_url.username() {
        Some(username) if username != "" => {
            Some(Authorization(Basic {
                username: username.to_owned(),
                password: Some(doc_url.password().unwrap_or("").to_owned())
            }))
        },
        _ => None
    }
}

pub fn process_response_headers(response: &HttpResponse,
                                url: &Url,
                                cookie_jar: &Arc<RwLock<CookieStorage>>,
                                hsts_list: &Arc<RwLock<HstsList>>,
                                load_data: &LoadData) {
    info!("got HTTP response {}, headers:", response.status());
    if log_enabled!(log::LogLevel::Info) {
        for header in response.headers().iter() {
            info!(" - {}", header);
        }
    }

    // https://fetch.spec.whatwg.org/#concept-http-network-fetch step 9
    if load_data.credentials_flag {
        set_cookies_from_response(url.clone(), response, cookie_jar);
    }
    update_sts_list_from_response(url, response, hsts_list);
}

pub fn obtain_response<A>(request_factory: &HttpRequestFactory<R=A>,
                          url: &Url,
                          method: &Method,
                          request_headers: &Headers,
                          cancel_listener: &CancellationListener,
                          data: &Option<Vec<u8>>,
                          load_data_method: &Method,
                          pipeline_id: &Option<PipelineId>,
                          iters: u32,
                          devtools_chan: &Option<Sender<DevtoolsControlMsg>>,
                          request_id: &str)
                          -> Result<A::R, LoadError> where A: HttpRequest + 'static  {

    let null_data = None;
    let response;
    let connection_url = replace_hosts(&url);

    // loop trying connections in connection pool
    // they may have grown stale (disconnected), in which case we'll get
    // a ConnectionAborted error. this loop tries again with a new
    // connection.
    loop {
        let mut headers = request_headers.clone();

        // Avoid automatically sending request body if a redirect has occurred.
        //
        // TODO - This is the wrong behaviour according to the RFC. However, I'm not
        // sure how much "correctness" vs. real-world is important in this case.
        //
        // https://tools.ietf.org/html/rfc7231#section-6.4
        let is_redirected_request = iters != 1;
        let request_body;
        match data {
            &Some(ref d) if !is_redirected_request => {
                headers.set(ContentLength(d.len() as u64));
                request_body = data;
            }
            _ => {
                if *load_data_method != Method::Get && *load_data_method != Method::Head {
                    headers.set(ContentLength(0))
                }
                request_body = &null_data;
            }
        }

        if log_enabled!(log::LogLevel::Info) {
            info!("{}", method);
            for header in headers.iter() {
                info!(" - {}", header);
            }
            info!("{:?}", data);
        }

        let req = try!(request_factory.create(connection_url.clone(), method.clone(),
                                              headers.clone()));

        if cancel_listener.is_cancelled() {
            return Err(LoadError::Cancelled(connection_url.clone(), "load cancelled".to_owned()));
        }

        let maybe_response = req.send(request_body);

        if let Some(pipeline_id) = *pipeline_id {
            send_request_to_devtools(
                devtools_chan.clone(), request_id.clone().into(),
                url.clone(), method.clone(), headers,
                request_body.clone(), pipeline_id, time::now()
            );
        }

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

    Ok(response)
}

pub trait UIProvider {
    fn input_username_and_password(&self) -> (Option<String>, Option<String>);
}

impl UIProvider for TFDProvider {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn input_username_and_password(&self) -> (Option<String>, Option<String>) {
        (tinyfiledialogs::input_box("Enter username", "Username:", ""),
        tinyfiledialogs::input_box("Enter password", "Password:", ""))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn input_username_and_password(&self) -> (Option<String>, Option<String>) {
        (None, None)
    }
}

struct TFDProvider;

pub fn load<A, B>(load_data: LoadData,
               ui_provider: &B,
               http_state: &HttpState,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               request_factory: &HttpRequestFactory<R=A>,
               user_agent: String,
               cancel_listener: &CancellationListener)
               -> Result<StreamedResponse<A::R>, LoadError> where A: HttpRequest + 'static, B: UIProvider {
    let max_redirects = prefs::get_pref("network.http.redirection-limit").as_i64().unwrap() as u32;
    let mut iters = 0;
    // URL of the document being loaded, as seen by all the higher-level code.
    let mut doc_url = load_data.url.clone();
    let mut redirected_to = HashSet::new();
    let mut method = load_data.method.clone();

    let mut new_auth_header: Option<Authorization<Basic>> = None;

    if cancel_listener.is_cancelled() {
        return Err(LoadError::Cancelled(doc_url, "load cancelled".to_owned()));
    }

    // If the URL is a view-source scheme then the scheme data contains the
    // real URL that should be used for which the source is to be viewed.
    // Change our existing URL to that and keep note that we are viewing
    // the source rather than rendering the contents of the URL.
    let viewing_source = doc_url.scheme == "view-source";
    if viewing_source {
        doc_url = inner_url(&load_data.url);
    }

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if &*doc_url.scheme == "http" && request_must_be_secured(&doc_url, &http_state.hsts_list) {
            info!("{} is in the strict transport security list, requesting secure host", doc_url);
            doc_url = secure_url(&doc_url);
        }

        if iters > max_redirects {
            return Err(LoadError::MaxRedirects(doc_url, iters - 1));
        }

        if &*doc_url.scheme != "http" && &*doc_url.scheme != "https" {
            return Err(LoadError::UnsupportedScheme(doc_url));
        }

        if cancel_listener.is_cancelled() {
            return Err(LoadError::Cancelled(doc_url, "load cancelled".to_owned()));
        }

        info!("requesting {}", doc_url.serialize());

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

        let request_id = uuid::Uuid::new_v4().simple().to_string();

        modify_request_headers(&mut request_headers, &doc_url,
                               &user_agent, &http_state.cookie_jar,
                               &http_state.auth_cache, &load_data);

        //if there is a new auth header then set the request headers with it
        if let Some(ref auth_header) = new_auth_header {
            request_headers.set(auth_header.clone());
        }

        let response = try!(obtain_response(request_factory, &doc_url, &method, &request_headers,
                                            &cancel_listener, &load_data.data, &load_data.method,
                                            &load_data.pipeline_id, iters, &devtools_chan, &request_id));

        process_response_headers(&response, &doc_url, &http_state.cookie_jar, &http_state.hsts_list, &load_data);

        //if response status is unauthorized then prompt user for username and password
        if response.status() == StatusCode::Unauthorized {
            let (username_option, password_option) = ui_provider.input_username_and_password();

            match username_option {
                Some(name) => {
                    new_auth_header =  Some(Authorization(Basic { username: name, password: password_option }));
                    continue;
                },
                None => {},
            }
        }

        new_auth_header = None;

        if let Some(auth_header) = request_headers.get::<Authorization<Basic>>() {
            if response.status().class() == StatusClass::Success {
                let auth_entry = AuthCacheEntry {
                    user_name: auth_header.username.to_owned(),
                    password: auth_header.password.to_owned().unwrap(),
                };

                http_state.auth_cache.write().unwrap().insert(doc_url.clone(), auth_entry);
            }
        }

        // --- Loop if there's a redirect
        if response.status().class() == StatusClass::Redirection {
            if let Some(&Location(ref new_url)) = response.headers().get::<Location>() {
                // CORS (https://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                if let Some(ref c) = load_data.cors {
                    if c.preflight {
                        return Err(
                            LoadError::Cors(
                                doc_url,
                                "Preflight fetch inconsistent with main fetch".to_owned()));
                    } else {
                        // XXXManishearth There are some CORS-related steps here,
                        // but they don't seem necessary until credentials are implemented
                    }
                }

                let new_doc_url = match doc_url.join(&new_url) {
                    Ok(u) => u,
                    Err(e) => {
                        return Err(LoadError::InvalidRedirect(doc_url, e.to_string()));
                    }
                };

                // According to https://tools.ietf.org/html/rfc7231#section-6.4.2,
                // historically UAs have rewritten POST->GET on 301 and 302 responses.
                if method == Method::Post &&
                    (response.status() == StatusCode::MovedPermanently ||
                        response.status() == StatusCode::Found) {
                    method = Method::Get;
                }

                if redirected_to.contains(&new_doc_url) {
                    return Err(LoadError::InvalidRedirect(doc_url, "redirect loop".to_owned()));
                }

                info!("redirecting to {}", new_doc_url);
                doc_url = new_doc_url;

                redirected_to.insert(doc_url.clone());
                continue;
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
        metadata.https_state = if doc_url.scheme == "https" {
            HttpsState::Modern
        } else {
            HttpsState::None
        };

        // --- Tell devtools that we got a response
        // Send an HttpResponse message to devtools with the corresponding request_id
        // TODO: Send this message even when the load fails?
        if let Some(pipeline_id) = load_data.pipeline_id {
                send_response_to_devtools(
                    devtools_chan, request_id,
                    metadata.headers.clone(), metadata.status.clone(),
                    pipeline_id);
         }
        return StreamedResponse::from_http_response(response, metadata)
    }
}

fn send_data<R: Read>(context: LoadContext,
                      reader: &mut R,
                      start_chan: LoadConsumer,
                      metadata: Metadata,
                      classifier: Arc<MIMEClassifier>,
                      cancel_listener: &CancellationListener) {
    let (progress_chan, mut chunk) = {
        let buf = match read_block(reader) {
            Ok(ReadResult::Payload(buf)) => buf,
            _ => vec!(),
        };
        let p = match start_sending_sniffed_opt(start_chan, metadata, classifier, &buf, context) {
            Ok(p) => p,
            _ => return
        };
        (p, buf)
    };

    loop {
        if cancel_listener.is_cancelled() {
            let _ = progress_chan.send(Done(Err("load cancelled".to_owned())));
            return;
        }

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

// FIXME: This incredibly hacky. Make it more robust, and at least test it.
fn is_cert_verify_error(error: &OpensslError) -> bool {
    match error {
        &OpensslError::UnknownError { ref library, ref function, ref reason } => {
            library == "SSL routines" &&
            function == "SSL3_GET_SERVER_CERTIFICATE" &&
            reason == "certificate verify failed"
        }
    }
}
