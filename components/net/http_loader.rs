/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use brotli::Decompressor;
use connector::Connector;
use content_blocker_parser::{LoadType, Reaction, Request as CBRequest, ResourceType};
use content_blocker_parser::{RuleList, process_rules_for_request};
use cookie;
use cookie_storage::CookieStorage;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest};
use devtools_traits::{HttpResponse as DevtoolsHttpResponse, NetworkEvent};
use flate2::read::{DeflateDecoder, GzDecoder};
use hsts::{HstsEntry, HstsList, secure_url};
use hyper::Error as HttpError;
use hyper::LanguageTag;
use hyper::client::{Pool, Request, Response};
use hyper::header::{Accept, AcceptEncoding, ContentEncoding, ContentLength, ContentType, Host, Referer};
use hyper::header::{AcceptLanguage, Authorization, Basic};
use hyper::header::{Encoding, Header, Headers, Quality, QualityItem};
use hyper::header::{Location, SetCookie, StrictTransportSecurity, UserAgent, qitem};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper::net::Fresh;
use hyper::status::{StatusClass, StatusCode};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use log;
use mime_classifier::MimeClassifier;
use msg::constellation_msg::PipelineId;
use net_traits::{CookieSource, IncludeSubdomains, LoadConsumer, LoadContext, LoadData};
use net_traits::{CustomResponse, CustomResponseMediator, Metadata, NetworkError, ReferrerPolicy};
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::hosts::replace_hosts;
use net_traits::response::HttpsState;
use openssl;
use openssl::ssl::error::{OpensslError, SslError};
use profile_traits::time::{ProfilerCategory, ProfilerChan, TimerMetadata, profile};
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType};
use resource_thread::{AuthCache, AuthCacheEntry, CancellationListener, send_error, start_sending_sniffed_opt};
use std::borrow::{Cow, ToOwned};
use std::boxed::FnBox;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io::{self, Cursor, Read, Write};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use time;
use time::Tm;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
use tinyfiledialogs;
use url::{Position, Url, Origin};
use util::prefs::PREFS;
use util::thread::spawn_named;
use uuid;

