/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::RequestBinding;
use dom::bindings::codegen::Bindings::RequestBinding::RequestMethods;
use dom::bindings::codegen::Bindings::RequestBinding::RequestType;
use dom::bindings::codegen::Bindings::RequestBinding::RequestDestination;
// use dom::bindings::codegen::Bindings::RequestBinding::ReferrerPolicy;
// use msg::constellation_msg::ReferrerPolicy;
use dom::bindings::codegen::Bindings::RequestBinding::RequestMode;
use dom::bindings::codegen::Bindings::RequestBinding::RequestCredentials;
use dom::bindings::codegen::Bindings::RequestBinding::RequestCache;
use dom::bindings::codegen::Bindings::RequestBinding::RequestRedirect;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root; 
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, USVString, DOMString};

#[dom_struct]
pub struct Request {
    reflector_: Reflector,
    method: ByteString,
    url: USVString,
    // header: Headers,
    request_type: RequestType,
    destination: RequestDestination,
    referrer: USVString,
    // referrer_policy: ReferrerPolicy,
    mode: RequestMode,
    credentials: RequestCredentials,
    cache: RequestCache,
    redirect: RequestRedirect,
    integrity: DOMString,
}

impl Request {
    pub fn new_inherited(method: ByteString,
                         url: USVString,
                         // header: Headers,
                         request_type: RequestType,
                         destination: RequestDestination,
                         referrer: USVString,
                         // referrer_policy: ReferrerPolicy,
                         mode: RequestMode,
                         credentials: RequestCredentials,
                         cache: RequestCache,
                         redirect: RequestRedirect,
                         integrity: DOMString
    ) -> Request {
        Request {
            reflector_: Reflector::new(),
            method: method,
            url: url,
            // header: header,
            request_type: request_type,
            destination: destination,
            referrer: referrer,
            // referrer_policy: referrer_policy,
            mode: mode,
            credentials: credentials,
            cache: cache,
            redirect: redirect,
            integrity: integrity,
        }
    }

    pub fn new(global: GlobalRef,
               method: ByteString,
               url: USVString,
               // header: Headers,
               request_type: RequestType,
               destination: RequestDestination,
               referrer: USVString,
               // referrer_policy: ReferrerPolicy,
               mode: RequestMode,
               credentials: RequestCredentials,
               cache: RequestCache,
               redirect: RequestRedirect,
               integrity: DOMString
    ) -> Root<Request> {
        reflect_dom_object(box Request::new_inherited(method,
                                                      url,
                                                      // header,
                                                      request_type,
                                                      destination,
                                                      referrer,
                                                      // referrer_policy,
                                                      mode,
                                                      credentials,
                                                      cache,
                                                      redirect,
                                                      integrity),
                           global, RequestBinding::Wrap)
    }
}

impl RequestMethods for Request {
    fn Method(&self) -> ByteString {
        self.method.to_owned()
    }
    
    fn Url(&self) -> USVString {
        self.url.clone()
    }

    fn Type(&self) -> RequestType {
        self.request_type
    }

    fn Destination(&self) -> RequestDestination {
        self.destination
    }

    fn Referrer(&self) -> USVString {
        self.referrer.clone()
    }

    // throws webidl build error
    // fn Referrer_Policy(&self) -> ReferrerPolicy {
    //     self.referrer_policy
    //}

    fn Mode(&self) -> RequestMode {
         self.mode
    }

    fn Credentials(&self) -> RequestCredentials {
        self.credentials
    }

    fn Cache(&self) -> RequestCache {
        self.cache
    }

    fn Redirect(&self) -> RequestRedirect {
        self.redirect
    }

    fn Integrity(&self) -> DOMString {
        let integrity_str = format!("{}", self.integrity);
        DOMString::from_string(integrity_str)
    }
    
}
