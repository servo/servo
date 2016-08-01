/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::headers::{Headers, Guard};
use dom::bindings::reflector::Reflectable;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::UnionTypes::{HeadersOrByteStringSequenceSequence, RequestOrUSVString};
use dom::bindings::codegen::Bindings::RequestBinding;
use dom::bindings::codegen::Bindings::RequestBinding::RequestCache;
use dom::bindings::codegen::Bindings::RequestBinding::RequestCredentials;
use dom::bindings::codegen::Bindings::RequestBinding::RequestDestination;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInfo;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::RequestBinding::RequestMethods;
use dom::bindings::codegen::Bindings::RequestBinding::RequestMode;
use dom::bindings::codegen::Bindings::RequestBinding::ReferrerPolicy;
use dom::bindings::codegen::Bindings::RequestBinding::RequestRedirect;
use dom::bindings::codegen::Bindings::RequestBinding::RequestType;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, USVString, DOMString};
use hyper;
use msg;
use net_traits::request as NetTraitsRequest;
use std::cell::{Cell, Ref, RefCell};
use url;

#[dom_struct]
pub struct Request {
    reflector_: Reflector,
    #[ignore_heap_size_of = "net_traits is missing HeapSizeOf implementation"]
    request: DOMRefCell<NetTraitsRequest::Request>,
    body_used: DOMRefCell<bool>,
    headers_reflector: MutNullableHeap<JS<Headers>>,
}

impl Request {
    pub fn new_inherited(url: url::Url,
                         origin: Option<NetTraitsRequest::Origin>,
                         is_service_worker_global_scope: bool,
                         pipeline_id: Option<msg::constellation_msg::PipelineId>,
                         body_used: bool) -> Request {
        Request {
            reflector_: Reflector::new(),
            request: DOMRefCell::new(
                NetTraitsRequest::Request::new(url,
                                               origin,
                                               is_service_worker_global_scope,
                                               pipeline_id)),
            body_used: DOMRefCell::new(body_used),
            headers_reflector: Default::default(),
        }
    }

    pub fn new(global: GlobalRef,
               url: url::Url,
               origin: Option<NetTraitsRequest::Origin>,
               is_service_worker_global_scope: bool,
               pipeline_id: Option<msg::constellation_msg::PipelineId>,
               body_used: bool) -> Root<Request> {
        reflect_dom_object(box Request::new_inherited(url,
                                                      origin,
                                                      is_service_worker_global_scope,
                                                      pipeline_id,
                                                      body_used),
                           global, RequestBinding::Wrap)
    }