pub fn factory(user_agent: Cow<'static, str>,
               http_state: HttpState,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               profiler_chan: ProfilerChan,
               swmanager_chan: Option<IpcSender<CustomResponseMediator>>,
               connector: Arc<Pool<Connector>>)
               -> Box<FnBox(LoadData,
                            LoadConsumer,
                            Arc<MimeClassifier>,
                            CancellationListener) + Send> {
    box move |load_data: LoadData, senders, classifier, cancel_listener| {
        spawn_named(format!("http_loader for {}", load_data.url), move || {
            let metadata = TimerMetadata {
                url: load_data.url.as_str().into(),
                iframe: TimerMetadataFrameType::RootWindow,
                incremental: TimerMetadataReflowType::FirstReflow,
            };
            profile(ProfilerCategory::NetHTTPRequestResponse, Some(metadata), profiler_chan, || {
                load_for_consumer(load_data,
                                  senders,
                                  classifier,
                                  connector,
                                  http_state,
                                  devtools_chan,
                                  swmanager_chan,
                                  cancel_listener,
                                  user_agent)
            })
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

pub struct HttpState {
    pub hsts_list: Arc<RwLock<HstsList>>,
    pub cookie_jar: Arc<RwLock<CookieStorage>>,
    pub auth_cache: Arc<RwLock<AuthCache>>,
    pub blocked_content: Arc<Option<RuleList>>,
}

impl HttpState {
    pub fn new() -> HttpState {
        HttpState {
            hsts_list: Arc::new(RwLock::new(HstsList::new())),
            cookie_jar: Arc::new(RwLock::new(CookieStorage::new())),
            auth_cache: Arc::new(RwLock::new(AuthCache::new())),
            blocked_content: Arc::new(None),
        }
    }
}

fn precise_time_ms() -> u64 {
    time::precise_time_ns() / (1000 * 1000)
}

fn load_for_consumer(load_data: LoadData,
                     start_chan: LoadConsumer,
                     classifier: Arc<MimeClassifier>,
                     connector: Arc<Pool<Connector>>,
                     http_state: HttpState,
                     devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                     swmanager_chan: Option<IpcSender<CustomResponseMediator>>,
                     cancel_listener: CancellationListener,
                     user_agent: Cow<'static, str>) {
    let factory = NetworkHttpRequestFactory {
        connector: connector,
    };

    let ui_provider = TFDProvider;
    match load(&load_data, &ui_provider, &http_state,
               devtools_chan, &factory,
               user_agent, &cancel_listener, swmanager_chan) {
        Err(error) => {
            match error.error {
                LoadErrorType::ConnectionAborted { .. } => unreachable!(),
                LoadErrorType::Ssl { reason } => send_error(error.url.clone(),
                                                        NetworkError::SslValidation(error.url, reason),
                                                        start_chan),
                LoadErrorType::Cancelled => send_error(error.url, NetworkError::LoadCancelled, start_chan),
                _ => send_error(error.url, NetworkError::Internal(error.error.description().to_owned()), start_chan)
            }
        }
        Ok(mut load_response) => {
            let metadata = load_response.metadata.clone();
            send_data(load_data.context, &mut load_response, start_chan, metadata, classifier, &cancel_listener)
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

pub struct ReadableCustomResponse {
    headers: Headers,
    raw_status: RawStatus,
    body: Cursor<Vec<u8>>
}

pub fn to_readable_response(custom_response: CustomResponse) -> ReadableCustomResponse {
    ReadableCustomResponse {
        headers: custom_response.headers,
        raw_status: custom_response.raw_status,
        body: Cursor::new(custom_response.body)
    }
}

impl HttpResponse for ReadableCustomResponse {
    fn headers(&self) -> &Headers { &self.headers }
    fn status(&self) -> StatusCode {
        StatusCode::Ok
    }
    fn status_raw(&self) -> &RawStatus { &self.raw_status }
}

impl Read for ReadableCustomResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.body.read(buf)
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
                    let mut error_report = vec![format!("ssl error ({}):", openssl::version::version())];
                    let mut suggestion = None;
                    for err in errors {
                        if is_unknown_message_digest_err(err) {
                            suggestion = Some("<b>Servo recommends upgrading to a newer OpenSSL version.</b>");
                        }
                        error_report.push(format_ssl_error(err));
                    }

                    if let Some(suggestion) = suggestion {
                        error_report.push(suggestion.to_owned());
                    }

                    let error_report = error_report.join("<br>\n");
                    return Err(LoadError::new(url, LoadErrorType::Ssl { reason: error_report }));
                }
            }
        }

        let mut request = match connection {
            Ok(req) => req,
            Err(e) => return Err(
                LoadError::new(url, LoadErrorType::Connection { reason: e.description().to_owned() })),
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
            Err(e) => return Err(LoadError::new(url, LoadErrorType::Connection { reason: e.description().to_owned() })),
        };

        if let Some(ref data) = *body {
            if let Err(e) = request_writer.write_all(&data) {
                return Err(LoadError::new(url, LoadErrorType::Connection { reason: e.description().to_owned() }))
            }
        }

        let response = match request_writer.send() {
            Ok(w) => w,
            Err(HttpError::Io(ref io_error)) if io_error.kind() == io::ErrorKind::ConnectionAborted => {
                let error_type = LoadErrorType::ConnectionAborted { reason: io_error.description().to_owned() };
                return Err(LoadError::new(url, error_type));
            },
            Err(e) => return Err(LoadError::new(url, LoadErrorType::Connection { reason: e.description().to_owned() })),
        };

        Ok(WrappedHttpResponse { response: response })
    }
}

#[derive(Debug)]
pub struct LoadError {
    pub url: Url,
    pub error: LoadErrorType,
}

impl LoadError {
    pub fn new(url: Url, error: LoadErrorType) -> LoadError {
        LoadError {
            url: url,
            error: error,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum LoadErrorType {
    Cancelled,
    Connection { reason: String },
    ConnectionAborted { reason: String },
    ContentBlocked,
    // Preflight fetch inconsistent with main fetch
    CorsPreflightFetchInconsistent,
    Decoding { reason: String },
    InvalidRedirect { reason: String },
    MaxRedirects(u32), // u32 indicates number of redirects that occurred
    RedirectLoop,
    Ssl { reason: String },
    UnsupportedScheme { scheme: String },
}

impl fmt::Display for LoadErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for LoadErrorType {
    fn description(&self) -> &str {
        match *self {
            LoadErrorType::Cancelled => "load cancelled",
            LoadErrorType::Connection { ref reason } => reason,
            LoadErrorType::ConnectionAborted { ref reason } => reason,
            LoadErrorType::ContentBlocked => "content blocked",
            LoadErrorType::CorsPreflightFetchInconsistent => "preflight fetch inconsistent with main fetch",
            LoadErrorType::Decoding { ref reason } => reason,
            LoadErrorType::InvalidRedirect { ref reason } => reason,
            LoadErrorType::MaxRedirects(_) => "too many redirects",
            LoadErrorType::RedirectLoop => "redirect loop",
            LoadErrorType::Ssl { ref reason } => reason,
            LoadErrorType::UnsupportedScheme { .. } => "unsupported url scheme",
        }
    }
}

pub fn set_default_accept_encoding(headers: &mut Headers) {
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

pub fn set_default_accept_language(headers: &mut Headers) {
    if headers.has::<AcceptLanguage>() {
        return;
    }

    let mut en_us: LanguageTag = Default::default();
    en_us.language = Some("en".to_owned());
    en_us.region = Some("US".to_owned());
    let mut en: LanguageTag = Default::default();
    en.language = Some("en".to_owned());
    headers.set(AcceptLanguage(vec![
        qitem(en_us),
        QualityItem::new(en, Quality(500)),
    ]));
}

/// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-state-no-referrer-when-downgrade
fn no_referrer_when_downgrade_header(referrer_url: Url, url: Url) -> Option<Url> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    return strip_url(referrer_url, false);
}

/// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-strict-origin
fn strict_origin(referrer_url: Url, url: Url) -> Option<Url> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    strip_url(referrer_url, true)
}

/// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-strict-origin-when-cross-origin
fn strict_origin_when_cross_origin(referrer_url: Url, url: Url) -> Option<Url> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    let cross_origin = referrer_url.origin() != url.origin();
    strip_url(referrer_url, cross_origin)
}

/// https://w3c.github.io/webappsec-referrer-policy/#strip-url
fn strip_url(mut referrer_url: Url, origin_only: bool) -> Option<Url> {
    if referrer_url.scheme() == "https" || referrer_url.scheme() == "http" {
        referrer_url.set_username("").unwrap();
        referrer_url.set_password(None).unwrap();
        referrer_url.set_fragment(None);
        if origin_only {
            referrer_url.set_path("");
            referrer_url.set_query(None);
        }
        return Some(referrer_url);
    }
    return None;
}

/// https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer
pub fn determine_request_referrer(headers: &mut Headers,
                                  referrer_policy: Option<ReferrerPolicy>,
                                  referrer_url: Option<Url>,
                                  url: Url) -> Option<Url> {
    //TODO - algorithm step 2 not addressed
    assert!(!headers.has::<Referer>());
    if let Some(ref_url) = referrer_url {
        let cross_origin = ref_url.origin() != url.origin();
        return match referrer_policy {
            Some(ReferrerPolicy::NoReferrer) => None,
            Some(ReferrerPolicy::Origin) => strip_url(ref_url, true),
            Some(ReferrerPolicy::SameOrigin) => if cross_origin { None } else { strip_url(ref_url, false) },
            Some(ReferrerPolicy::UnsafeUrl) => strip_url(ref_url, false),
            Some(ReferrerPolicy::OriginWhenCrossOrigin) => strip_url(ref_url, cross_origin),
            Some(ReferrerPolicy::StrictOrigin) => strict_origin(ref_url, url),
            Some(ReferrerPolicy::StrictOriginWhenCrossOrigin) => strict_origin_when_cross_origin(ref_url, url),
            Some(ReferrerPolicy::NoReferrerWhenDowngrade) | None =>
                no_referrer_when_downgrade_header(ref_url, url),
        };
    }
    return None;
}

pub fn set_request_cookies(url: &Url, headers: &mut Headers, cookie_jar: &Arc<RwLock<CookieStorage>>) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    if let Some(cookie_list) = cookie_jar.cookies_for_url(url, CookieSource::HTTP) {
        let mut v = Vec::new();
        v.push(cookie_list.into_bytes());
        headers.set_raw("Cookie".to_owned(), v);
    }
}

fn set_cookie_for_url(cookie_jar: &Arc<RwLock<CookieStorage>>,
                      request: &Url,
                      cookie_val: String) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    let source = CookieSource::HTTP;
    let header = Header::parse_header(&[cookie_val.into_bytes()]);

    if let Ok(SetCookie(cookies)) = header {
        for bare_cookie in cookies {
            if let Some(cookie) = cookie::Cookie::new_wrapped(bare_cookie, request, source) {
                cookie_jar.push(cookie, source);
            }
        }
    }
}

pub fn set_cookies_from_headers(url: &Url, headers: &Headers, cookie_jar: &Arc<RwLock<CookieStorage>>) {
    if let Some(cookies) = headers.get_raw("set-cookie") {
        for cookie in cookies.iter() {
            if let Ok(cookie_value) = String::from_utf8(cookie.clone()) {
                set_cookie_for_url(&cookie_jar,
                                   &url,
                                   cookie_value);
            }
        }
    }
}

fn update_sts_list_from_response(url: &Url, response: &HttpResponse, hsts_list: &Arc<RwLock<HstsList>>) {
    if url.scheme() != "https" {
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

pub struct StreamedResponse {
    decoder: Decoder,
    pub metadata: Metadata
}


impl Read for StreamedResponse {
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

impl StreamedResponse {
    fn new(m: Metadata, d: Decoder) -> StreamedResponse {
        StreamedResponse { metadata: m, decoder: d }
    }

    pub fn from_http_response(response: Box<HttpResponse>, m: Metadata) -> Result<StreamedResponse, LoadError> {
        let decoder = match response.content_encoding() {
            Some(Encoding::Gzip) => {
                let result = GzDecoder::new(response);
                match result {
                    Ok(response_decoding) => Decoder::Gzip(response_decoding),
                    Err(err) => {
                        return Err(
                            LoadError::new(m.final_url, LoadErrorType::Decoding { reason: err.to_string() }))
                    }
                }
            }
            Some(Encoding::Deflate) => {
                Decoder::Deflate(DeflateDecoder::new(response))
            }
            Some(Encoding::EncodingExt(ref ext)) if ext == "br" => {
                Decoder::Brotli(Decompressor::new(response, 1024))
            }
            _ => {
                Decoder::Plain(response)
            }
        };
        Ok(StreamedResponse::new(m, decoder))
    }
}

enum Decoder {
    Gzip(GzDecoder<Box<HttpResponse>>),
    Deflate(DeflateDecoder<Box<HttpResponse>>),
    Brotli(Decompressor<Box<HttpResponse>>),
    Plain(Box<HttpResponse>)
}

fn prepare_devtools_request(request_id: String,
                            url: Url,
                            method: Method,
                            headers: Headers,
                            body: Option<Vec<u8>>,
                            pipeline_id: PipelineId,
                            now: Tm,
                            connect_time: u64,
                            send_time: u64,
                            is_xhr: bool) -> ChromeToDevtoolsControlMsg {
    let request = DevtoolsHttpRequest {
        url: url,
        method: method,
        headers: headers,
        body: body,
        pipeline_id: pipeline_id,
        startedDateTime: now,
        timeStamp: now.to_timespec().sec,
        connect_time: connect_time,
        send_time: send_time,
        is_xhr: is_xhr,
    };
    let net_event = NetworkEvent::HttpRequest(request);

    ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event)
}

pub fn send_request_to_devtools(msg: ChromeToDevtoolsControlMsg,
                            devtools_chan: &Sender<DevtoolsControlMsg>) {
    devtools_chan.send(DevtoolsControlMsg::FromChrome(msg)).unwrap();
}

pub fn send_response_to_devtools(devtools_chan: &Sender<DevtoolsControlMsg>,
                             request_id: String,
                             headers: Option<Headers>,
                             status: Option<(u16, Vec<u8>)>,
                             pipeline_id: PipelineId) {
    let response = DevtoolsHttpResponse { headers: headers, status: status, body: None, pipeline_id: pipeline_id };
    let net_event_response = NetworkEvent::HttpResponse(response);

    let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event_response);
    let _ = devtools_chan.send(DevtoolsControlMsg::FromChrome(msg));
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
                              referrer_policy: Option<ReferrerPolicy>,
                              referrer_url: &mut Option<Url>) {
    // Ensure that the host header is set from the original url
    let host = Host {
        hostname: url.host_str().unwrap().to_owned(),
        port: url.port_or_known_default()
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
    set_default_accept_language(headers);
    set_default_accept_encoding(headers);

    *referrer_url = determine_request_referrer(headers,
                                               referrer_policy.clone(),
                                               referrer_url.clone(),
                                               url.clone());

    if let Some(referrer_val) = referrer_url.clone() {
        headers.set(Referer(referrer_val.into_string()));
    }
}

fn set_auth_header(headers: &mut Headers,
                   url: &Url,
                   auth_cache: &Arc<RwLock<AuthCache>>) {
    if !headers.has::<Authorization<Basic>>() {
        if let Some(auth) = auth_from_url(url) {
            headers.set(auth);
        } else {
            if let Some(basic) = auth_from_cache(auth_cache, &url.origin()) {
                headers.set(Authorization(basic));
            }
        }
    }
}

pub fn auth_from_cache(auth_cache: &Arc<RwLock<AuthCache>>, origin: &Origin) -> Option<Basic> {
    if let Some(ref auth_entry) = auth_cache.read().unwrap().entries.get(&origin.ascii_serialization()) {
        let user_name = auth_entry.user_name.clone();
        let password  = Some(auth_entry.password.clone());
        Some(Basic { username: user_name, password: password })
    } else {
        None
    }
}

fn auth_from_url(doc_url: &Url) -> Option<Authorization<Basic>> {
    let username = doc_url.username();
    if username != "" {
        Some(Authorization(Basic {
            username: username.to_owned(),
            password: Some(doc_url.password().unwrap_or("").to_owned())
        }))
    } else {
        None
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
        set_cookies_from_headers(url, response.headers(), cookie_jar);
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
                          request_id: Option<&str>,
                          is_xhr: bool)
                          -> Result<(A::R, Option<ChromeToDevtoolsControlMsg>), LoadError>
                          where A: HttpRequest + 'static  {
    let null_data = None;
    let response;
    let connection_url = replace_hosts(&url);
    let mut msg;


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
            info!("{} {}", method, connection_url);
            for header in headers.iter() {
                info!(" - {}", header);
            }
            info!("{:?}", data);
        }

        let connect_start = precise_time_ms();

        let req = try!(request_factory.create(connection_url.clone(), method.clone(),
                                              headers.clone()));

        let connect_end = precise_time_ms();

        if cancel_listener.is_cancelled() {
            return Err(LoadError::new(connection_url.clone(), LoadErrorType::Cancelled));
        }

        let send_start = precise_time_ms();

        let maybe_response = req.send(request_body);

        let send_end = precise_time_ms();

        msg = if let Some(request_id) = request_id {
            if let Some(pipeline_id) = *pipeline_id {
                Some(prepare_devtools_request(
                    request_id.into(),
                    url.clone(), method.clone(), headers,
                    request_body.clone(), pipeline_id, time::now(),
                    connect_end - connect_start, send_end - send_start, is_xhr))
            } else {
                None
            }
        } else {
            None
        };

        response = match maybe_response {
            Ok(r) => r,
            Err(e) => {
                if let LoadErrorType::ConnectionAborted { reason } = e.error {
                    debug!("connection aborted ({:?}), possibly stale, trying new connection", reason);
                    continue;
                } else {
                    return Err(e)
                }
            },
        };

        // if no ConnectionAborted, break the loop
        break;
    }

    Ok((response, msg))
}

pub trait UIProvider {
    fn input_username_and_password(&self, prompt: &str) -> (Option<String>, Option<String>);
}

impl UIProvider for TFDProvider {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    fn input_username_and_password(&self, prompt: &str) -> (Option<String>, Option<String>) {
        (tinyfiledialogs::input_box(prompt, "Username:", ""),
        tinyfiledialogs::input_box(prompt, "Password:", ""))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn input_username_and_password(&self, _prompt: &str) -> (Option<String>, Option<String>) {
        (None, None)
    }
}

struct TFDProvider;

pub fn load<A, B>(load_data: &LoadData,
                  ui_provider: &B,
                  http_state: &HttpState,
                  devtools_chan: Option<Sender<DevtoolsControlMsg>>,
                  request_factory: &HttpRequestFactory<R=A>,
                  user_agent: Cow<'static, str>,
                  cancel_listener: &CancellationListener,
                  swmanager_chan: Option<IpcSender<CustomResponseMediator>>)
                  -> Result<StreamedResponse, LoadError> where A: HttpRequest + 'static, B: UIProvider {
    let max_redirects = PREFS.get("network.http.redirection-limit").as_i64().unwrap() as u32;
    let mut iters = 0;
    // URL of the document being loaded, as seen by all the higher-level code.
    let mut doc_url = load_data.url.clone();
    let mut redirected_to = HashSet::new();
    let mut method = load_data.method.clone();
    // URL of referrer - to be updated with redirects
    let mut referrer_url = load_data.referrer_url.clone();

    let mut new_auth_header: Option<Authorization<Basic>> = None;

    if cancel_listener.is_cancelled() {
        return Err(LoadError::new(doc_url, LoadErrorType::Cancelled));
    }

    let (msg_sender, msg_receiver) = ipc::channel().unwrap();
    let response_mediator = CustomResponseMediator {
        response_chan: msg_sender,
        load_url: doc_url.clone()
    };
    if let Some(sender) = swmanager_chan {
        let _ = sender.send(response_mediator);
        if let Ok(Some(custom_response)) = msg_receiver.recv() {
            let metadata = Metadata::default(doc_url.clone());
            let readable_response = to_readable_response(custom_response);
            return StreamedResponse::from_http_response(box readable_response, metadata);
        }
    } else {
        debug!("Did not receive a custom response");
    }

    // If the URL is a view-source scheme then the scheme data contains the
    // real URL that should be used for which the source is to be viewed.
    // Change our existing URL to that and keep note that we are viewing
    // the source rather than rendering the contents of the URL.
    let viewing_source = doc_url.scheme() == "view-source";
    if viewing_source {
        doc_url = Url::parse(&load_data.url[Position::BeforeUsername..]).unwrap();
    }

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if doc_url.scheme() == "http" && request_must_be_secured(&doc_url, &http_state.hsts_list) {
            info!("{} is in the strict transport security list, requesting secure host", doc_url);
            doc_url = secure_url(&doc_url);
        }

        if iters > max_redirects {
            return Err(LoadError::new(doc_url, LoadErrorType::MaxRedirects(iters - 1)));
        }

        if !matches!(doc_url.scheme(), "http" | "https") {
            let scheme = doc_url.scheme().to_owned();
            return Err(LoadError::new(doc_url, LoadErrorType::UnsupportedScheme { scheme: scheme }));
        }

        if cancel_listener.is_cancelled() {
            return Err(LoadError::new(doc_url, LoadErrorType::Cancelled));
        }

        let mut block_cookies = false;
        if let Some(ref rules) = *http_state.blocked_content {
            let same_origin =
                load_data.referrer_url.as_ref()
                         .map(|url| url.origin() == doc_url.origin())
                         .unwrap_or(false);
            let load_type = if same_origin { LoadType::FirstParty } else { LoadType::ThirdParty };
            let actions = process_rules_for_request(rules, &CBRequest {
                url: &doc_url,
                resource_type: to_resource_type(&load_data.context),
                load_type: load_type,
            });
            for action in actions {
                match action {
                    Reaction::Block => {
                        return Err(LoadError::new(doc_url, LoadErrorType::ContentBlocked));
                    },
                    Reaction::BlockCookies => block_cookies = true,
                    Reaction::HideMatchingElements(_) => (),
                }
            }
        }

        info!("requesting {}", doc_url);

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

        let request_id = devtools_chan.as_ref().map(|_| {
            uuid::Uuid::new_v4().simple().to_string()
        });

        modify_request_headers(&mut request_headers, &doc_url,
                               &user_agent, load_data.referrer_policy,
                               &mut referrer_url);

        // https://fetch.spec.whatwg.org/#concept-http-network-or-cache-fetch step 11
        if load_data.credentials_flag {
            if !block_cookies {
                set_request_cookies(&doc_url, &mut request_headers, &http_state.cookie_jar);
            }

            // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch step 12
            set_auth_header(&mut request_headers, &doc_url, &http_state.auth_cache);
        }

        //if there is a new auth header then set the request headers with it
        if let Some(ref auth_header) = new_auth_header {
            request_headers.set(auth_header.clone());
        }

        let (response, msg) =
            try!(obtain_response(request_factory, &doc_url, &method, &request_headers,
                                 &cancel_listener, &load_data.data, &load_data.method,
                                 &load_data.pipeline_id, iters,
                                 request_id.as_ref().map(Deref::deref), false));

        process_response_headers(&response, &doc_url, &http_state.cookie_jar, &http_state.hsts_list, &load_data);

        //if response status is unauthorized then prompt user for username and password
        if response.status() == StatusCode::Unauthorized &&
           response.headers().get_raw("WWW-Authenticate").is_some() {
            let (username_option, password_option) =
                ui_provider.input_username_and_password(doc_url.as_str());

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
            if response.status().class() == StatusClass::Success ||
               response.status().class() == StatusClass::Redirection {
                let auth_entry = AuthCacheEntry {
                    user_name: auth_header.username.to_owned(),
                    password: auth_header.password.to_owned().unwrap(),
                };

                let serialized_origin = doc_url.origin().ascii_serialization();
                http_state.auth_cache.write().unwrap().entries.insert(serialized_origin, auth_entry);
            }
        }

        // --- Loop if there's a redirect
        if response.status().class() == StatusClass::Redirection {
            if let Some(&Location(ref new_url)) = response.headers().get::<Location>() {
                // CORS (https://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                if let Some(ref c) = load_data.cors {
                    if c.preflight {
                        return Err(LoadError::new(doc_url, LoadErrorType::CorsPreflightFetchInconsistent));
                    } else {
                        // XXXManishearth There are some CORS-related steps here,
                        // but they don't seem necessary until credentials are implemented
                    }
                }

                let new_doc_url = match doc_url.join(&new_url) {
                    Ok(u) => u,
                    Err(e) => return Err(
                        LoadError::new(doc_url, LoadErrorType::InvalidRedirect { reason: e.to_string() })),
                };

                // According to https://tools.ietf.org/html/rfc7231#section-6.4.2,
                // historically UAs have rewritten POST->GET on 301 and 302 responses.
                if method == Method::Post &&
                    (response.status() == StatusCode::MovedPermanently ||
                        response.status() == StatusCode::Found) {
                    method = Method::Get;
                }

                if redirected_to.contains(&new_doc_url) {
                    return Err(LoadError::new(doc_url, LoadErrorType::RedirectLoop));
                }

                info!("redirecting to {}", new_doc_url);
                doc_url = new_doc_url;

                redirected_to.insert(doc_url.clone());
            }
        }

        // Only notify the devtools about the final request that received a response.
        if let Some(m) = msg {
            send_request_to_devtools(m, devtools_chan.as_ref().unwrap());
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
        metadata.headers = Some(Serde(adjusted_headers));
        metadata.status = Some((response.status_raw().0,
                                response.status_raw().1.as_bytes().to_vec()));
        metadata.https_state = if doc_url.scheme() == "https" {
            HttpsState::Modern
        } else {
            HttpsState::None
        };
        metadata.referrer = referrer_url.clone();

        // --- Tell devtools that we got a response
        // Send an HttpResponse message to devtools with the corresponding request_id
        // TODO: Send this message even when the load fails?
        if let Some(pipeline_id) = load_data.pipeline_id {
            if let Some(ref chan) = devtools_chan {
                send_response_to_devtools(
                    &chan, request_id.unwrap(),
                    metadata.headers.clone().map(Serde::into_inner),
                    metadata.status.clone(),
                    pipeline_id);
            }
        }
        if response.status().class() == StatusClass::Redirection {
            continue;
        } else {
            return StreamedResponse::from_http_response(box response, metadata);
        }
    }
}

