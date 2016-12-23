/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use body::{BodyOperations, BodyType, consume_body};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HeadersBinding::{HeadersInit, HeadersMethods};
use dom::bindings::codegen::Bindings::RequestBinding;
use dom::bindings::codegen::Bindings::RequestBinding::ReferrerPolicy;
use dom::bindings::codegen::Bindings::RequestBinding::RequestCache;
use dom::bindings::codegen::Bindings::RequestBinding::RequestCredentials;
use dom::bindings::codegen::Bindings::RequestBinding::RequestDestination;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInfo;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::RequestBinding::RequestMethods;
use dom::bindings::codegen::Bindings::RequestBinding::RequestMode;
use dom::bindings::codegen::Bindings::RequestBinding::RequestRedirect;
use dom::bindings::codegen::Bindings::RequestBinding::RequestType;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString, USVString};
use dom::globalscope::GlobalScope;
use dom::headers::{Guard, Headers};
use dom::promise::Promise;
use dom::xmlhttprequest::Extractable;
use hyper::method::Method as HttpMethod;
use net_traits::ReferrerPolicy as MsgReferrerPolicy;
use net_traits::request::{Origin, Window};
use net_traits::request::CacheMode as NetTraitsRequestCache;
use net_traits::request::CredentialsMode as NetTraitsRequestCredentials;
use net_traits::request::Destination as NetTraitsRequestDestination;
use net_traits::request::RedirectMode as NetTraitsRequestRedirect;
use net_traits::request::Referrer as NetTraitsRequestReferrer;
use net_traits::request::Request as NetTraitsRequest;
use net_traits::request::RequestMode as NetTraitsRequestMode;
use net_traits::request::Type as NetTraitsRequestType;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::cell::{Cell, Ref};
use std::rc::Rc;

#[dom_struct]
pub struct Request {
    reflector_: Reflector,
    request: DOMRefCell<NetTraitsRequest>,
    body_used: Cell<bool>,
    headers: MutNullableJS<Headers>,
    mime_type: DOMRefCell<Vec<u8>>,
    #[ignore_heap_size_of = "Rc"]
    body_promise: DOMRefCell<Option<(Rc<Promise>, BodyType)>>,
}

impl Request {
    fn new_inherited(global: &GlobalScope,
                     url: ServoUrl,
                     is_service_worker_global_scope: bool) -> Request {
        Request {
            reflector_: Reflector::new(),
            request: DOMRefCell::new(
                net_request_from_global(global,
                                        url,
                                        is_service_worker_global_scope)),
            body_used: Cell::new(false),
            headers: Default::default(),
            mime_type: DOMRefCell::new("".to_string().into_bytes()),
            body_promise: DOMRefCell::new(None),
        }
    }

    pub fn new(global: &GlobalScope,
               url: ServoUrl,
               is_service_worker_global_scope: bool) -> Root<Request> {
        reflect_dom_object(box Request::new_inherited(global,
                                                      url,
                                                      is_service_worker_global_scope),
                           global, RequestBinding::Wrap)
    }