    // https://fetch.spec.whatwg.org/#dom-request
    pub fn Constructor(global: GlobalRef,
                       input: RequestInfo,
                       init: &RequestInit)
                      -> Fallible<Root<Request>> {

        let mut request =
            NetTraitsRequest::Request::new(url::Url::parse("").unwrap(),
                                           None,
                                           false,
                                           None);

        // Step 1
        if is_request(&input) &&
            (requestinfo_is_disturbed(&input) || requestinfo_is_locked(&input)) {
                return Err(Error::Type("Input is disturbed or locked".to_string()))
            }

        // Step 2
        let mut temporary_request =
            NetTraitsRequest::Request::new(url::Url::parse("").unwrap(),
                                           None,
                                           false,
                                           None);
        if let &RequestOrUSVString::Request(ref req) = &input {
            temporary_request = req.request.borrow().clone();
        }

        // TODO: entry settings object origin is not implemented yet.
        // Step 3
        // if you need an origin at some point, you can try using `global.url().origin()`
        // let origin = "entry settings object origin";

        // Step 4
        let mut window = Cell::new(NetTraitsRequest::Window::Client);

        // TODO: environment settings object is not implemented in Servo yet.
        // Step 5
        // if temporary_request.window == "environment settings object"
        //     && temporary_request.origin == origin {
        //         window = request.window;
        // }

        // Step 6
        if !init.window.is_undefined() && !init.window.is_null() {
            return Err(Error::Type("Window is present and is not null".to_string()))
        }

        // Step 7
        if !init.window.is_undefined() {
            window.set(NetTraitsRequest::Window::NoWindow);
        }

        // Step 8
        if let Some(url) = get_current_url(&temporary_request) {
            request.url_list = RefCell::new(vec![url.clone()]);
        }
        request.method = temporary_request.method;
        request.headers = temporary_request.headers.clone();
        request.unsafe_request = true;
        request.window = window;
        // TODO: client is not implemented in Servo yet.
        // new_request's client = entry settings object
        request.origin = RefCell::new(NetTraitsRequest::Origin::Client);
        request.omit_origin_header = temporary_request.omit_origin_header;
        request.same_origin_data = Cell::new(true);
        request.referer = temporary_request.referer;
        request.referrer_policy = temporary_request.referrer_policy;
        request.mode = temporary_request.mode;
        request.credentials_mode = temporary_request.credentials_mode;
        request.cache_mode = temporary_request.cache_mode;
        request.redirect_mode = temporary_request.redirect_mode;
        request.integrity_metadata = temporary_request.integrity_metadata;

        // Step 9
        let mut fallback_mode: Option<NetTraitsRequest::RequestMode>;
        fallback_mode = None;

        // Step 10
        let mut fallback_credentials: Option<NetTraitsRequest::CredentialsMode>;
        fallback_credentials = None;

        // Step 11
        // TODO: entry settings object is not implemented in Servo yet.
        // let base_url = entry settings object's API base URL

        // Step 12
        if let &RequestOrUSVString::USVString(USVString(ref usv_string)) = &input {
            // Step 12.1
            // TODO: will have to use url::Url::join with base_url as base_url.
            let parsed_url = url::Url::parse(&usv_string);
            // Step 12.2
            if let &Err(_) = &parsed_url {
                return Err(Error::Type("Url could not be parsed".to_string()))
            }
            // Step 12.3
            if includes_credentials(&parsed_url) {
                return Err(Error::Type("Url includes credentials".to_string()))
            }
            // Step 12.4
            if let Ok(url) = parsed_url {
                request.url_list = RefCell::new(vec![url]);
            }
            // Step 12.5
            fallback_mode = Some(NetTraitsRequest::RequestMode::CORSMode);
            // Step 12.6
            fallback_credentials = Some(NetTraitsRequest::CredentialsMode::Omit);
        }

        // Step 13
        if init.body.is_some() ||
            init.cache.is_some() ||
            init.credentials.is_some() ||
            init.integrity.is_some() ||
            init.method.is_some() ||
            init.mode.is_some() ||
            init.redirect.is_some() ||
            init.referrer.is_some() ||
            init.referrerPolicy.is_some() ||
            !init.window.is_undefined() {
                // Step 13.1
                if let NetTraitsRequest::RequestMode::Navigate
                    = request.mode {
                        return Err(Error::Type(
                            "Init is present and request mode is 'navigate'".to_string()));
                    }
                // Step 13.2
                request.omit_origin_header = Cell::new(false);
                // Step 13.3
                request.referer = RefCell::new(NetTraitsRequest::Referer::Client);
                // Step 13.4
                request.referrer_policy = Cell::new(None);
            }

        // Step 14
        if let Some(init_referrer) = init.referrer.as_ref() {
            let parsed_referrer: url::Url;
            // Step 14.1
            let referrer: String = init_referrer.0.clone();
            // Step 14.2
            if referrer.is_empty() {
                request.referer = RefCell::new(NetTraitsRequest::Referer::NoReferer);
            } else {
                // Step 14.3
                // TODO: should use url::Url::join with baseURL
                let parsed_referrer = url::Url::parse(&referrer);
                // Step 14.4
                if let Err(_) = parsed_referrer {
                    return Err(Error::Type(
                        "Failed to parse referrer url".to_string()));
                }
                // Step 14.5
                // TODO: check if parsed_referrer Non-relative flag is set.
                if let Ok(parsed_referrer) = parsed_referrer {
                    if parsed_referrer.scheme() == "about" &&
                        parsed_referrer.path() == "client" {
                            request.referer =
                                RefCell::new(NetTraitsRequest::Referer::Client);
                        } else {
                            // Step 14.6
                            // TODO: origin is defined in Step 3.
                            // if parsed_referrer.origin() != origin {
                            //     return Err(Error::Type(
                            //         "Parsed referrer url's origin is not the same as origin".to_string()));
                            // } else {
                            // Step 14.7
                            request.referer =
                                RefCell::new(NetTraitsRequest::Referer::RefererUrl(parsed_referrer));
                        }
                }
            }
        }

        // Step 15
        if let Some(init_referrerpolicy) = init.referrerPolicy.as_ref() {
            let init_referrer_policy = init_referrerpolicy.clone().into();
            request.referrer_policy = Cell::new(Some(init_referrer_policy));
        }

        // Step 16
        let mut mode: Option<NetTraitsRequest::RequestMode>;
        mode = None;
        match init.mode.as_ref() {
            Some(init_mode) =>  mode = Some(init_mode.clone().into()),
            None => mode = Some(fallback_mode.unwrap()),
        }

        // Step 17
        if let Some(NetTraitsRequest::RequestMode::Navigate) = mode {
            return Err(Error::Type("Request mode is Navigate".to_string()));
        }

        // Step 18
        if mode.is_some() {
            request.mode = mode.unwrap();
        }

        // Step 19
        let mut credentials: Option<NetTraitsRequest::CredentialsMode>;
        match init.credentials.as_ref() {
            Some(init_credentials) =>  credentials = Some(init_credentials.clone().into()),
            None => credentials = Some(fallback_credentials.unwrap()),
        }

        // Step 20
        if credentials.is_some() {
            request.credentials_mode = credentials.unwrap();
        }

        // Step 21
        if let Some(init_cache) = init.cache.as_ref() {
            let cache = init_cache.clone().into();
            request.cache_mode = Cell::new(cache);
        }

        // Step 22
        if let NetTraitsRequest::CacheMode::OnlyIfCached = request.cache_mode.get() {
            match request.mode {
                NetTraitsRequest::RequestMode::SameOrigin => {},
                _ => return Err(Error::Type("Cache is 'only-if-cached' and mode is not 'same-origin'".to_string())),
            }
        }

        // Step 23
        if let Some(init_redirect) = init.redirect.as_ref() {
            let redirect = init_redirect.clone().into();
            request.redirect_mode = Cell::new(redirect);
        }

        // Step 24
        if let Some(init_integrity) = init.integrity.as_ref() {
            let integrity = init_integrity.clone().to_string();
            request.integrity_metadata = RefCell::new(integrity);
        }

        // Step 25
        if let Some(init_method) = init.method.as_ref() {
            let method: ByteString = init_method.clone();
            // Step 25.1
            if !is_method(&method) {
                return Err(Error::Type("Method is not a method".to_string()));
            }
            if is_forbidden_method(&method) {
                return Err(Error::Type("Method is forbidden".to_string()));
            }
            // TODO: normalized_method
            // Step 25.2
            let normalized_method = method.as_str().unwrap();
            // let normalized_method = normalize_method(method);
            // Step 25.3
            let hyper_method = from_normalized_method_to_method_enum(normalized_method);
            request.method = RefCell::new(hyper_method);
        }

        // Step 26
        let mut r = Request::new(global,
                                 url::Url::parse("").unwrap(),
                                 None,
                                 // expects NetTraitsRequest::Origin
                                 // global.get_url().origin() returns url::Origin
                                 false,
                                 None,
                                 false);
        *r.request.borrow_mut() = request;
        r.headers_reflector.or_init(|| Headers::new(r.global().r(), None).unwrap());
        r.headers_reflector.get().unwrap().set_guard(Guard::Request);

        // Step 27
        let mut headers = r.headers_reflector.get().clone();

        // Step 28
        // TODO: temp fix -> adding [#derive(Clone)] to generated code
        let mut headers_init: Option<HeadersOrByteStringSequenceSequence>;
        headers_init = None;
        if let Some(init_headers) = init.headers.as_ref() {
            // init_headers is &HeadersOrByteStringSequenceSequence
            // ... and &T is Clone for all types T
            // ... therefore init_headers can be cloned
            // ... but HeadersOrByteStringSequenceSequence is not Clone
            // ... so it cannot be cloned
            // ... so rust chooses to clone the &HeadersOrByteStringSequenceSequence reference
            // ... rather than doing the equivalent of std::clone::Clone(*init_heades)
            // ... and so headers is another &HeadersOrByteStringSequenceSequence now
            let headers = init_headers.clone();
            // ... and this is going to complain becuase headers_init is supposed to be Option<T> not Option<&T>
            // ... where T = HeadersOrByteStringSequenceSequence
            headers_init = Some(headers);
        }

        // Step 29
        r.headers_reflector.get().unwrap().empty_header_list();

        // Step 30
        if let NetTraitsRequest::RequestMode::NoCORS = r.request.borrow().mode {
            let method = r.request.borrow().method.clone();
            if !is_cors_safelisted_method(method.into_inner()) {
                return Err(Error::Type("The mode is 'no-cors' but the method is not a cors-safelisted method".to_string()));
            }
            let integrity_metadata = r.request.borrow().integrity_metadata.clone();
            if !integrity_metadata.into_inner().is_empty() {
                return Err(Error::Type("Integrity metadata is not an empty string".to_string()));
            }
            r.headers_reflector.get().unwrap().set_guard(Guard::RequestNoCors);
        }

        // Step 31
        if headers_init.is_some() {
            r.headers_reflector.set(Some(Headers::new(global, headers_init).unwrap().r()));
        }

        // Step 32
        let mut input_body: Option<Vec<u8>>;
        input_body = None;
        if let RequestInfo::Request(input_request) = input {
            let request_body = input_request.request.borrow().clone().body.into_inner();
            if request_body.is_some() {
                input_body = Some(request_body.unwrap());
            }
        }

        // Step 33
        unimplemented!();
    }
}

