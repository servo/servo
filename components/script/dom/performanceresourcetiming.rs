/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::{
    self, PerformanceResourceTimingMethods,
};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;
use dom_struct::dom_struct;
use net_traits::ResourceFetchTiming;
use servo_url::ServoUrl;

// TODO UA may choose to limit how many resources are included as PerformanceResourceTiming objects
// recommended minimum is 150, can be changed by setResourceTimingBufferSize in performance
// https://w3c.github.io/resource-timing/#sec-extensions-performance-interface

// TODO Cross origin resources MUST BE INCLUDED as PerformanceResourceTiming objects
// https://w3c.github.io/resource-timing/#sec-cross-origin-resources

// TODO CSS, Beacon
#[derive(Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub enum InitiatorType {
    LocalName(String),
    Navigation,
    XMLHttpRequest,
    Fetch,
    Other,
}

#[dom_struct]
pub struct PerformanceResourceTiming {
    entry: PerformanceEntry,
    initiator_type: InitiatorType,
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

// TODO(#21254): startTime
// TODO(#21255): duration
// TODO(#21269): next_hop
// TODO(#21264): worker_start
// TODO(#21256): redirect_start
// TODO(#21257): redirect_end
// TODO(#21258): fetch_start
// TODO(#21259): domain_lookup_start
// TODO(#21260): domain_lookup_end
// TODO(#21261): connect_start
// TODO(#21262): connect_end
impl PerformanceResourceTiming {
    pub fn new_inherited(
        url: ServoUrl,
        initiator_type: InitiatorType,
        next_hop: Option<DOMString>,
        fetch_start: f64,
    ) -> PerformanceResourceTiming {
        PerformanceResourceTiming {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(url.into_string()),
                DOMString::from("resource"),
                0.,
                0.,
            ),
            initiator_type: initiator_type,
            next_hop: next_hop,
            worker_start: 0.,
            redirect_start: 0.,
            redirect_end: 0.,
            fetch_start: fetch_start,
            domain_lookup_end: 0.,
            domain_lookup_start: 0.,
            connect_start: 0.,
            connect_end: 0.,
            secure_connection_start: 0.,
            request_start: 0.,
            response_start: 0.,
            response_end: 0.,
        }
    }

    //TODO fetch start should be in RFT
    #[allow(unrooted_must_root)]
    fn from_resource_timing(
        url: ServoUrl,
        initiator_type: InitiatorType,
        next_hop: Option<DOMString>,
        resource_timing: &ResourceFetchTiming,
    ) -> PerformanceResourceTiming {
        PerformanceResourceTiming {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(url.into_string()),
                DOMString::from("resource"),
                0.,
                0.,
            ),
            initiator_type: initiator_type,
            next_hop: next_hop,
            worker_start: 0.,
            redirect_start: resource_timing.redirect_start as f64,
            redirect_end: 0.,
            fetch_start: resource_timing.fetch_start as f64,
            domain_lookup_start: 0.,
            domain_lookup_end: 0.,
            connect_start: 0.,
            connect_end: resource_timing.connect_end as f64,
            secure_connection_start: 0.,
            request_start: resource_timing.request_start as f64,
            response_start: resource_timing.response_start as f64,
            response_end: resource_timing.response_end as f64,
        }
    }

    pub fn new(
        global: &GlobalScope,
        url: ServoUrl,
        initiator_type: InitiatorType,
        next_hop: Option<DOMString>,
        resource_timing: &ResourceFetchTiming,
    ) -> DomRoot<PerformanceResourceTiming> {
        reflect_dom_object(
            Box::new(PerformanceResourceTiming::from_resource_timing(
                url,
                initiator_type,
                next_hop,
                resource_timing,
            )),
            global,
            PerformanceResourceTimingBinding::Wrap,
        )
    }
}

// https://w3c.github.io/resource-timing/
impl PerformanceResourceTimingMethods for PerformanceResourceTiming {
    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-initiatortype
    fn InitiatorType(&self) -> DOMString {
        match self.initiator_type {
            InitiatorType::LocalName(ref n) => DOMString::from(n.clone()),
            InitiatorType::Navigation => DOMString::from("navigation"),
            InitiatorType::XMLHttpRequest => DOMString::from("xmlhttprequest"),
            InitiatorType::Fetch => DOMString::from("fetch"),
            InitiatorType::Other => DOMString::from("other"),
        }
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-nexthopprotocol
    // returns the ALPN protocol ID of the network protocol used to fetch the resource
    // when a proxy is configured
    fn NextHopProtocol(&self) -> DOMString {
        match self.next_hop {
            Some(ref protocol) => DOMString::from(protocol.clone()),
            None => DOMString::from(""),
        }
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupstart
    fn DomainLookupStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.domain_lookup_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-requeststart
    fn RequestStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.request_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectstart
    fn RedirectStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.redirect_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responsestart
    fn ResponseStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.response_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-fetchstart
    fn FetchStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.fetch_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectend
    fn ConnectEnd(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.connect_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responseend
    fn ResponseEnd(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.response_end)
    }
}
