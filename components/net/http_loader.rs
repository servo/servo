/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use brotli::Decompressor;
use connector::{Connector, create_http_connector};
use cookie;
use cookie_storage::CookieStorage;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, HttpRequest as DevtoolsHttpRequest};
use devtools_traits::{HttpResponse as DevtoolsHttpResponse, NetworkEvent};
use fetch::cors_cache::CorsCache;
use fetch::methods::{Data, DoneChannel, FetchContext, Target, is_simple_header, is_simple_method, main_fetch};
use flate2::read::{DeflateDecoder, GzDecoder};
use hsts::HstsList;
use hyper::Error as HttpError;
use hyper::LanguageTag;
use hyper::client::{Pool, Request as HyperRequest, Response as HyperResponse};
use hyper::client::pool::PooledStream;
use hyper::header::{AcceptEncoding, AcceptLanguage, AccessControlAllowCredentials};
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders, AccessControlAllowMethods};
use hyper::header::{AccessControlRequestHeaders, AccessControlMaxAge, AccessControlRequestMethod};
use hyper::header::{Authorization, Basic, CacheControl, CacheDirective, ContentEncoding};
use hyper::header::{ContentLength, Encoding, Header, Headers, Host, IfMatch, IfRange};
use hyper::header::{IfUnmodifiedSince, IfModifiedSince, IfNoneMatch, Location, Pragma, Quality};
use hyper::header::{QualityItem, Referer, SetCookie, UserAgent, qitem};
use hyper::header::Origin as HyperOrigin;
use hyper::method::Method;
use hyper::net::{Fresh, HttpStream, HttpsStream, NetworkConnector};
use hyper::status::StatusCode;
use hyper_serde::Serde;
use log;
use msg::constellation_msg::PipelineId;
use net_traits::{CookieSource, FetchMetadata, NetworkError, ReferrerPolicy};
use net_traits::hosts::replace_host;
use net_traits::request::{CacheMode, CredentialsMode, Destination, Origin};
use net_traits::request::{RedirectMode, Referrer, Request, RequestMode, ResponseTainting};
use net_traits::response::{HttpsState, Response, ResponseBody, ResponseType};
use openssl;
use openssl::ssl::SslStream;
use openssl::ssl::error::{OpensslError, SslError};
use resource_thread::AuthCache;
use servo_url::{ImmutableOrigin, ServoUrl};
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Read, Write};
use std::iter::FromIterator;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use time;
use time::Tm;
use unicase::UniCase;
use uuid;

fn read_block<R: Read>(reader: &mut R) -> Result<Data, ()> {
    let mut buf = vec![0; 1024];

    match reader.read(&mut buf) {
        Ok(len) if len > 0 => {
            buf.truncate(len);
            Ok(Data::Payload(buf))
        }
        Ok(_) => Ok(Data::Done),
        Err(_) => Err(()),
    }
}

pub struct HttpState {
    pub hsts_list: Arc<RwLock<HstsList>>,
    pub cookie_jar: Arc<RwLock<CookieStorage>>,
    pub auth_cache: Arc<RwLock<AuthCache>>,
    pub connector_pool: Arc<Pool<Connector>>,
}

impl HttpState {
    pub fn new(certificate_path: &str) -> HttpState {
        HttpState {
            hsts_list: Arc::new(RwLock::new(HstsList::new())),
            cookie_jar: Arc::new(RwLock::new(CookieStorage::new(150))),
            auth_cache: Arc::new(RwLock::new(AuthCache::new())),
            connector_pool: create_http_connector(certificate_path),
        }
    }
}

fn precise_time_ms() -> u64 {
    time::precise_time_ns() / (1000 * 1000)
}

pub struct WrappedHttpResponse {
    pub response: HyperResponse
}

impl Read for WrappedHttpResponse {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.response.read(buf)
    }
}

impl WrappedHttpResponse {
    fn headers(&self) -> &Headers {
        &self.response.headers
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

struct NetworkHttpRequestFactory {
    pub connector: Arc<Pool<Connector>>,
}

impl NetworkConnector for NetworkHttpRequestFactory {
    type Stream = PooledStream<HttpsStream<SslStream<HttpStream>>>;

    fn connect(&self, host: &str, port: u16, scheme: &str) -> Result<Self::Stream, HttpError> {
        self.connector.connect(&replace_host(host), port, scheme)
    }
}

impl NetworkHttpRequestFactory {
    fn create(&self, url: ServoUrl, method: Method, headers: Headers)
              -> Result<HyperRequest<Fresh>, NetworkError> {
        let connection = HyperRequest::with_connector(method, url.clone().into_url(), self);

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
                    return Err(NetworkError::SslValidation(url, error_report));
                }
            }
        }

        let mut request = match connection {
            Ok(req) => req,
            Err(e) => return Err(NetworkError::Internal(e.description().to_owned())),
        };
        *request.headers_mut() = headers;