// TODO
fn from_normalized_method_to_method_enum(m: &str) -> hyper::method::Method {
    match m {
        "DELETE" => hyper::method::Method::Delete,
        "GET" => hyper::method::Method::Get,
        "HEAD" => hyper::method::Method::Head,
        "OPTIONS" => hyper::method::Method::Options,
        "POST" => hyper::method::Method::Post,
        "PUT" => hyper::method::Method::Put,
        a => hyper::method::Method::Extension(a.to_string())
    }
}

// TODO
// make it return all caps
// https://fetch.spec.whatwg.org/#concept-method-normalize
fn normalize_method(m: &ByteString) -> String {
    match m.to_lower().as_str() {
        Some("delete") => "DELETE".to_string(),
        Some("get") => "GET".to_string(),
        Some("head") => "HEAD".to_string(),
        Some("options") => "OPTIONS".to_string(),
        Some("post") => "POST".to_string(),
        Some("put") => "PUT".to_string(),
        Some(a) => a.to_string(),
        None => "NONE".to_string(),
    }
}

// TODO
// https://tools.ietf.org/html/rfc7230#section-3.1.1
fn is_method(m: &ByteString) -> bool {
    return true;
}

// TODO
    // https://fetch.spec.whatwg.org/#forbidden-method
fn is_forbidden_method(m: &ByteString) -> bool {
    return false;
}