    // https://fetch.spec.whatwg.org/#dom-request
    pub fn Constructor(global: &GlobalScope,
                       input: RequestInfo,
                       init: &RequestInit)
                       -> Fallible<Root<Request>> {
        // Step 1
        let temporary_request: NetTraitsRequest;

        // Step 2
        let mut fallback_mode: Option<NetTraitsRequestMode> = None;

        // Step 3
        let mut fallback_credentials: Option<NetTraitsRequestCredentials> = None;

        // Step 4
        let base_url = global.api_base_url();

        match input {
            // Step 5
            RequestInfo::USVString(USVString(ref usv_string)) => {
                // Step 5.1
                let parsed_url = base_url.join(&usv_string);
                // Step 5.2
                if parsed_url.is_err() {
                    return Err(Error::Type("Url could not be parsed".to_string()))
                }
                // Step 5.3
                let url = parsed_url.unwrap();
                if includes_credentials(&url) {
                    return Err(Error::Type("Url includes credentials".to_string()))
                }
                // Step 5.4
                temporary_request = net_request_from_global(global,
                                                            url,
                                                            false);
                // Step 5.5
                fallback_mode = Some(NetTraitsRequestMode::CorsMode);
                // Step 5.6
                fallback_credentials = Some(NetTraitsRequestCredentials::Omit);
            }
            // Step 6
            RequestInfo::Request(ref input_request) => {
                // Step 6.1
                if request_is_disturbed(input_request) || request_is_locked(input_request) {
                    return Err(Error::Type("Input is disturbed or locked".to_string()))
                }
                // Step 6.2
                temporary_request = input_request.request.borrow().clone();
            }
        }

        // Step 7
        // TODO: `entry settings object` is not implemented yet.
        let origin = base_url.origin();

        // Step 8
        let mut window = Window::Client;

        // Step 9
        // TODO: `environment settings object` is not implemented in Servo yet.

        // Step 10
        if !init.window.is_undefined() && !init.window.is_null() {
            return Err(Error::Type("Window is present and is not null".to_string()))
        }

        // Step 11
        if !init.window.is_undefined() {
            window = Window::NoWindow;
        }

        // Step 12
        let mut request: NetTraitsRequest;
        request = net_request_from_global(global,
                                          temporary_request.current_url(),
                                          false);
        request.method = temporary_request.method;
        request.headers = temporary_request.headers.clone();
        request.unsafe_request = true;
        request.window.set(window);
        // TODO: `entry settings object` is not implemented in Servo yet.
        *request.origin.borrow_mut() = Origin::Client;
        request.omit_origin_header = temporary_request.omit_origin_header;
        request.referrer = temporary_request.referrer;
        request.referrer_policy = temporary_request.referrer_policy;
        request.mode = temporary_request.mode;
        request.credentials_mode = temporary_request.credentials_mode;
        request.cache_mode = temporary_request.cache_mode;
        request.redirect_mode = temporary_request.redirect_mode;
        request.integrity_metadata = temporary_request.integrity_metadata;

        // Step 13
        if init.body.is_some() ||
            init.cache.is_some() ||
            init.credentials.is_some() ||
            init.integrity.is_some() ||
            init.headers.is_some() ||
            init.method.is_some() ||
            init.mode.is_some() ||
            init.redirect.is_some() ||
            init.referrer.is_some() ||
            init.referrerPolicy.is_some() ||
            !init.window.is_undefined() {
                // Step 13.1
                if request.mode == NetTraitsRequestMode::Navigate {
                    return Err(Error::Type(
                        "Init is present and request mode is 'navigate'".to_string()));
                    }
                // Step 13.2
                request.omit_origin_header.set(false);
                // Step 13.3
                *request.referrer.borrow_mut() = NetTraitsRequestReferrer::Client;
                // Step 13.4
                request.referrer_policy.set(None);
            }

        // Step 14
        if let Some(init_referrer) = init.referrer.as_ref() {
            // Step 14.1
            let ref referrer = init_referrer.0;
            // Step 14.2
            if referrer.is_empty() {
                *request.referrer.borrow_mut() = NetTraitsRequestReferrer::NoReferrer;
            } else {
                // Step 14.3
                let parsed_referrer = base_url.join(referrer);
                // Step 14.4
                if parsed_referrer.is_err() {
                    return Err(Error::Type(
                        "Failed to parse referrer url".to_string()));
                }
                // Step 14.5
                if let Ok(parsed_referrer) = parsed_referrer {
                    if parsed_referrer.cannot_be_a_base() &&
                        parsed_referrer.scheme() == "about" &&
                        parsed_referrer.path() == "client" {
                            *request.referrer.borrow_mut() = NetTraitsRequestReferrer::Client;
                        } else {
                            // Step 14.6
                            if parsed_referrer.origin() != origin {
                                return Err(Error::Type(
                                    "RequestInit's referrer has invalid origin".to_string()));
                            }
                            // Step 14.7
                            *request.referrer.borrow_mut() = NetTraitsRequestReferrer::ReferrerUrl(parsed_referrer);
                        }
                }
            }
        }

        // Step 15
        if let Some(init_referrerpolicy) = init.referrerPolicy.as_ref() {
            let init_referrer_policy = init_referrerpolicy.clone().into();
            request.referrer_policy.set(Some(init_referrer_policy));
        }

        // Step 16
        let mode = init.mode.as_ref().map(|m| m.clone().into()).or(fallback_mode);

        // Step 17
        if let Some(NetTraitsRequestMode::Navigate) = mode {
            return Err(Error::Type("Request mode is Navigate".to_string()));
        }

        // Step 18
        if let Some(m) = mode {
            request.mode = m;
        }

        // Step 19
        let credentials = init.credentials.as_ref().map(|m| m.clone().into()).or(fallback_credentials);

        // Step 20
        if let Some(c) = credentials {
            request.credentials_mode = c;
        }

        // Step 21
        if let Some(init_cache) = init.cache.as_ref() {
            let cache = init_cache.clone().into();
            request.cache_mode.set(cache);
        }

        // Step 22
        if request.cache_mode.get() == NetTraitsRequestCache::OnlyIfCached {
            if request.mode != NetTraitsRequestMode::SameOrigin {
                return Err(Error::Type(
                    "Cache is 'only-if-cached' and mode is not 'same-origin'".to_string()));
            }
        }

        // Step 23
        if let Some(init_redirect) = init.redirect.as_ref() {
            let redirect = init_redirect.clone().into();
            request.redirect_mode.set(redirect);
        }

        // Step 24
        if let Some(init_integrity) = init.integrity.as_ref() {
            let integrity = init_integrity.clone().to_string();
            *request.integrity_metadata.borrow_mut() = integrity;
        }

        // Step 25
        if let Some(init_method) = init.method.as_ref() {
            // Step 25.1
            if !is_method(&init_method) {
                return Err(Error::Type("Method is not a method".to_string()));
            }
            if is_forbidden_method(&init_method) {
                return Err(Error::Type("Method is forbidden".to_string()));
            }
            // Step 25.2
            let method = match init_method.as_str() {
                Some(s) => normalize_method(s),
                None => return Err(Error::Type("Method is not a valid UTF8".to_string())),
            };
            // Step 25.3
            *request.method.borrow_mut() = method;
        }

        // Step 26
        let r = Request::from_net_request(global,
                                          false,
                                          request);
        r.headers.or_init(|| Headers::for_request(&r.global()));

        // Step 27
        let mut headers_copy = r.Headers();

        // Step 28
        if let Some(possible_header) = init.headers.as_ref() {
            match possible_header {
                &HeadersInit::Headers(ref init_headers) => {
                    headers_copy = init_headers.clone();
                }
                &HeadersInit::ByteStringSequenceSequence(ref init_sequence) => {
                    try!(headers_copy.fill(Some(
                        HeadersInit::ByteStringSequenceSequence(init_sequence.clone()))));
                },
                &HeadersInit::ByteStringMozMap(ref init_map) => {
                    try!(headers_copy.fill(Some(
                        HeadersInit::ByteStringMozMap(init_map.clone()))));
                },
            }
        }

        // Step 29
        // We cannot empty `r.Headers().header_list` because
        // we would undo the Step 27 above.  One alternative is to set
        // `headers_copy` as a deep copy of `r.Headers()`. However,
        // `r.Headers()` is a `Root<T>`, and therefore it is difficult
        // to obtain a mutable reference to `r.Headers()`. Without the
        // mutable reference, we cannot mutate `r.Headers()` to be the
        // deep copied headers in Step 27.

        // Step 30
        if r.request.borrow().mode == NetTraitsRequestMode::NoCors {
            let borrowed_request = r.request.borrow();
            // Step 30.1
            if !is_cors_safelisted_method(&borrowed_request.method.borrow()) {
                return Err(Error::Type(
                    "The mode is 'no-cors' but the method is not a cors-safelisted method".to_string()));
            }
            // Step 30.2
            if !borrowed_request.integrity_metadata.borrow().is_empty() {
                return Err(Error::Type("Integrity metadata is not an empty string".to_string()));
            }
            // Step 30.3
            r.Headers().set_guard(Guard::RequestNoCors);
        }

        // Step 31
        match init.headers {
            None => {
                // This is equivalent to the specification's concept of
                // "associated headers list". If an init headers is not given,
                // but an input with headers is given, set request's
                // headers as the input's Headers.
                if let RequestInfo::Request(ref input_request) = input {
                    try!(r.Headers().fill(Some(HeadersInit::Headers(input_request.Headers()))));
                }
            },
            Some(HeadersInit::Headers(_)) => try!(r.Headers().fill(Some(HeadersInit::Headers(headers_copy)))),
            _ => {},
        }

        // Step 32
        let mut input_body = if let RequestInfo::Request(ref input_request) = input {
            let input_request_request = input_request.request.borrow();
            let body = input_request_request.body.borrow();
            body.clone()
        } else {
            None
        };

        // Step 33
        if let Some(init_body_option) = init.body.as_ref() {
            if init_body_option.is_some() || input_body.is_some() {
                let req = r.request.borrow();
                let req_method = req.method.borrow();
                match &*req_method {
                    &HttpMethod::Get => return Err(Error::Type(
                        "Init's body is non-null, and request method is GET".to_string())),
                    &HttpMethod::Head => return Err(Error::Type(
                        "Init's body is non-null, and request method is HEAD".to_string())),
                    _ => {},
                }
            }
        }

        // Step 34
        if let Some(Some(ref init_body)) = init.body {
            // Step 34.2
            let extracted_body_tmp = init_body.extract();
            input_body = Some(extracted_body_tmp.0);
            let content_type = extracted_body_tmp.1;

            // Step 34.3
            if let Some(contents) = content_type {
                if !r.Headers().Has(ByteString::new(b"Content-Type".to_vec())).unwrap() {
                    try!(r.Headers().Append(ByteString::new(b"Content-Type".to_vec()),
                                            ByteString::new(contents.as_bytes().to_vec())));
                }
            }
        }

        // Step 35
        {
            let borrowed_request = r.request.borrow();
            *borrowed_request.body.borrow_mut() = input_body;
        }

        // Step 36
        let extracted_mime_type = r.Headers().extract_mime_type();
        *r.mime_type.borrow_mut() = extracted_mime_type;

        // Step 37
        // TODO: `ReadableStream` object is not implemented in Servo yet.

        // Step 38
        Ok(r)
    }