        Ok(request)
    }
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
            let referrer = referrer_url.as_mut_url();
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
/// Steps 4-6.
pub fn determine_request_referrer(headers: &mut Headers,
                                  referrer_policy: ReferrerPolicy,
                                  referrer_source: ServoUrl,
                                  current_url: ServoUrl)
                                  -> Option<ServoUrl> {
    assert!(!headers.has::<Referer>());
    // FIXME(#14505): this does not seem to be the correct way of checking for
    //                same-origin requests.
    let cross_origin = referrer_source.origin() != current_url.origin();
    // FIXME(#14506): some of these cases are expected to consider whether the
    //                request's client is "TLS-protected", whatever that means.
    match referrer_policy {
        ReferrerPolicy::NoReferrer => None,
        ReferrerPolicy::Origin => strip_url(referrer_source, true),
        ReferrerPolicy::SameOrigin => if cross_origin { None } else { strip_url(referrer_source, false) },
        ReferrerPolicy::UnsafeUrl => strip_url(referrer_source, false),
        ReferrerPolicy::OriginWhenCrossOrigin => strip_url(referrer_source, cross_origin),
        ReferrerPolicy::StrictOrigin => strict_origin(referrer_source, current_url),
        ReferrerPolicy::StrictOriginWhenCrossOrigin => strict_origin_when_cross_origin(referrer_source, current_url),
        ReferrerPolicy::NoReferrerWhenDowngrade => no_referrer_when_downgrade_header(referrer_source, current_url),
    }
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
                cookie_jar.push(cookie, request, source);
            }
        }
    }
}

