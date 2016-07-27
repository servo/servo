/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::headers::Headers;
use dom::bindings::reflector::Reflectable;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::UnionTypes::RequestOrUSVString;
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
use msg;
use net_traits;
use std::cell::{Cell, Ref, RefCell};
use url;

#[dom_struct]
pub struct Request {
    reflector_: Reflector,
    #[ignore_heap_size_of = "net_traits is missing HeapSizeOf implementation"]
    request: DOMRefCell<net_traits::request::Request>,
    body_used: DOMRefCell<bool>,
    headers_reflector: MutNullableHeap<JS<Headers>>,
}

impl Request {
    // https://fetch.spec.whatwg.org/#concept-request-url
    fn get_associated_url(req: &net_traits::request::Request) -> Option<Ref<url::Url>> {
        let url_list = req.url_list.borrow();
        if url_list.len() > 0 {
            Some(Ref::map(url_list, |urls| urls.first().unwrap()))
        } else {
            None
        }
    }

    // https://fetch.spec.whatwg.org/#concept-request-current-url
    fn get_current_url(req: &net_traits::request::Request) -> Option<Ref<url::Url>> {
        let url_list = req.url_list.borrow();
        if url_list.len() > 0 {
            Some(Ref::map(url_list, |urls| urls.last().unwrap()))
        } else {
            None
        }
    }
    
    pub fn new_inherited(url: url::Url,
                         origin: Option<net_traits::request::Origin>,
                         is_service_worker_global_scope: bool,
                         body_used: bool) -> Request {
        Request {
            reflector_: Reflector::new(),
            request: DOMRefCell::new(
                net_traits::request::Request::new(url,
                                                  origin,
                                                  is_service_worker_global_scope)),
            body_used: DOMRefCell::new(body_used),
            headers_reflector: Default::default(),
        }
    }

    pub fn new(global: GlobalRef,
               url: url::Url,
               origin: Option<net_traits::request::Origin>,
               is_service_worker_global_scope: bool,
               body_used: bool) -> Root<Request> {
        reflect_dom_object(box Request::new_inherited(url,
                                                      origin,
                                                      is_service_worker_global_scope,
                                                      body_used),
                           global, RequestBinding::Wrap)
    }

    // https://fetch.spec.whatwg.org/#dom-request
    pub fn Constructor(global: GlobalRef,
                       input: RequestInfo,
                       init: &RequestInit)
                      -> Fallible<Root<Request>> {

        let mut new_request = Request::new(global,
                                           url::Url::parse("").unwrap(),
                                           None,
                                           false,
                                           false);

        // Step 1
        if is_request(&input) &&
            (requestinfo_is_disturbed(&input) || requestinfo_is_locked(&input)) {
                return Err(Error::Type("Input is disturbed or locked".to_string()))
            }

        // Step 2
        let mut new_request_request =
            net_traits::request::Request::new(url::Url::parse("").unwrap(),
                                              None,
                                              false);
        if let &RequestOrUSVString::Request(ref req) = &input {
            new_request_request = req.request.borrow().clone();
        }

        // TODO
        // Step 3
        // if you need an origin at some point, you can try using `global.url().origin()`
        // let origin = "entry settings object origin";

        // Step 4
        let mut window = Cell::new(net_traits::request::Window::Client);

        // TODO
        // Step 5
        // if new_request_request.window == "environment settings object"
        //     && new_request_request.origin == origin {
        //         window = new_request.request.borrow_mut().window;
        // }

        // Step 6
        if !init.window.is_undefined() && !init.window.is_null() {
            return Err(Error::Type("Window is present and is not null".to_string()))
        }

        // Step 7
        if !init.window.is_undefined() {
            window.set(net_traits::request::Window::NoWindow);
        }

        // Step 8
        if let Some(url) = Request::get_current_url(&new_request_request) {
            new_request.request.borrow_mut().url_list = RefCell::new(vec![url.clone()]);
        }
        new_request.request.borrow_mut().method =
            new_request_request.method;
        new_request.request.borrow_mut().headers =
            new_request_request.headers.clone();
        new_request.request.borrow_mut().unsafe_request = true;
        new_request.request.borrow_mut().window =
            window;
        // TODO: client
        // client is entry settings object
        new_request.request.borrow_mut().origin =
            RefCell::new(net_traits::request::Origin::Client);
        new_request.request.borrow_mut().omit_origin_header =
            new_request_request.omit_origin_header;
        new_request.request.borrow_mut().same_origin_data =
            Cell::new(true);
        new_request.request.borrow_mut().referer =
            new_request_request.referer;
        new_request.request.borrow_mut().referrer_policy =
            new_request_request.referrer_policy;
        new_request.request.borrow_mut().mode =
            new_request_request.mode;
        new_request.request.borrow_mut().credentials_mode =
            new_request_request.credentials_mode;
        new_request.request.borrow_mut().cache_mode =
            new_request_request.cache_mode;
        new_request.request.borrow_mut().redirect_mode =
            new_request_request.redirect_mode;
        new_request.request.borrow_mut().integrity_metadata =
            new_request_request.integrity_metadata;

        // Step 9
        let mut fallback_mode: net_traits::request::RequestMode;

        // Step 10
        let mut fallback_credentials: net_traits::request::CredentialsMode;

        // TODO
        // Step 11
        // let base_url = "somebaseurl";

        // Step 12
        if let &RequestOrUSVString::USVString(USVString(ref usv_string)) = &input {
            // Step 12.1
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
                new_request.request.borrow_mut().url_list = RefCell::new(vec![url]);
            }
            // Step 12.5
            fallback_mode = net_traits::request::RequestMode::CORSMode;
            // Step 12.6
            fallback_credentials = net_traits::request::CredentialsMode::Omit;
                    //let mut new_request_request = req.request.borrow().clone();
        //*new_request.request.borrow_mut() = req.request.borrow().clone();
        }
        unimplemented!();
    }

    
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
        self.headers_reflector.or_init(|| Headers::new(self.global().r()))
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
            net_traits::request::Referer::NoReferer => USVString(String::from("no-referrer")),
            net_traits::request::Referer::Client => USVString(String::from("client")),
            net_traits::request::Referer::RefererUrl(u) => USVString(u.into_string()),
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
        let url = self.request.borrow().clone().url_list.into_inner()[0].clone();
        let origin = self.request.borrow().clone().origin.into_inner();
        let is_service_worker_global_scope = self.request.borrow().clone().is_service_worker_global_scope;
        Ok(Request::new(self.global().r(),
                        url,
                        Some(origin),
                        is_service_worker_global_scope,
                        false))
    }
}