    // https://fetch.spec.whatwg.org/#concept-body-locked
    fn locked(&self) -> bool {
        // TODO: ReadableStream is unimplemented. Just return false
        // for now.
        false
    }
}

impl Request {
    fn from_net_request(global: &GlobalScope,
                        is_service_worker_global_scope: bool,
                        net_request: NetTraitsRequest) -> Root<Request> {
        let r = Request::new(global,
                             net_request.current_url(),
                             is_service_worker_global_scope);
        *r.request.borrow_mut() = net_request;
        r
    }

    fn clone_from(r: &Request) -> Fallible<Root<Request>> {
        let req = r.request.borrow();
        let url = req.url();
        let is_service_worker_global_scope = req.is_service_worker_global_scope;
        let body_used = r.body_used.get();
        let mime_type = r.mime_type.borrow().clone();
        let headers_guard = r.Headers().get_guard();
        let r_clone = Request::new(&r.global(), url, is_service_worker_global_scope);
        r_clone.request.borrow_mut().pipeline_id.set(req.pipeline_id.get());
        {
            let mut borrowed_r_request = r_clone.request.borrow_mut();
            *borrowed_r_request.origin.borrow_mut() = req.origin.borrow().clone();
        }
        *r_clone.request.borrow_mut() = req.clone();
        r_clone.body_used.set(body_used);
        *r_clone.mime_type.borrow_mut() = mime_type;
        try!(r_clone.Headers().fill(Some(HeadersInit::Headers(r.Headers()))));
        r_clone.Headers().set_guard(headers_guard);
        Ok(r_clone)
    }