fn set_cookies_from_headers(url: &ServoUrl, headers: &Headers, cookie_jar: &Arc<RwLock<CookieStorage>>) {
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

struct StreamedResponse {
    decoder: Decoder,
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
    fn from_http_response(response: WrappedHttpResponse) -> io::Result<StreamedResponse> {
        let decoder = match response.content_encoding() {
            Some(Encoding::Gzip) => {
                Decoder::Gzip(try!(GzDecoder::new(response)))
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
        Ok(StreamedResponse { decoder: decoder })
    }
}

enum Decoder {
    Gzip(GzDecoder<WrappedHttpResponse>),
    Deflate(DeflateDecoder<WrappedHttpResponse>),
    Brotli(Decompressor<WrappedHttpResponse>),
    Plain(WrappedHttpResponse)
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

fn send_request_to_devtools(msg: ChromeToDevtoolsControlMsg,
                            devtools_chan: &Sender<DevtoolsControlMsg>) {
    devtools_chan.send(DevtoolsControlMsg::FromChrome(msg)).unwrap();
}

fn send_response_to_devtools(devtools_chan: &Sender<DevtoolsControlMsg>,
                             request_id: String,
                             headers: Option<Headers>,
                             status: Option<(u16, Vec<u8>)>,
                             pipeline_id: PipelineId) {
    let response = DevtoolsHttpResponse { headers: headers, status: status, body: None, pipeline_id: pipeline_id };
    let net_event_response = NetworkEvent::HttpResponse(response);

    let msg = ChromeToDevtoolsControlMsg::NetworkEvent(request_id, net_event_response);
    let _ = devtools_chan.send(DevtoolsControlMsg::FromChrome(msg));
}

fn auth_from_cache(auth_cache: &Arc<RwLock<AuthCache>>, origin: &ImmutableOrigin) -> Option<Basic> {
    if let Some(ref auth_entry) = auth_cache.read().unwrap().entries.get(&origin.ascii_serialization()) {
        let user_name = auth_entry.user_name.clone();
        let password  = Some(auth_entry.password.clone());
        Some(Basic { username: user_name, password: password })
    } else {
        None
    }
}

fn obtain_response(request_factory: &NetworkHttpRequestFactory,
                   url: &ServoUrl,
                   method: &Method,
                   request_headers: &Headers,
                   data: &Option<Vec<u8>>,
                   load_data_method: &Method,
                   pipeline_id: &Option<PipelineId>,
                   iters: u32,
                   request_id: Option<&str>,
                   is_xhr: bool)
                   -> Result<(WrappedHttpResponse, Option<ChromeToDevtoolsControlMsg>), NetworkError> {
    let null_data = None;

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
            info!("{} {}", method, url);
            for header in headers.iter() {
                info!(" - {}", header);
            }
            info!("{:?}", data);
        }

        let connect_start = precise_time_ms();

        let request = try!(request_factory.create(url.clone(), method.clone(),
                                                  headers.clone()));

        let connect_end = precise_time_ms();

        let send_start = precise_time_ms();

        let mut request_writer = match request.start() {
            Ok(streaming) => streaming,
            Err(e) => return Err(NetworkError::Internal(e.description().to_owned())),
        };

        if let Some(ref data) = *request_body {
            if let Err(e) = request_writer.write_all(&data) {
                return Err(NetworkError::Internal(e.description().to_owned()))
            }
        }

        let response = match request_writer.send() {
            Ok(w) => w,
            Err(HttpError::Io(ref io_error)) if io_error.kind() == io::ErrorKind::ConnectionAborted => {
                debug!("connection aborted ({:?}), possibly stale, trying new connection", io_error.description());
                continue;
            },
            Err(e) => return Err(NetworkError::Internal(e.description().to_owned())),
        };

        let send_end = precise_time_ms();

        let msg = if let Some(request_id) = request_id {
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

        return Ok((WrappedHttpResponse { response: response }, msg));
    }
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

/// [HTTP fetch](https://fetch.spec.whatwg.org#http-fetch)
pub fn http_fetch(request: Rc<Request>,
                  cache: &mut CorsCache,
                  cors_flag: bool,
                  cors_preflight_flag: bool,
                  authentication_fetch_flag: bool,
                  target: Target,
                  done_chan: &mut DoneChannel,
                  context: &FetchContext)
                  -> Response {
    // This is a new async fetch, reset the channel we are waiting on
    *done_chan = None;
    // Step 1
    let mut response: Option<Response> = None;

    // Step 2
    // nothing to do, since actual_response is a function on response

    // Step 3
    if !request.skip_service_worker.get() && !request.is_service_worker_global_scope {
        // Substep 1
        // TODO (handle fetch unimplemented)

        if let Some(ref res) = response {
            // Substep 2
            // nothing to do, since actual_response is a function on response

            // Substep 3
            if (res.response_type == ResponseType::Opaque &&
                request.mode != RequestMode::NoCors) ||
               (res.response_type == ResponseType::OpaqueRedirect &&
                request.redirect_mode.get() != RedirectMode::Manual) ||
               (res.url_list.borrow().len() > 1 &&
                request.redirect_mode.get() != RedirectMode::Follow) ||
               res.is_network_error() {
                return Response::network_error(NetworkError::Internal("Request failed".into()));
            }

            // Substep 4
            // TODO: set response's CSP list on actual_response
        }
    }

    // Step 4
    let credentials = match request.credentials_mode {
        CredentialsMode::Include => true,
        CredentialsMode::CredentialsSameOrigin if request.response_tainting.get() == ResponseTainting::Basic
            => true,
        _ => false
    };

    // Step 5
    if response.is_none() {
        // Substep 1
        if cors_preflight_flag {
            let method_cache_match = cache.match_method(&*request,
                                                        request.method.borrow().clone());

            let method_mismatch = !method_cache_match && (!is_simple_method(&request.method.borrow()) ||
                                                          request.use_cors_preflight);
            let header_mismatch = request.headers.borrow().iter().any(|view|
                !cache.match_header(&*request, view.name()) && !is_simple_header(&view)
            );

            // Sub-substep 1
            if method_mismatch || header_mismatch {
                let preflight_result = cors_preflight_fetch(request.clone(), cache, context);
                // Sub-substep 2
                if let Some(e) = preflight_result.get_network_error() {
                    return Response::network_error(e.clone());
                }
            }
        }

        // Substep 2
        request.skip_service_worker.set(true);

        // Substep 3
        let fetch_result = http_network_or_cache_fetch(request.clone(), authentication_fetch_flag,
                                                       cors_flag, done_chan, context);

        // Substep 4
        if cors_flag && cors_check(request.clone(), &fetch_result).is_err() {
            return Response::network_error(NetworkError::Internal("CORS check failed".into()));
        }

        fetch_result.return_internal.set(false);
        response = Some(fetch_result);
    }

    // response is guaranteed to be something by now
    let mut response = response.unwrap();

    // Step 5
    match response.actual_response().status {
        // Code 301, 302, 303, 307, 308
        Some(StatusCode::MovedPermanently) |
        Some(StatusCode::Found) |
        Some(StatusCode::SeeOther) |
        Some(StatusCode::TemporaryRedirect) |
        Some(StatusCode::PermanentRedirect) => {
            response = match request.redirect_mode.get() {
                RedirectMode::Error => Response::network_error(NetworkError::Internal("Redirect mode error".into())),
                RedirectMode::Manual => {
                    response.to_filtered(ResponseType::OpaqueRedirect)
                },
                RedirectMode::Follow => {
                    // set back to default
                    response.return_internal.set(true);
                    http_redirect_fetch(request, cache, response,
                                        cors_flag, target, done_chan, context)
                }
            }
        },

        // Code 401
        Some(StatusCode::Unauthorized) => {
            // Step 1
            // FIXME: Figure out what to do with request window objects
            if cors_flag || !credentials {
                return response;
            }

            // Step 2
            // TODO: Spec says requires testing on multiple WWW-Authenticate headers

            // Step 3
            if !request.use_url_credentials || authentication_fetch_flag {
                // TODO: Prompt the user for username and password from the window
                // Wrong, but will have to do until we are able to prompt the user
                // otherwise this creates an infinite loop
                // We basically pretend that the user declined to enter credentials
                return response;
            }

            // Step 4
            return http_fetch(request, cache, cors_flag, cors_preflight_flag,
                              true, target, done_chan, context);
        }

        // Code 407
        Some(StatusCode::ProxyAuthenticationRequired) => {
            // Step 1
            // TODO: Figure out what to do with request window objects

            // Step 2
            // TODO: Spec says requires testing on Proxy-Authenticate headers

            // Step 3
            // TODO: Prompt the user for proxy authentication credentials
            // Wrong, but will have to do until we are able to prompt the user
            // otherwise this creates an infinite loop
            // We basically pretend that the user declined to enter credentials
            return response;

            // Step 4
            // return http_fetch(request, cache,
            //                   cors_flag, cors_preflight_flag,
            //                   authentication_fetch_flag, target,
            //                   done_chan, context);
        }

        _ => { }
    }

    // Step 6
    if authentication_fetch_flag {
        // TODO: Create authentication entry for this request
    }

    // set back to default
    response.return_internal.set(true);
    // Step 7
    response
}

/// [HTTP redirect fetch](https://fetch.spec.whatwg.org#http-redirect-fetch)
fn http_redirect_fetch(request: Rc<Request>,
                       cache: &mut CorsCache,
                       response: Response,
                       cors_flag: bool,
                       target: Target,
                       done_chan: &mut DoneChannel,
                       context: &FetchContext)
                       -> Response {
    // Step 1
    assert_eq!(response.return_internal.get(), true);

    // Step 2
    if !response.actual_response().headers.has::<Location>() {
        return response;
    }

    // Step 3
    let location = match response.actual_response().headers.get::<Location>() {
        Some(&Location(ref location)) => location.clone(),
        _ => return Response::network_error(NetworkError::Internal("Location header parsing failure".into()))
    };
    let response_url = response.actual_response().url().unwrap();
    let location_url = response_url.join(&*location);
    let location_url = match location_url {
        Ok(url) => url,
        _ => return Response::network_error(NetworkError::Internal("Location URL parsing failure".into()))
    };

    // Step 4
    match location_url.scheme() {
        "http" | "https" => { },
        _ => return Response::network_error(NetworkError::Internal("Not an HTTP(S) Scheme".into()))
    }

    // Step 5
    if request.redirect_count.get() >= 20 {
        return Response::network_error(NetworkError::Internal("Too many redirects".into()));
    }

    // Step 6
    request.redirect_count.set(request.redirect_count.get() + 1);

    // Step 7
    let same_origin = location_url.origin()== request.current_url().origin();
    let has_credentials = has_credentials(&location_url);

    if request.mode == RequestMode::CorsMode && !same_origin && has_credentials {
        return Response::network_error(NetworkError::Internal("Cross-origin credentials check failed".into()));
    }

    // Step 8
    if cors_flag && has_credentials {
        return Response::network_error(NetworkError::Internal("Credentials check failed".into()));
    }

    // Step 9
    if cors_flag && !same_origin {
        *request.origin.borrow_mut() = Origin::Origin(ImmutableOrigin::new_opaque());
    }

    // Step 10
    let status_code = response.actual_response().status.unwrap();
    if ((status_code == StatusCode::MovedPermanently || status_code == StatusCode::Found) &&
        *request.method.borrow() == Method::Post) ||
        status_code == StatusCode::SeeOther {
        *request.method.borrow_mut() = Method::Get;
        *request.body.borrow_mut() = None;
    }

    // Step 11
    request.url_list.borrow_mut().push(location_url);

    // Step 12
    // TODO implement referrer policy

    // Step 13
    main_fetch(request, cache, cors_flag, true, target, done_chan, context)
}

fn try_immutable_origin_to_hyper_origin(url_origin: &ImmutableOrigin) -> Option<HyperOrigin> {
    match *url_origin {
        // TODO (servo/servo#15569) Set "Origin: null" when hyper supports it
        ImmutableOrigin::Opaque(_) => None,
        ImmutableOrigin::Tuple(ref scheme, ref host, ref port) =>
            Some(HyperOrigin::new(scheme.clone(), host.to_string(), Some(port.clone())))
    }
}

/// [HTTP network or cache fetch](https://fetch.spec.whatwg.org#http-network-or-cache-fetch)
fn http_network_or_cache_fetch(request: Rc<Request>,
                               authentication_fetch_flag: bool,
                               cors_flag: bool,
                               done_chan: &mut DoneChannel,
                               context: &FetchContext)
                               -> Response {
    // TODO: Implement Window enum for Request
    let request_has_no_window = true;

    // Step 1
    let http_request = if request_has_no_window &&
        request.redirect_mode.get() == RedirectMode::Error {
        request
    } else {
        Rc::new((*request).clone())
    };

    // Step 2
    let credentials_flag = match http_request.credentials_mode {
        CredentialsMode::Include => true,
        CredentialsMode::CredentialsSameOrigin if http_request.response_tainting.get() == ResponseTainting::Basic
            => true,
        _ => false
    };

    let content_length_value = match *http_request.body.borrow() {
        None =>
            match *http_request.method.borrow() {
                // Step 4
                Method::Post | Method::Put =>
                    Some(0),
                // Step 3
                _ => None
            },
        // Step 5
        Some(ref http_request_body) => Some(http_request_body.len() as u64)
    };

    // Step 6
    if let Some(content_length_value) = content_length_value {
        http_request.headers.borrow_mut().set(ContentLength(content_length_value));
    }

    // Step 7 TODO

    // Step 8
    match *http_request.referrer.borrow() {
        Referrer::NoReferrer => (),
        Referrer::ReferrerUrl(ref http_request_referrer) =>
            http_request.headers.borrow_mut().set(Referer(http_request_referrer.to_string())),
        Referrer::Client =>
            // it should be impossible for referrer to be anything else during fetching
            // https://fetch.spec.whatwg.org/#concept-request-referrer
            unreachable!()
    };

    // Step 9
    if !http_request.omit_origin_header.get() {
        let method = http_request.method.borrow();
        if cors_flag || (*method != Method::Get && *method != Method::Head) {
            debug_assert!(*http_request.origin.borrow() != Origin::Client);
            if let Origin::Origin(ref url_origin) = *http_request.origin.borrow() {
                if let Some(hyper_origin) = try_immutable_origin_to_hyper_origin(url_origin) {
                    http_request.headers.borrow_mut().set(hyper_origin)
                }
            }
        }
    }

    // Step 10
    if !http_request.headers.borrow().has::<UserAgent>() {
        let user_agent = context.user_agent.clone().into_owned();
        http_request.headers.borrow_mut().set(UserAgent(user_agent));
    }

    match http_request.cache_mode.get() {
        // Step 11
        CacheMode::Default if is_no_store_cache(&http_request.headers.borrow()) => {
            http_request.cache_mode.set(CacheMode::NoStore);
        },

        // Step 12
        CacheMode::NoCache if !http_request.headers.borrow().has::<CacheControl>() => {
            http_request.headers.borrow_mut().set(CacheControl(vec![CacheDirective::MaxAge(0)]));
        },

        // Step 13
        CacheMode::Reload | CacheMode::NoStore => {
            // Substep 1
            if !http_request.headers.borrow().has::<Pragma>() {
                http_request.headers.borrow_mut().set(Pragma::NoCache);
            }

            // Substep 2
            if !http_request.headers.borrow().has::<CacheControl>() {
                http_request.headers.borrow_mut().set(CacheControl(vec![CacheDirective::NoCache]));
            }
        },

        _ => {}
    }

    // Step 14
    let current_url = http_request.current_url();
    {
        let headers = &mut *http_request.headers.borrow_mut();
        let host = Host {
            hostname: current_url.host_str().unwrap().to_owned(),
            port: current_url.port()
        };
        headers.set(host);
        // unlike http_loader, we should not set the accept header
        // here, according to the fetch spec
        set_default_accept_encoding(headers);
    }

    // Step 15
    // TODO some of this step can't be implemented yet
    if credentials_flag {
        // Substep 1
        // TODO http://mxr.mozilla.org/servo/source/components/net/http_loader.rs#504
        // XXXManishearth http_loader has block_cookies: support content blocking here too
        set_request_cookies(&current_url,
                            &mut *http_request.headers.borrow_mut(),
                            &context.state.cookie_jar);
        // Substep 2
        if !http_request.headers.borrow().has::<Authorization<String>>() {
            // Substep 3
            let mut authorization_value = None;

            // Substep 4
            if let Some(basic) = auth_from_cache(&context.state.auth_cache, &current_url.origin()) {
                if !http_request.use_url_credentials || !has_credentials(&current_url) {
                    authorization_value = Some(basic);
                }
            }

            // Substep 5
            if authentication_fetch_flag && authorization_value.is_none() {
                if has_credentials(&current_url) {
                    authorization_value = Some(Basic {
                        username: current_url.username().to_owned(),
                        password: current_url.password().map(str::to_owned)
                    })
                }
            }

            // Substep 6
            if let Some(basic) = authorization_value {
                http_request.headers.borrow_mut().set(Authorization(basic));
            }
        }
    }

    // Step 16
    // TODO If there’s a proxy-authentication entry, use it as appropriate.

    // Step 17
    let mut response: Option<Response> = None;

    // Step 18
    // TODO have a HTTP cache to check for a completed response
    let complete_http_response_from_cache: Option<Response> = None;
    if http_request.cache_mode.get() != CacheMode::NoStore &&
        http_request.cache_mode.get() != CacheMode::Reload &&
        complete_http_response_from_cache.is_some() {
        // Substep 1
        if http_request.cache_mode.get() == CacheMode::ForceCache ||
           http_request.cache_mode.get() == CacheMode::OnlyIfCached {
            // TODO pull response from HTTP cache
            // response = http_request
        }

        let revalidation_needed = match response {
            Some(ref response) => response_needs_revalidation(&response),
            _ => false
        };

        // Substep 2
        if !revalidation_needed && http_request.cache_mode.get() == CacheMode::Default {
            // TODO pull response from HTTP cache
            // response = http_request
            // response.cache_state = CacheState::Local;
        }

        // Substep 3
        if revalidation_needed && http_request.cache_mode.get() == CacheMode::Default ||
            http_request.cache_mode.get() == CacheMode::NoCache {
            // TODO this substep
        }

    // Step 19
    // TODO have a HTTP cache to check for a partial response
    } else if http_request.cache_mode.get() == CacheMode::Default ||
        http_request.cache_mode.get() == CacheMode::ForceCache {
        // TODO this substep
    }

    // Step 20
    if response.is_none() {
        if http_request.cache_mode.get() == CacheMode::OnlyIfCached {
            return Response::network_error(NetworkError::Internal("Couldn't find response in cache".into()))
        }
        response = Some(http_network_fetch(http_request.clone(), credentials_flag,
                                           done_chan, context));
    }
    let response = response.unwrap();

    if let Some(status) = response.status {
        match status {
            StatusCode::NotModified => {
                // Step 21
                if http_request.cache_mode.get() == CacheMode::Default ||
                   http_request.cache_mode.get() == CacheMode::NoCache {
                    // Substep 1
                    // TODO this substep
                    // let cached_response: Option<Response> = None;

                    // Substep 2
                    // if cached_response.is_none() {
                    //     return Response::network_error();
                    // }

                    // Substep 3

                    // Substep 4
                    // response = cached_response;

                    // Substep 5
                    // TODO cache_state is immutable?
                    // response.cache_state = CacheState::Validated;
                }
            },
            StatusCode::Unauthorized => {
                // Step 22
                // FIXME: Figure out what to do with request window objects
                if cors_flag && !credentials_flag {
                    return response;
                }

                // Step 1
                // TODO: Spec says requires testing on multiple WWW-Authenticate headers

                // Step 2
                if !http_request.use_url_credentials || authentication_fetch_flag {
                    // TODO: Prompt the user for username and password from the window
                    // Wrong, but will have to do until we are able to prompt the user
                    // otherwise this creates an infinite loop
                    // We basically pretend that the user declined to enter credentials
                    return response;
                }

                // Step 3
                return http_network_or_cache_fetch(http_request, true, cors_flag, done_chan, context);
            },
            StatusCode::ProxyAuthenticationRequired => {
                // Step 23
                // Step 1
                // TODO: Figure out what to do with request window objects

                // Step 2
                // TODO: Spec says requires testing on Proxy-Authenticate headers

                // Step 3
                // TODO: Prompt the user for proxy authentication credentials
                // Wrong, but will have to do until we are able to prompt the user
                // otherwise this creates an infinite loop
                // We basically pretend that the user declined to enter credentials
                return response;

                // Step 4
                // return http_network_or_cache_fetch(request, authentication_fetch_flag,
                //                                    cors_flag, done_chan, context);
            },
            _ => {}
        }
    }

    // Step 24
    if authentication_fetch_flag {
        // TODO Create the authentication entry for request and the given realm
    }

    // Step 25
    response
}

/// [HTTP network fetch](https://fetch.spec.whatwg.org/#http-network-fetch)
fn http_network_fetch(request: Rc<Request>,
                      credentials_flag: bool,
                      done_chan: &mut DoneChannel,
                      context: &FetchContext)
                      -> Response {
    // TODO: Implement HTTP network fetch spec

    // Step 1
    // nothing to do here, since credentials_flag is already a boolean

    // Step 2
    // TODO be able to create connection using current url's origin and credentials

    // Step 3
    // TODO be able to tell if the connection is a failure

    // Step 4
    let factory = NetworkHttpRequestFactory {
        connector: context.state.connector_pool.clone(),
    };

    let url = request.current_url();

    let request_id = context.devtools_chan.as_ref().map(|_| {
        uuid::Uuid::new_v4().simple().to_string()
    });

    // XHR uses the default destination; other kinds of fetches (which haven't been implemented yet)
    // do not. Once we support other kinds of fetches we'll need to be more fine grained here
    // since things like image fetches are classified differently by devtools
    let is_xhr = request.destination == Destination::None;
    let wrapped_response = obtain_response(&factory, &url, &request.method.borrow(),
                                           &request.headers.borrow(),
                                           &request.body.borrow(), &request.method.borrow(),
                                           &request.pipeline_id.get(), request.redirect_count.get() + 1,
                                           request_id.as_ref().map(Deref::deref), is_xhr);

    let pipeline_id = request.pipeline_id.get();
    let (res, msg) = match wrapped_response {
        Ok(wrapped_response) => wrapped_response,
        Err(error) => return Response::network_error(error),
    };

    let mut response = Response::new(url.clone());
    response.status = Some(res.response.status);
    response.raw_status = Some((res.response.status_raw().0,
                                res.response.status_raw().1.as_bytes().to_vec()));
    response.headers = res.response.headers.clone();
    response.referrer = request.referrer.borrow().to_url().cloned();

    let res_body = response.body.clone();

    // We're about to spawn a thread to be waited on here
    let (done_sender, done_receiver) = channel();
    *done_chan = Some((done_sender.clone(), done_receiver));
    let meta = match response.metadata().expect("Response metadata should exist at this stage") {
        FetchMetadata::Unfiltered(m) => m,
        FetchMetadata::Filtered { unsafe_, .. } => unsafe_
    };
    let devtools_sender = context.devtools_chan.clone();
    let meta_status = meta.status.clone();
    let meta_headers = meta.headers.clone();
    thread::Builder::new().name(format!("fetch worker thread")).spawn(move || {
        match StreamedResponse::from_http_response(res) {
            Ok(mut res) => {
                *res_body.lock().unwrap() = ResponseBody::Receiving(vec![]);

                if let Some(ref sender) = devtools_sender {
                    if let Some(m) = msg {
                        send_request_to_devtools(m, &sender);
                    }

                    // --- Tell devtools that we got a response
                    // Send an HttpResponse message to devtools with the corresponding request_id
                    if let Some(pipeline_id) = pipeline_id {
                        send_response_to_devtools(
                            &sender, request_id.unwrap(),
                            meta_headers.map(Serde::into_inner),
                            meta_status,
                            pipeline_id);
                    }
                }

                loop {
                    match read_block(&mut res) {
                        Ok(Data::Payload(chunk)) => {
                            if let ResponseBody::Receiving(ref mut body) = *res_body.lock().unwrap() {
                                body.extend_from_slice(&chunk);
                                let _ = done_sender.send(Data::Payload(chunk));
                            }
                        },
                        Ok(Data::Done) | Err(_) => {
                            let mut body = res_body.lock().unwrap();
                            let completed_body = match *body {
                                ResponseBody::Receiving(ref mut body) => {
                                    mem::replace(body, vec![])
                                },
                                _ => vec![],
                            };
                            *body = ResponseBody::Done(completed_body);
                            let _ = done_sender.send(Data::Done);
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                // XXXManishearth we should propagate this error somehow
                *res_body.lock().unwrap() = ResponseBody::Done(vec![]);
                let _ = done_sender.send(Data::Done);
            }
        }
    }).expect("Thread spawning failed");

        // TODO these substeps aren't possible yet
        // Substep 1

        // Substep 2

    // TODO Determine if response was retrieved over HTTPS
    // TODO Servo needs to decide what ciphers are to be treated as "deprecated"
    response.https_state = HttpsState::None;

    // TODO Read request

    // Step 5-9
    // (needs stream bodies)

    // Step 10
    // TODO when https://bugzilla.mozilla.org/show_bug.cgi?id=1030660
    // is resolved, this step will become uneccesary
    // TODO this step
    if let Some(encoding) = response.headers.get::<ContentEncoding>() {
        if encoding.contains(&Encoding::Gzip) {
        }

        else if encoding.contains(&Encoding::Compress) {
        }
    };

    // Step 11
    // TODO this step isn't possible yet (CSP)

    // Step 12
    if response.is_network_error() && request.cache_mode.get() == CacheMode::NoStore {
        // TODO update response in the HTTP cache for request
    }

    // TODO this step isn't possible yet
    // Step 13

    // Step 14.
    if credentials_flag {
        set_cookies_from_headers(&url, &response.headers, &context.state.cookie_jar);
    }

    // TODO these steps
    // Step 15
        // Substep 1
        // Substep 2
            // Sub-substep 1
            // Sub-substep 2
            // Sub-substep 3
            // Sub-substep 4
        // Substep 3

    // Step 16
    response
}

/// [CORS preflight fetch](https://fetch.spec.whatwg.org#cors-preflight-fetch)
fn cors_preflight_fetch(request: Rc<Request>,
                        cache: &mut CorsCache,
                        context: &FetchContext)
                        -> Response {
    // Step 1
    let mut preflight = Request::new(request.current_url(), Some(request.origin.borrow().clone()),
                                     request.is_service_worker_global_scope, request.pipeline_id.get());
    *preflight.method.borrow_mut() = Method::Options;
    preflight.initiator = request.initiator.clone();
    preflight.type_ = request.type_.clone();
    preflight.destination = request.destination.clone();
    *preflight.referrer.borrow_mut() = request.referrer.borrow().clone();
    preflight.referrer_policy.set(request.referrer_policy.get());

    // Step 2
    preflight.headers.borrow_mut().set::<AccessControlRequestMethod>(
        AccessControlRequestMethod(request.method.borrow().clone()));

    // Step 3, 4
    let mut value = request.headers.borrow().iter()
                                            .filter(|view| !is_simple_header(view))
                                            .map(|view| UniCase(view.name().to_owned()))
                                            .collect::<Vec<UniCase<String>>>();
    value.sort();

    // Step 5
    preflight.headers.borrow_mut().set::<AccessControlRequestHeaders>(
        AccessControlRequestHeaders(value));

    // Step 6
    let preflight = Rc::new(preflight);
    let response = http_network_or_cache_fetch(preflight.clone(), false, false, &mut None, context);

    // Step 7
    if cors_check(request.clone(), &response).is_ok() &&
       response.status.map_or(false, |status| status.is_success()) {
        // Substep 1
        let mut methods = if response.headers.has::<AccessControlAllowMethods>() {
            match response.headers.get::<AccessControlAllowMethods>() {
                Some(&AccessControlAllowMethods(ref m)) => m.clone(),
                // Substep 3
                None => return Response::network_error(NetworkError::Internal("CORS ACAM check failed".into()))
            }
        } else {
            vec![]
        };

        // Substep 2
        let header_names = if response.headers.has::<AccessControlAllowHeaders>() {
            match response.headers.get::<AccessControlAllowHeaders>() {
                Some(&AccessControlAllowHeaders(ref hn)) => hn.clone(),
                // Substep 3
                None => return Response::network_error(NetworkError::Internal("CORS ACAH check failed".into()))
            }
        } else {
            vec![]
        };

        // Substep 4
        if methods.is_empty() && request.use_cors_preflight {
            methods = vec![request.method.borrow().clone()];
        }

        // Substep 5
        debug!("CORS check: Allowed methods: {:?}, current method: {:?}",
                methods, request.method.borrow());
        if methods.iter().all(|method| *method != *request.method.borrow()) &&
            !is_simple_method(&*request.method.borrow()) {
            return Response::network_error(NetworkError::Internal("CORS method check failed".into()));
        }

        // Substep 6
        debug!("CORS check: Allowed headers: {:?}, current headers: {:?}",
                header_names, request.headers.borrow());
        let set: HashSet<&UniCase<String>> = HashSet::from_iter(header_names.iter());
        if request.headers.borrow().iter().any(|ref hv| !set.contains(&UniCase(hv.name().to_owned())) &&
                                                        !is_simple_header(hv)) {
            return Response::network_error(NetworkError::Internal("CORS headers check failed".into()));
        }

        // Substep 7, 8
        let max_age = response.headers.get::<AccessControlMaxAge>().map(|acma| acma.0).unwrap_or(0);

        // TODO: Substep 9 - Need to define what an imposed limit on max-age is

        // Substep 11, 12
        for method in &methods {
            cache.match_method_and_update(&*request, method.clone(), max_age);
        }

        // Substep 13, 14
        for header_name in &header_names {
            cache.match_header_and_update(&*request, &*header_name, max_age);
        }

        // Substep 15
        return response;
    }

    // Step 8
    Response::network_error(NetworkError::Internal("CORS check failed".into()))
}

/// [CORS check](https://fetch.spec.whatwg.org#concept-cors-check)
fn cors_check(request: Rc<Request>, response: &Response) -> Result<(), ()> {
    // Step 1
    let origin = response.headers.get::<AccessControlAllowOrigin>().cloned();

    // Step 2
    let origin = try!(origin.ok_or(()));

    // Step 3
    if request.credentials_mode != CredentialsMode::Include &&
       origin == AccessControlAllowOrigin::Any {
        return Ok(());
    }

    // Step 4
    let origin = match origin {
        AccessControlAllowOrigin::Value(origin) => origin,
        // if it's Any or Null at this point, there's nothing to do but return Err(())
        _ => return Err(())
    };

    match *request.origin.borrow() {
        Origin::Origin(ref o) if o.ascii_serialization() == origin => {},
        _ => return Err(())
    }

    // Step 5
    if request.credentials_mode != CredentialsMode::Include {
        return Ok(());
    }

    // Step 6
    let credentials = response.headers.get::<AccessControlAllowCredentials>().cloned();

    // Step 7
    if credentials.is_some() {
        return Ok(());
    }

    // Step 8
    Err(())
}

fn has_credentials(url: &ServoUrl) -> bool {
    !url.username().is_empty() || url.password().is_some()
}

fn is_no_store_cache(headers: &Headers) -> bool {
    headers.has::<IfModifiedSince>() | headers.has::<IfNoneMatch>() |
    headers.has::<IfUnmodifiedSince>() | headers.has::<IfMatch>() |
    headers.has::<IfRange>()
}

fn response_needs_revalidation(_response: &Response) -> bool {
    // TODO this function
    false
}