impl Into<net_traits::request::CacheMode> for RequestCache {
    fn into(self) -> net_traits::request::CacheMode {
        match self {
            RequestCache::Default => net_traits::request::CacheMode::Default,
            RequestCache::No_store => net_traits::request::CacheMode::NoStore,
            RequestCache::Reload => net_traits::request::CacheMode::Reload,
            RequestCache::No_cache => net_traits::request::CacheMode::NoCache,
            RequestCache::Force_cache => net_traits::request::CacheMode::ForceCache,
            RequestCache::Only_if_cached => net_traits::request::CacheMode::OnlyIfCached,
        }
    }
}

impl Into<RequestCache> for net_traits::request::CacheMode {
    fn into(self) -> RequestCache {
        match self {
            net_traits::request::CacheMode::Default => RequestCache::Default,
            net_traits::request::CacheMode::NoStore => RequestCache::No_store,
            net_traits::request::CacheMode::Reload => RequestCache::Reload,
            net_traits::request::CacheMode::NoCache => RequestCache::No_cache,
            net_traits::request::CacheMode::ForceCache => RequestCache::Force_cache,
            net_traits::request::CacheMode::OnlyIfCached => RequestCache::Only_if_cached,
        }
    }
}

impl Into<net_traits::request::CredentialsMode> for RequestCredentials {
    fn into(self) -> net_traits::request::CredentialsMode {
        match self {
            RequestCredentials::Omit => net_traits::request::CredentialsMode::Omit,
            RequestCredentials::Same_origin => net_traits::request::CredentialsMode::CredentialsSameOrigin,
            RequestCredentials::Include => net_traits::request::CredentialsMode::Include,
        }
    }
}

impl Into<RequestCredentials> for net_traits::request::CredentialsMode {
    fn into(self) -> RequestCredentials {
        match self {
            net_traits::request::CredentialsMode::Omit => RequestCredentials::Omit,
            net_traits::request::CredentialsMode::CredentialsSameOrigin => RequestCredentials::Same_origin,
            net_traits::request::CredentialsMode::Include => RequestCredentials::Include,
        }
    }
}

impl Into<net_traits::request::Destination> for RequestDestination {
    fn into(self) -> net_traits::request::Destination {
        match self {
            RequestDestination::_empty => net_traits::request::Destination::None,
            RequestDestination::Document => net_traits::request::Destination::Document,
            RequestDestination::Embed => net_traits::request::Destination::Embed,
            RequestDestination::Font => net_traits::request::Destination::Font,
            RequestDestination::Image => net_traits::request::Destination::Image,
            RequestDestination::Manifest => net_traits::request::Destination::Manifest,
            RequestDestination::Media => net_traits::request::Destination::Media,
            RequestDestination::Object => net_traits::request::Destination::Object,
            RequestDestination::Report => net_traits::request::Destination::Report,
            RequestDestination::Script => net_traits::request::Destination::Script,
            RequestDestination::Serviceworker => net_traits::request::Destination::ServiceWorker,
            RequestDestination::Sharedworker => net_traits::request::Destination::SharedWorker,
            RequestDestination::Style => net_traits::request::Destination::Style,
            RequestDestination::Worker => net_traits::request::Destination::Worker,
            RequestDestination::Xslt => net_traits::request::Destination::XSLT,
        }
    }
}