    pub fn get_request(&self) -> NetTraitsRequest {
        self.request.borrow().clone()
    }
}

fn net_request_from_global(global: &GlobalScope,
                           url: ServoUrl,
                           is_service_worker_global_scope: bool) -> NetTraitsRequest {
    let origin = Origin::Origin(global.get_url().origin());
    let pipeline_id = global.pipeline_id();
    NetTraitsRequest::new(url,
                          Some(origin),
                          is_service_worker_global_scope,
                          Some(pipeline_id))
}

// https://fetch.spec.whatwg.org/#concept-method-normalize
fn normalize_method(m: &str) -> HttpMethod {
    match m {
        m if m.eq_ignore_ascii_case("DELETE") => HttpMethod::Delete,
        m if m.eq_ignore_ascii_case("GET") => HttpMethod::Get,
        m if m.eq_ignore_ascii_case("HEAD") => HttpMethod::Head,
        m if m.eq_ignore_ascii_case("OPTIONS") => HttpMethod::Options,
        m if m.eq_ignore_ascii_case("POST") => HttpMethod::Post,
        m if m.eq_ignore_ascii_case("PUT") => HttpMethod::Put,
        m => HttpMethod::Extension(m.to_string()),
    }
}

// https://fetch.spec.whatwg.org/#concept-method
fn is_method(m: &ByteString) -> bool {
    match m.to_lower().as_str() {
        Some("get") => true,
        Some("head") => true,
        Some("post") => true,
        Some("put") => true,
        Some("delete") => true,
        Some("connect") => true,
        Some("options") => true,
        Some("trace") => true,
        _ => false,
    }
}

