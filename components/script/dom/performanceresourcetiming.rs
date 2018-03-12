/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unused)]

use dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::PerformanceResourceTimingMethods;
use dom::bindings::num::Finite;
use dom::bindings::str::DOMString;
use dom::performanceentry::PerformanceEntry;
use dom_struct::dom_struct;
use servo_url::ServoUrl;

// TODO UA may choose to limit how many resources are included as PerformanceResourceTiming objects
// recommended minimum is 150, can be changed by setResourceTimingBufferSize in performance
// https://w3c.github.io/resource-timing/#sec-extensions-performance-interface

// TODO Cross origin resources MUST BE INCLUDED as PerformanceResourceTiming objects
// https://w3c.github.io/resource-timing/#sec-cross-origin-resources


#[dom_struct]
pub struct PerformanceResourceTiming {
    entry: PerformanceEntry,
    initiator_type: DOMString,
    next_hop: Option<DOMString>,
    worker_start: f64,
    redirect_start: f64,
    redirect_end: f64,
    fetch_start: f64,
    domain_lookup_start: f64,
    domain_lookup_end: f64,
    connect_start: f64,
    connect_end: f64,
    secure_connection_start: f64,
    request_start: f64,
    response_start: f64,
    response_end: f64,
    // transfer_size: f64, //size in octets
    // encoded_body_size: f64, //size in octets
    // decoded_body_size: f64, //size in octets
}

#[allow(unsafe_code)]
unsafe impl Send for PerformanceResourceTiming {}

impl PerformanceResourceTiming {
    pub fn new_inherited(url: ServoUrl,
                     initiator_type: DOMString,
                     next_hop: Option<DOMString>,
                     fetch_start: f64)
                         -> PerformanceResourceTiming {
        // TODO Do i know the end time yet?
        PerformanceResourceTiming {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(url.into_string()),
                DOMString::from("resource"),
                fetch_start,
                0.),
            initiator_type: initiator_type,
            next_hop: next_hop,
            worker_start: 0.,
            redirect_start: 0.,
            redirect_end: 0.,
            fetch_start: fetch_start,
            domain_lookup_start: 0.,
            domain_lookup_end: 0.,
            connect_start: 0.,
            connect_end: 0.,
            secure_connection_start: 0.,
            request_start: 0.,
            response_start: 0.,
            response_end: 0.,
        }
    }

    // TODO prevent setting start if it's already been set?
    pub fn set_worker_start(&mut self, start_time: f64) {
        self.worker_start = start_time;
    }

    pub fn set_redirect_start(&mut self, start_time: f64) {
        self.redirect_start = start_time;
    }

    pub fn set_fetch_start(&mut self, start_time: f64) {
        self.fetch_start = start_time;
    }

    pub fn set_domain_lookup_start(&mut self, start_time: f64) {
        self.domain_lookup_start = start_time;
    }

    pub fn set_connect_start(&mut self, start_time: f64) {
        self.connect_start = start_time;
    }

    pub fn set_secure_connection_start(&mut self, start_time: f64) {
        self.secure_connection_start = start_time;
    }

    pub fn set_request_start(&mut self, start_time: f64) {
        self.request_start = start_time;
    }

    pub fn set_response_start(&mut self, start_time: f64) {
        self.response_start = start_time;
    }

    pub fn set_response_end(&mut self, end_time: f64) {
        self.response_end = end_time;
    }

    pub fn set_redirect_end(&mut self, end_time: f64) {
        self.redirect_end = end_time;
    }

    pub fn set_domain_lookup_end(&mut self, end_time: f64) {
        self.domain_lookup_end = end_time;
    }

    pub fn set_connect_end(&mut self, end_time: f64) {
        self.connect_end = end_time;
    }
}

// https://w3c.github.io/resource-timing/
impl PerformanceResourceTimingMethods for PerformanceResourceTiming {
    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-initiatortype
    fn InitiatorType(&self) -> DOMString {
        DOMString::from(self.initiator_type.clone())
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-nexthopprotocol
    // returns the ALPN protocol ID of the network protocol used to fetch the resource
    // when a proxy is configured TODO
    fn NextHopProtocol(&self) -> DOMString {
        // TODO
        match self.next_hop {
            Some(ref protocol) => DOMString::from(protocol.clone()),
            None => DOMString::from(""),
        }
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-workerstart
    fn WorkerStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.worker_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectstart
    fn RedirectStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.redirect_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectend
    fn RedirectEnd(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.redirect_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-fetchstart
    fn FetchStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.fetch_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupstart
    fn DomainLookupStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.domain_lookup_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupend
    fn DomainLookupEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.domain_lookup_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectstart
    fn ConnectStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.connect_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectend
    fn ConnectEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.connect_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-secureconnectstart
    fn SecureConnectionStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.secure_connection_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-requeststart
    fn RequestStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.request_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responsestart
    fn ResponseStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.response_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responseend
    fn ResponseEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(self.response_end)
    }

}