// https://fetch.spec.whatwg.org/#concept-request-url
fn get_associated_url(req: &NetTraitsRequest::Request) -> Option<Ref<url::Url>> {
    let url_list = req.url_list.borrow();
    if url_list.len() > 0 {
        Some(Ref::map(url_list, |urls| urls.first().unwrap()))
        } else {
        None
    }
}

// https://fetch.spec.whatwg.org/#concept-request-current-url
fn get_current_url(req: &NetTraitsRequest::Request) -> Option<Ref<url::Url>> {
    let url_list = req.url_list.borrow();
    if url_list.len() > 0 {
        Some(Ref::map(url_list, |urls| urls.last().unwrap()))
        } else {
        None
    }
}

fn is_cors_safelisted_method(m: hyper::method::Method) -> bool {
    m == hyper::method::Method::Get ||
        m == hyper::method::Method::Head ||
        m == hyper::method::Method::Post
}

fn includes_credentials(input: &Result<url::Url, url::ParseError>) -> bool {
    return false;
}

fn is_request(input: &RequestInfo) -> bool {
    match input {
        &RequestInfo::Request(_) => true,
        _ => false,
    }
}

fn is_usv_string(input: &RequestInfo) -> bool {
    match input {
        &RequestInfo::USVString(_) => true,
        _ => false,
    }
}