// https://fetch.spec.whatwg.org/#forbidden-method
fn is_forbidden_method(m: &ByteString) -> bool {
    match m.to_lower().as_str() {
        Some("connect") => true,
        Some("trace") => true,
        Some("track") => true,
        _ => false,
    }
}

// https://fetch.spec.whatwg.org/#cors-safelisted-method
fn is_cors_safelisted_method(m: &HttpMethod) -> bool {
    m == &HttpMethod::Get ||
        m == &HttpMethod::Head ||
        m == &HttpMethod::Post
}

// https://url.spec.whatwg.org/#include-credentials
fn includes_credentials(input: &ServoUrl) -> bool {
    !input.username().is_empty() || input.password().is_some()
}

// TODO: `Readable Stream` object is not implemented in Servo yet.
// https://fetch.spec.whatwg.org/#concept-body-disturbed
fn request_is_disturbed(_input: &Request) -> bool {
    false
}

// TODO: `Readable Stream` object is not implemented in Servo yet.
// https://fetch.spec.whatwg.org/#concept-body-locked
fn request_is_locked(_input: &Request) -> bool {
    false
}

impl RequestMethods for Request {
    // https://fetch.spec.whatwg.org/#dom-request-method
    fn Method(&self) -> ByteString {
        let r = self.request.borrow();
        let m = r.method.borrow();
        ByteString::new(m.as_ref().as_bytes().into())
    }

    // https://fetch.spec.whatwg.org/#dom-request-url
    fn Url(&self) -> USVString {
        let r = self.request.borrow();
        let url = r.url_list.borrow();
        USVString(url.get(0).map_or("", |u| u.as_str()).into())
    }

    // https://fetch.spec.whatwg.org/#dom-request-headers
    fn Headers(&self) -> Root<Headers> {
        self.headers.or_init(|| Headers::new(&self.global()))
    }

