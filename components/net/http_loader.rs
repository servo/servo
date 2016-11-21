/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use brotli::Decompressor;
use connector::Connector;
use content_blocker_parser::RuleList;
use cookie;
use cookie_storage::CookieStorage;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest};
use devtools_traits::{HttpResponse as DevtoolsHttpResponse, NetworkEvent};
use flate2::read::{DeflateDecoder, GzDecoder};
use hsts::HstsList;
use hyper::Error as HttpError;
use hyper::LanguageTag;
use hyper::client::{Pool, Request, Response};
use hyper::header::{AcceptEncoding, AcceptLanguage, Basic, ContentEncoding, ContentLength};
use hyper::header::{Encoding, Header, Headers, Quality, QualityItem, Referer};
use hyper::header::{SetCookie, qitem};
use hyper::http::RawStatus;
use hyper::method::Method;
use hyper::net::Fresh;
use hyper::status::StatusCode;
use log;
use msg::constellation_msg::PipelineId;
use net_traits::{CookieSource, Metadata, ReferrerPolicy};
use net_traits::hosts::replace_hosts;
use openssl;
use openssl::ssl::error::{OpensslError, SslError};
use resource_thread::AuthCache;
use servo_url::ServoUrl;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use time;
use time::Tm;
use url::Origin;

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

pub trait HttpRequestFactory {
    type R: HttpRequest;

    fn create(&self, url: ServoUrl, method: Method, headers: Headers) -> Result<Self::R, LoadError>;
}

pub struct NetworkHttpRequestFactory {
    pub connector: Arc<Pool<Connector>>,
}

impl HttpRequestFactory for NetworkHttpRequestFactory {
    type R = WrappedHttpRequest;

    fn create(&self, url: ServoUrl, method: Method, headers: Headers)
              -> Result<WrappedHttpRequest, LoadError> {
        let connection = Request::with_connector(method,
                                                 url.clone().into_url().unwrap(),
                                                 &*self.connector);

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
        let url = ServoUrl::from_url(self.request.url.clone());
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
    pub url: ServoUrl,
    pub error: LoadErrorType,
}

impl LoadError {
    pub fn new(url: ServoUrl, error: LoadErrorType) -> LoadError {
        LoadError {
            url: url,
            error: error,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum LoadErrorType {
    Connection { reason: String },
    ConnectionAborted { reason: String },
    Decoding { reason: String },
    Ssl { reason: String },
}

impl fmt::Display for LoadErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for LoadErrorType {
    fn description(&self) -> &str {
        match *self {
            LoadErrorType::Connection { ref reason } => reason,
            LoadErrorType::ConnectionAborted { ref reason } => reason,
            LoadErrorType::Decoding { ref reason } => reason,
            LoadErrorType::Ssl { ref reason } => reason,
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
fn no_referrer_when_downgrade_header(referrer_url: ServoUrl, url: ServoUrl) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    return strip_url(referrer_url, false);
}

/// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-strict-origin
fn strict_origin(referrer_url: ServoUrl, url: ServoUrl) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    strip_url(referrer_url, true)
}

/// https://w3c.github.io/webappsec-referrer-policy/#referrer-policy-strict-origin-when-cross-origin
fn strict_origin_when_cross_origin(referrer_url: ServoUrl, url: ServoUrl) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" && url.scheme() != "https" {
        return None;
    }
    let cross_origin = referrer_url.origin() != url.origin();
    strip_url(referrer_url, cross_origin)
}

/// https://w3c.github.io/webappsec-referrer-policy/#strip-url
fn strip_url(mut referrer_url: ServoUrl, origin_only: bool) -> Option<ServoUrl> {
    if referrer_url.scheme() == "https" || referrer_url.scheme() == "http" {
        {
            let referrer = referrer_url.as_mut_url().unwrap();
            referrer.set_username("").unwrap();
            referrer.set_password(None).unwrap();
            referrer.set_fragment(None);
            if origin_only {
                referrer.set_path("");
                referrer.set_query(None);
            }
        }
        return Some(referrer_url);
    }
    return None;
}

/// https://w3c.github.io/webappsec-referrer-policy/#determine-requests-referrer
pub fn determine_request_referrer(headers: &mut Headers,
                                  referrer_policy: Option<ReferrerPolicy>,
                                  referrer_url: Option<ServoUrl>,
                                  url: ServoUrl) -> Option<ServoUrl> {
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

pub fn set_request_cookies(url: &ServoUrl, headers: &mut Headers, cookie_jar: &Arc<RwLock<CookieStorage>>) {
    let mut cookie_jar = cookie_jar.write().unwrap();
    if let Some(cookie_list) = cookie_jar.cookies_for_url(url, CookieSource::HTTP) {
        let mut v = Vec::new();
        v.push(cookie_list.into_bytes());
        headers.set_raw("Cookie".to_owned(), v);
    }
}

fn set_cookie_for_url(cookie_jar: &Arc<RwLock<CookieStorage>>,
                      request: &ServoUrl,
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

pub fn set_cookies_from_headers(url: &ServoUrl, headers: &Headers, cookie_jar: &Arc<RwLock<CookieStorage>>) {
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
                            url: ServoUrl,
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

pub fn auth_from_cache(auth_cache: &Arc<RwLock<AuthCache>>, origin: &Origin) -> Option<Basic> {
    if let Some(ref auth_entry) = auth_cache.read().unwrap().entries.get(&origin.ascii_serialization()) {
        let user_name = auth_entry.user_name.clone();
        let password  = Some(auth_entry.password.clone());
        Some(Basic { username: user_name, password: password })
    } else {
        None
    }
}

pub fn obtain_response<A>(request_factory: &HttpRequestFactory<R=A>,
                          url: &ServoUrl,
                          method: &Method,
                          request_headers: &Headers,
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
                debug!("Not notifying devtools (no pipeline_id)");
                None
            }
        } else {
            debug!("Not notifying devtools (no request_id)");
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