impl Into<RequestDestination> for net_traits::request::Destination {
    fn into(self) -> RequestDestination {
        match self {
            net_traits::request::Destination::None => RequestDestination::_empty,
            net_traits::request::Destination::Document => RequestDestination::Document,
            net_traits::request::Destination::Embed => RequestDestination::Embed,
            net_traits::request::Destination::Font => RequestDestination::Font,
            net_traits::request::Destination::Image => RequestDestination::Image,
            net_traits::request::Destination::Manifest => RequestDestination::Manifest,
            net_traits::request::Destination::Media => RequestDestination::Media,
            net_traits::request::Destination::Object => RequestDestination::Object,
            net_traits::request::Destination::Report => RequestDestination::Report,
            net_traits::request::Destination::Script => RequestDestination::Script,          
            net_traits::request::Destination::ServiceWorker => RequestDestination::Serviceworker,
            net_traits::request::Destination::SharedWorker => RequestDestination::Sharedworker,
            net_traits::request::Destination::Style => RequestDestination::Style,
            net_traits::request::Destination::Worker => RequestDestination::Worker,
            net_traits::request::Destination::XSLT => RequestDestination::Xslt,
        }
    }
}

impl Into<net_traits::request::Type> for RequestType {
    fn into(self) -> net_traits::request::Type {
        match self {
            RequestType::_empty => net_traits::request::Type::None,
            RequestType::Audio => net_traits::request::Type::Audio,
            RequestType::Font => net_traits::request::Type::Font,
            RequestType::Image => net_traits::request::Type::Image,
            RequestType::Script => net_traits::request::Type::Script,
            RequestType::Style => net_traits::request::Type::Style,
            RequestType::Track => net_traits::request::Type::Track,
            RequestType::Video => net_traits::request::Type::Video,
        }
    }
}

impl Into<RequestType> for net_traits::request::Type {
    fn into(self) -> RequestType {
        match self {
            net_traits::request::Type::None => RequestType::_empty,
            net_traits::request::Type::Audio => RequestType::Audio,
            net_traits::request::Type::Font => RequestType::Font,
            net_traits::request::Type::Image => RequestType::Image,
            net_traits::request::Type::Script => RequestType::Script,
            net_traits::request::Type::Style => RequestType::Style,
            net_traits::request::Type::Track => RequestType::Track,
            net_traits::request::Type::Video => RequestType::Video,
        }
    }
}

impl Into<net_traits::request::RequestMode> for RequestMode {
    fn into(self) -> net_traits::request::RequestMode {
        match self {
            RequestMode::Navigate => net_traits::request::RequestMode::Navigate,
            RequestMode::Same_origin => net_traits::request::RequestMode::SameOrigin,
            RequestMode::No_cors => net_traits::request::RequestMode::NoCORS,
            RequestMode::Cors => net_traits::request::RequestMode::CORSMode,
        }
    }
}

impl Into<RequestMode> for net_traits::request::RequestMode {
    fn into(self) -> RequestMode {
        match self {
            net_traits::request::RequestMode::Navigate => RequestMode::Navigate,
            net_traits::request::RequestMode::SameOrigin => RequestMode::Same_origin,
            net_traits::request::RequestMode::NoCORS => RequestMode::No_cors,
            net_traits::request::RequestMode::CORSMode => RequestMode::Cors,
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

impl Into<net_traits::request::RedirectMode> for RequestRedirect {
    fn into(self) -> net_traits::request::RedirectMode {
        match self {
            RequestRedirect::Follow => net_traits::request::RedirectMode::Follow,
            RequestRedirect::Error => net_traits::request::RedirectMode::Error,
            RequestRedirect::Manual => net_traits::request::RedirectMode::Manual,
        }
    }
}

impl Into<RequestRedirect> for net_traits::request::RedirectMode {
    fn into(self) -> RequestRedirect {
        match self {
            net_traits::request::RedirectMode::Follow => RequestRedirect::Follow,
            net_traits::request::RedirectMode::Error => RequestRedirect::Error,
            net_traits::request::RedirectMode::Manual => RequestRedirect::Manual,
        }
    }
}