fn send_data<R: Read>(context: LoadContext,
                      reader: &mut R,
                      start_chan: LoadConsumer,
                      metadata: Metadata,
                      classifier: Arc<MimeClassifier>,
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
            let _ = progress_chan.send(Done(Err(NetworkError::LoadCancelled)));
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
            function.to_uppercase() == "SSL3_GET_SERVER_CERTIFICATE" &&
            reason == "certificate verify failed"
        }
    }
}

fn is_unknown_message_digest_err(error: &OpensslError) -> bool {
    match error {
        &OpensslError::UnknownError { ref library, ref function, ref reason } => {
            library == "asn1 encoding routines" &&
            function == "ASN1_item_verify" &&
            reason == "unknown message digest algorithm"
        }
    }
}

fn format_ssl_error(error: &OpensslError) -> String {
    match error {
        &OpensslError::UnknownError { ref library, ref function, ref reason } => {
            format!("{}: {} - {}", library, function, reason)
        }
    }
}

fn to_resource_type(context: &LoadContext) -> ResourceType {
    match *context {
        LoadContext::Browsing => ResourceType::Document,
        LoadContext::Image => ResourceType::Image,
        LoadContext::AudioVideo => ResourceType::Media,
        LoadContext::Plugin => ResourceType::Raw,
        LoadContext::Style => ResourceType::StyleSheet,
        LoadContext::Script => ResourceType::Script,
        LoadContext::Font => ResourceType::Font,
        LoadContext::TextTrack => ResourceType::Media,
        LoadContext::CacheManifest => ResourceType::Raw,
    }
}