// TODO
// https://fetch.spec.whatwg.org/#concept-body-disturbed
fn request_is_disturbed(input: &Request) -> bool {
    false
}

// TODO
// https://fetch.spec.whatwg.org/#concept-body-locked
fn request_is_locked(input: &Request) -> bool {
    false
}

// TODO
// https://fetch.spec.whatwg.org/#concept-body-disturbed
fn requestinfo_is_disturbed(input: &RequestInfo) -> bool {
    false
}

// TODO
// https://fetch.spec.whatwg.org/#concept-body-locked
fn requestinfo_is_locked(input: &RequestInfo) -> bool {
    false
}

impl RequestMethods for Request {
    // https://fetch.spec.whatwg.org/#dom-request-method
    fn Method(&self) -> ByteString {
        let r = self.request.borrow().clone();
        let m = r.method.into_inner();
        ByteString::new(Vec::from(m.as_ref().as_bytes()))
    }

    // https://fetch.spec.whatwg.org/#dom-request-url
    fn Url(&self) -> USVString {
        let r = self.request.borrow().clone();
        let url = r.url_list.into_inner()[0].clone();
        USVString(url.into_string())
    }

    // https://fetch.spec.whatwg.org/#dom-request-headers
    fn Headers(&self) -> Root<Headers> {
        self.headers_reflector.or_init(|| Headers::new(self.global().r(), None).unwrap())
    }