    // https://fetch.spec.whatwg.org/#dom-request-type
    fn Type(&self) -> RequestType {
        self.request.borrow().type_.into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-destination
    fn Destination(&self) -> RequestDestination {
        self.request.borrow().destination.into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-referrer
    fn Referrer(&self) -> USVString {
        let r = self.request.borrow();
        let referrer = r.referrer.borrow();
        USVString(match &*referrer {
            &NetTraitsRequestReferrer::NoReferrer => String::from("no-referrer"),
            &NetTraitsRequestReferrer::Client => String::from("client"),
            &NetTraitsRequestReferrer::ReferrerUrl(ref u) => {
                let u_c = u.clone();
                u_c.into_string()
            }
        })
    }

    // https://fetch.spec.whatwg.org/#dom-request-referrerpolicy
    fn ReferrerPolicy(&self) -> ReferrerPolicy {
        self.request.borrow().referrer_policy.get().map(|m| m.into()).unwrap_or(ReferrerPolicy::_empty)
    }

    // https://fetch.spec.whatwg.org/#dom-request-mode
    fn Mode(&self) -> RequestMode {
        self.request.borrow().mode.into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-credentials
    fn Credentials(&self) -> RequestCredentials {
        let r = self.request.borrow().clone();
        r.credentials_mode.into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-cache
    fn Cache(&self) -> RequestCache {
        let r = self.request.borrow().clone();
        r.cache_mode.get().into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-redirect
    fn Redirect(&self) -> RequestRedirect {
        let r = self.request.borrow().clone();
        r.redirect_mode.get().into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-integrity
    fn Integrity(&self) -> DOMString {
        let r = self.request.borrow();
        let integrity = r.integrity_metadata.borrow();
        DOMString::from_string(integrity.clone())
    }

    // https://fetch.spec.whatwg.org/#dom-body-bodyused
    fn BodyUsed(&self) -> bool {
        self.body_used.get()
    }

    // https://fetch.spec.whatwg.org/#dom-request-clone
    fn Clone(&self) -> Fallible<Root<Request>> {
        // Step 1
        if request_is_locked(self) {
            return Err(Error::Type("Request is locked".to_string()));
        }
        if request_is_disturbed(self) {
            return Err(Error::Type("Request is disturbed".to_string()));
        }

        // Step 2
        Request::clone_from(self)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-text
    fn Text(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Text)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-blob
    fn Blob(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Blob)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-formdata
    fn FormData(&self) -> Rc<Promise> {
        consume_body(self, BodyType::FormData)
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#dom-body-json
    fn Json(&self) -> Rc<Promise> {
        consume_body(self, BodyType::Json)
    }
}

impl BodyOperations for Request {
    fn get_body_used(&self) -> bool {
        self.BodyUsed()
    }

    fn set_body_promise(&self, p: &Rc<Promise>, body_type: BodyType) {
        assert!(self.body_promise.borrow().is_none());
        self.body_used.set(true);
        *self.body_promise.borrow_mut() = Some((p.clone(), body_type));
    }

    fn is_locked(&self) -> bool {
        self.locked()
    }

    fn take_body(&self) -> Option<Vec<u8>> {
        let request = self.request.borrow_mut();
        let body = request.body.borrow_mut().take();
        Some(body.unwrap_or(vec![]))
    }

    fn get_mime_type(&self) -> Ref<Vec<u8>> {
        self.mime_type.borrow()
    }
}

impl Into<NetTraitsRequestCache> for RequestCache {
    fn into(self) -> NetTraitsRequestCache {
        match self {
            RequestCache::Default => NetTraitsRequestCache::Default,
            RequestCache::No_store => NetTraitsRequestCache::NoStore,
            RequestCache::Reload => NetTraitsRequestCache::Reload,
            RequestCache::No_cache => NetTraitsRequestCache::NoCache,
            RequestCache::Force_cache => NetTraitsRequestCache::ForceCache,
            RequestCache::Only_if_cached => NetTraitsRequestCache::OnlyIfCached,
        }
    }
}

impl Into<RequestCache> for NetTraitsRequestCache {
    fn into(self) -> RequestCache {
        match self {
            NetTraitsRequestCache::Default => RequestCache::Default,
            NetTraitsRequestCache::NoStore => RequestCache::No_store,
            NetTraitsRequestCache::Reload => RequestCache::Reload,
            NetTraitsRequestCache::NoCache => RequestCache::No_cache,
            NetTraitsRequestCache::ForceCache => RequestCache::Force_cache,
            NetTraitsRequestCache::OnlyIfCached => RequestCache::Only_if_cached,
        }
    }
}

impl Into<NetTraitsRequestCredentials> for RequestCredentials {
    fn into(self) -> NetTraitsRequestCredentials {
        match self {
            RequestCredentials::Omit => NetTraitsRequestCredentials::Omit,
            RequestCredentials::Same_origin => NetTraitsRequestCredentials::CredentialsSameOrigin,
            RequestCredentials::Include => NetTraitsRequestCredentials::Include,
        }
    }
}

impl Into<RequestCredentials> for NetTraitsRequestCredentials {
    fn into(self) -> RequestCredentials {
        match self {
            NetTraitsRequestCredentials::Omit => RequestCredentials::Omit,
            NetTraitsRequestCredentials::CredentialsSameOrigin => RequestCredentials::Same_origin,
            NetTraitsRequestCredentials::Include => RequestCredentials::Include,
        }
    }
}

impl Into<NetTraitsRequestDestination> for RequestDestination {
    fn into(self) -> NetTraitsRequestDestination {
        match self {
            RequestDestination::_empty => NetTraitsRequestDestination::None,
            RequestDestination::Document => NetTraitsRequestDestination::Document,
            RequestDestination::Embed => NetTraitsRequestDestination::Embed,
            RequestDestination::Font => NetTraitsRequestDestination::Font,
            RequestDestination::Image => NetTraitsRequestDestination::Image,
            RequestDestination::Manifest => NetTraitsRequestDestination::Manifest,
            RequestDestination::Media => NetTraitsRequestDestination::Media,
            RequestDestination::Object => NetTraitsRequestDestination::Object,
            RequestDestination::Report => NetTraitsRequestDestination::Report,
            RequestDestination::Script => NetTraitsRequestDestination::Script,
            RequestDestination::Serviceworker => NetTraitsRequestDestination::ServiceWorker,
            RequestDestination::Sharedworker => NetTraitsRequestDestination::SharedWorker,
            RequestDestination::Style => NetTraitsRequestDestination::Style,
            RequestDestination::Worker => NetTraitsRequestDestination::Worker,
            RequestDestination::Xslt => NetTraitsRequestDestination::XSLT,
        }
    }
}

impl Into<RequestDestination> for NetTraitsRequestDestination {
    fn into(self) -> RequestDestination {
        match self {
            NetTraitsRequestDestination::None => RequestDestination::_empty,
            NetTraitsRequestDestination::Document => RequestDestination::Document,
            NetTraitsRequestDestination::Embed => RequestDestination::Embed,
            NetTraitsRequestDestination::Font => RequestDestination::Font,
            NetTraitsRequestDestination::Image => RequestDestination::Image,
            NetTraitsRequestDestination::Manifest => RequestDestination::Manifest,
            NetTraitsRequestDestination::Media => RequestDestination::Media,
            NetTraitsRequestDestination::Object => RequestDestination::Object,
            NetTraitsRequestDestination::Report => RequestDestination::Report,
            NetTraitsRequestDestination::Script => RequestDestination::Script,
            NetTraitsRequestDestination::ServiceWorker => RequestDestination::Serviceworker,
            NetTraitsRequestDestination::SharedWorker => RequestDestination::Sharedworker,
            NetTraitsRequestDestination::Style => RequestDestination::Style,
            NetTraitsRequestDestination::XSLT => RequestDestination::Xslt,
            NetTraitsRequestDestination::Worker => RequestDestination::Worker,
        }
    }
}

impl Into<NetTraitsRequestType> for RequestType {
    fn into(self) -> NetTraitsRequestType {
        match self {
            RequestType::_empty => NetTraitsRequestType::None,
            RequestType::Audio => NetTraitsRequestType::Audio,
            RequestType::Font => NetTraitsRequestType::Font,
            RequestType::Image => NetTraitsRequestType::Image,
            RequestType::Script => NetTraitsRequestType::Script,
            RequestType::Style => NetTraitsRequestType::Style,
            RequestType::Track => NetTraitsRequestType::Track,
            RequestType::Video => NetTraitsRequestType::Video,
        }
    }
}

impl Into<RequestType> for NetTraitsRequestType {
    fn into(self) -> RequestType {
        match self {
            NetTraitsRequestType::None => RequestType::_empty,
            NetTraitsRequestType::Audio => RequestType::Audio,
            NetTraitsRequestType::Font => RequestType::Font,
            NetTraitsRequestType::Image => RequestType::Image,
            NetTraitsRequestType::Script => RequestType::Script,
            NetTraitsRequestType::Style => RequestType::Style,
            NetTraitsRequestType::Track => RequestType::Track,
            NetTraitsRequestType::Video => RequestType::Video,
        }
    }
}

impl Into<NetTraitsRequestMode> for RequestMode {
    fn into(self) -> NetTraitsRequestMode {
        match self {
            RequestMode::Navigate => NetTraitsRequestMode::Navigate,
            RequestMode::Same_origin => NetTraitsRequestMode::SameOrigin,
            RequestMode::No_cors => NetTraitsRequestMode::NoCors,
            RequestMode::Cors => NetTraitsRequestMode::CorsMode,
        }
    }
}

impl Into<RequestMode> for NetTraitsRequestMode {
    fn into(self) -> RequestMode {
        match self {
            NetTraitsRequestMode::Navigate => RequestMode::Navigate,
            NetTraitsRequestMode::SameOrigin => RequestMode::Same_origin,
            NetTraitsRequestMode::NoCors => RequestMode::No_cors,
            NetTraitsRequestMode::CorsMode => RequestMode::Cors,
        }
    }
}

// TODO
// When whatwg/fetch PR #346 is merged, fix this.
impl Into<MsgReferrerPolicy> for ReferrerPolicy {
    fn into(self) -> MsgReferrerPolicy {
        match self {
            ReferrerPolicy::_empty => MsgReferrerPolicy::NoReferrer,
            ReferrerPolicy::No_referrer => MsgReferrerPolicy::NoReferrer,
            ReferrerPolicy::No_referrer_when_downgrade =>
                MsgReferrerPolicy::NoReferrerWhenDowngrade,
            ReferrerPolicy::Origin => MsgReferrerPolicy::Origin,
            ReferrerPolicy::Origin_when_cross_origin => MsgReferrerPolicy::OriginWhenCrossOrigin,
            ReferrerPolicy::Unsafe_url => MsgReferrerPolicy::UnsafeUrl,
            ReferrerPolicy::Strict_origin => MsgReferrerPolicy::StrictOrigin,
            ReferrerPolicy::Strict_origin_when_cross_origin =>
                MsgReferrerPolicy::StrictOriginWhenCrossOrigin,
        }
    }
}

impl Into<ReferrerPolicy> for MsgReferrerPolicy {
    fn into(self) -> ReferrerPolicy {
        match self {
            MsgReferrerPolicy::NoReferrer => ReferrerPolicy::No_referrer,
            MsgReferrerPolicy::NoReferrerWhenDowngrade =>
                ReferrerPolicy::No_referrer_when_downgrade,
            MsgReferrerPolicy::Origin => ReferrerPolicy::Origin,
            MsgReferrerPolicy::SameOrigin => ReferrerPolicy::Origin,
            MsgReferrerPolicy::OriginWhenCrossOrigin => ReferrerPolicy::Origin_when_cross_origin,
            MsgReferrerPolicy::UnsafeUrl => ReferrerPolicy::Unsafe_url,
            MsgReferrerPolicy::StrictOrigin => ReferrerPolicy::Strict_origin,
            MsgReferrerPolicy::StrictOriginWhenCrossOrigin =>
                ReferrerPolicy::Strict_origin_when_cross_origin,
        }
    }
}

impl Into<NetTraitsRequestRedirect> for RequestRedirect {
    fn into(self) -> NetTraitsRequestRedirect {
        match self {
            RequestRedirect::Follow => NetTraitsRequestRedirect::Follow,
            RequestRedirect::Error => NetTraitsRequestRedirect::Error,
            RequestRedirect::Manual => NetTraitsRequestRedirect::Manual,
        }
    }
}

impl Into<RequestRedirect> for NetTraitsRequestRedirect {
    fn into(self) -> RequestRedirect {
        match self {
            NetTraitsRequestRedirect::Follow => RequestRedirect::Follow,
            NetTraitsRequestRedirect::Error => RequestRedirect::Error,
            NetTraitsRequestRedirect::Manual => RequestRedirect::Manual,
        }
    }
}

impl Clone for HeadersInit {
    fn clone(&self) -> HeadersInit {
    match self {
        &HeadersInit::Headers(ref h) =>
            HeadersInit::Headers(h.clone()),
        &HeadersInit::ByteStringSequenceSequence(ref b) =>
            HeadersInit::ByteStringSequenceSequence(b.clone()),
        &HeadersInit::ByteStringMozMap(ref m) =>
            HeadersInit::ByteStringMozMap(m.clone()),
        }
    }
}