    // https://fetch.spec.whatwg.org/#dom-request-type
    fn Type(&self) -> RequestType {
        let r = self.request.borrow().clone();
        r.type_.into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-destination
    fn Destination(&self) -> RequestDestination {
        let r = self.request.borrow().clone();
        r.destination.into()
    }

    // https://fetch.spec.whatwg.org/#dom-request-referrer
    fn Referrer(&self) -> USVString {
        let r = self.request.borrow().clone();
        let referrer = r.referer.into_inner();
        match referrer {
            NetTraitsRequest::Referer::NoReferer => USVString(String::from("no-referrer")),
            NetTraitsRequest::Referer::Client => USVString(String::from("client")),
            NetTraitsRequest::Referer::RefererUrl(u) => USVString(u.into_string()),
        }
    }

    // https://fetch.spec.whatwg.org/#dom-request-referrerpolicy
    fn ReferrerPolicy(&self) -> ReferrerPolicy {
        let r = self.request.borrow().clone();
        let rp = r.referrer_policy.get();
        match rp {
            Some(referrer_policy) => referrer_policy.into(),
            _ => ReferrerPolicy::_empty,
        }
    }

    // https://fetch.spec.whatwg.org/#dom-request-mode
    fn Mode(&self) -> RequestMode {
        let r = self.request.borrow().clone();
        r.mode.into()
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
        let r = self.request.borrow().clone();
        let integrity = r.integrity_metadata.into_inner();
        DOMString::from_string(integrity)
    }

    // https://fetch.spec.whatwg.org/#dom-body-bodyused
    fn BodyUsed(&self) -> bool {
        self.body_used.borrow().clone()
    }

    // https://fetch.spec.whatwg.org/#dom-request-clone
    fn Clone(&self) -> Fallible<Root<Request>> {
        // Step 1
        if request_is_locked(&self) {
            return Err(Error::Type("Request is locked".to_string()));
        }
        if request_is_disturbed(&self) {
            return Err(Error::Type("Request is disturbed".to_string()));
        }

        // Step 2
        // TODO: Headers object whose guard is context object's Headers' guard. 
        let url = self.request.borrow().clone().url_list.into_inner()[0].clone();
        let origin = self.request.borrow().clone().origin.into_inner();
        let is_service_worker_global_scope = self.request.borrow().clone().is_service_worker_global_scope;
        let pipeline_id = self.request.borrow().clone().pipeline_id.get();
        let body_used = self.body_used.borrow().clone();
        Ok(Request::new(self.global().r(),
                        url,
                        Some(origin),
                        is_service_worker_global_scope,
                        pipeline_id,
                        body_used))
    }
}

impl Into<NetTraitsRequest::CacheMode> for RequestCache {
    fn into(self) -> NetTraitsRequest::CacheMode {
        match self {
            RequestCache::Default => NetTraitsRequest::CacheMode::Default,
            RequestCache::No_store => NetTraitsRequest::CacheMode::NoStore,
            RequestCache::Reload => NetTraitsRequest::CacheMode::Reload,
            RequestCache::No_cache => NetTraitsRequest::CacheMode::NoCache,
            RequestCache::Force_cache => NetTraitsRequest::CacheMode::ForceCache,
            RequestCache::Only_if_cached => NetTraitsRequest::CacheMode::OnlyIfCached,
        }
    }
}

impl Into<RequestCache> for NetTraitsRequest::CacheMode {
    fn into(self) -> RequestCache {
        match self {
            NetTraitsRequest::CacheMode::Default => RequestCache::Default,
            NetTraitsRequest::CacheMode::NoStore => RequestCache::No_store,
            NetTraitsRequest::CacheMode::Reload => RequestCache::Reload,
            NetTraitsRequest::CacheMode::NoCache => RequestCache::No_cache,
            NetTraitsRequest::CacheMode::ForceCache => RequestCache::Force_cache,
            NetTraitsRequest::CacheMode::OnlyIfCached => RequestCache::Only_if_cached,
        }
    }
}

impl Into<NetTraitsRequest::CredentialsMode> for RequestCredentials {
    fn into(self) -> NetTraitsRequest::CredentialsMode {
        match self {
            RequestCredentials::Omit => NetTraitsRequest::CredentialsMode::Omit,
            RequestCredentials::Same_origin => NetTraitsRequest::CredentialsMode::CredentialsSameOrigin,
            RequestCredentials::Include => NetTraitsRequest::CredentialsMode::Include,
        }
    }
}

impl Into<RequestCredentials> for NetTraitsRequest::CredentialsMode {
    fn into(self) -> RequestCredentials {
        match self {
            NetTraitsRequest::CredentialsMode::Omit => RequestCredentials::Omit,
            NetTraitsRequest::CredentialsMode::CredentialsSameOrigin => RequestCredentials::Same_origin,
            NetTraitsRequest::CredentialsMode::Include => RequestCredentials::Include,
        }
    }
}

impl Into<NetTraitsRequest::Destination> for RequestDestination {
    fn into(self) -> NetTraitsRequest::Destination {
        match self {
            RequestDestination::_empty => NetTraitsRequest::Destination::None,
            RequestDestination::Document => NetTraitsRequest::Destination::Document,
            RequestDestination::Embed => NetTraitsRequest::Destination::Embed,
            RequestDestination::Font => NetTraitsRequest::Destination::Font,
            RequestDestination::Image => NetTraitsRequest::Destination::Image,
            RequestDestination::Manifest => NetTraitsRequest::Destination::Manifest,
            RequestDestination::Media => NetTraitsRequest::Destination::Media,
            RequestDestination::Object => NetTraitsRequest::Destination::Object,
            RequestDestination::Report => NetTraitsRequest::Destination::Report,
            RequestDestination::Script => NetTraitsRequest::Destination::Script,
            RequestDestination::Serviceworker => NetTraitsRequest::Destination::ServiceWorker,
            RequestDestination::Sharedworker => NetTraitsRequest::Destination::SharedWorker,
            RequestDestination::Style => NetTraitsRequest::Destination::Style,
            RequestDestination::Worker => NetTraitsRequest::Destination::Worker,
            RequestDestination::Xslt => NetTraitsRequest::Destination::XSLT,
        }
    }
}

impl Into<RequestDestination> for NetTraitsRequest::Destination {
    fn into(self) -> RequestDestination {
        match self {
            NetTraitsRequest::Destination::None => RequestDestination::_empty,
            NetTraitsRequest::Destination::Document => RequestDestination::Document,
            NetTraitsRequest::Destination::Embed => RequestDestination::Embed,
            NetTraitsRequest::Destination::Font => RequestDestination::Font,
            NetTraitsRequest::Destination::Image => RequestDestination::Image,
            NetTraitsRequest::Destination::Manifest => RequestDestination::Manifest,
            NetTraitsRequest::Destination::Media => RequestDestination::Media,
            NetTraitsRequest::Destination::Object => RequestDestination::Object,
            NetTraitsRequest::Destination::Report => RequestDestination::Report,
            NetTraitsRequest::Destination::Script => RequestDestination::Script,          
            NetTraitsRequest::Destination::ServiceWorker => RequestDestination::Serviceworker,
            NetTraitsRequest::Destination::SharedWorker => RequestDestination::Sharedworker,
            NetTraitsRequest::Destination::Style => RequestDestination::Style,
            NetTraitsRequest::Destination::Worker => RequestDestination::Worker,
            NetTraitsRequest::Destination::XSLT => RequestDestination::Xslt,
        }
    }
}

impl Into<NetTraitsRequest::Type> for RequestType {
    fn into(self) -> NetTraitsRequest::Type {
        match self {
            RequestType::_empty => NetTraitsRequest::Type::None,
            RequestType::Audio => NetTraitsRequest::Type::Audio,
            RequestType::Font => NetTraitsRequest::Type::Font,
            RequestType::Image => NetTraitsRequest::Type::Image,
            RequestType::Script => NetTraitsRequest::Type::Script,
            RequestType::Style => NetTraitsRequest::Type::Style,
            RequestType::Track => NetTraitsRequest::Type::Track,
            RequestType::Video => NetTraitsRequest::Type::Video,
        }
    }
}

impl Into<RequestType> for NetTraitsRequest::Type {
    fn into(self) -> RequestType {
        match self {
            NetTraitsRequest::Type::None => RequestType::_empty,
            NetTraitsRequest::Type::Audio => RequestType::Audio,
            NetTraitsRequest::Type::Font => RequestType::Font,
            NetTraitsRequest::Type::Image => RequestType::Image,
            NetTraitsRequest::Type::Script => RequestType::Script,
            NetTraitsRequest::Type::Style => RequestType::Style,
            NetTraitsRequest::Type::Track => RequestType::Track,
            NetTraitsRequest::Type::Video => RequestType::Video,
        }
    }
}

impl Into<NetTraitsRequest::RequestMode> for RequestMode {
    fn into(self) -> NetTraitsRequest::RequestMode {
        match self {
            RequestMode::Navigate => NetTraitsRequest::RequestMode::Navigate,
            RequestMode::Same_origin => NetTraitsRequest::RequestMode::SameOrigin,
            RequestMode::No_cors => NetTraitsRequest::RequestMode::NoCORS,
            RequestMode::Cors => NetTraitsRequest::RequestMode::CORSMode,
        }
    }
}

impl Into<RequestMode> for NetTraitsRequest::RequestMode {
    fn into(self) -> RequestMode {
        match self {
            NetTraitsRequest::RequestMode::Navigate => RequestMode::Navigate,
            NetTraitsRequest::RequestMode::SameOrigin => RequestMode::Same_origin,
            NetTraitsRequest::RequestMode::NoCORS => RequestMode::No_cors,
            NetTraitsRequest::RequestMode::CORSMode => RequestMode::Cors,
        }
    }
}

// TODO
// RequestBinding::ReferrerPolicy does not match msg::constellation_msg::ReferrerPolicy
// RequestBinding::ReferrerPolicy has _empty
//   that is not in msg::constellation_msg::ReferrerPolicy
// msg::constellation_msg::ReferrerPolicy has SameOrigin
//   that is not in RequestBinding::ReferrerPolicy
impl Into<msg::constellation_msg::ReferrerPolicy> for ReferrerPolicy {
    fn into(self) -> msg::constellation_msg::ReferrerPolicy {
        match self {
            ReferrerPolicy::_empty => msg::constellation_msg::ReferrerPolicy::NoReferrer,
            ReferrerPolicy::No_referrer => msg::constellation_msg::ReferrerPolicy::NoReferrer,
            ReferrerPolicy::No_referrer_when_downgrade => msg::constellation_msg::ReferrerPolicy::NoReferrerWhenDowngrade,
            ReferrerPolicy::Origin => msg::constellation_msg::ReferrerPolicy::Origin,
            ReferrerPolicy::Origin_when_cross_origin => msg::constellation_msg::ReferrerPolicy::OriginWhenCrossOrigin,
            ReferrerPolicy::Unsafe_url => msg::constellation_msg::ReferrerPolicy::UnsafeUrl,
        }
    }
}

impl Into<ReferrerPolicy> for msg::constellation_msg::ReferrerPolicy {
    fn into(self) -> ReferrerPolicy {
        match self {
            msg::constellation_msg::ReferrerPolicy::NoReferrer => ReferrerPolicy::No_referrer,
            msg::constellation_msg::ReferrerPolicy::NoReferrerWhenDowngrade => ReferrerPolicy::No_referrer_when_downgrade,
            msg::constellation_msg::ReferrerPolicy::Origin => ReferrerPolicy::Origin,
            msg::constellation_msg::ReferrerPolicy::SameOrigin => ReferrerPolicy::Origin,
            msg::constellation_msg::ReferrerPolicy::OriginWhenCrossOrigin => ReferrerPolicy::Origin_when_cross_origin,
            msg::constellation_msg::ReferrerPolicy::UnsafeUrl => ReferrerPolicy::Unsafe_url,
        }
    }
}

impl Into<NetTraitsRequest::RedirectMode> for RequestRedirect {
    fn into(self) -> NetTraitsRequest::RedirectMode {
        match self {
            RequestRedirect::Follow => NetTraitsRequest::RedirectMode::Follow,
            RequestRedirect::Error => NetTraitsRequest::RedirectMode::Error,
            RequestRedirect::Manual => NetTraitsRequest::RedirectMode::Manual,
        }
    }
}

impl Into<RequestRedirect> for NetTraitsRequest::RedirectMode {
    fn into(self) -> RequestRedirect {
        match self {
            NetTraitsRequest::RedirectMode::Follow => RequestRedirect::Follow,
            NetTraitsRequest::RedirectMode::Error => RequestRedirect::Error,
            NetTraitsRequest::RedirectMode::Manual => RequestRedirect::Manual,
        }
    }
}
