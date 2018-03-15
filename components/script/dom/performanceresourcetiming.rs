/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::PerformanceResourceTimingMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom_struct::dom_struct;
use servo_url::ServoUrl;


#[dom_struct]
pub struct PerformanceResourceTiming {
    reflector_: Reflector,
    // returns the resolved URL of the requested resource
    // does not change even if redirected to a different URl
    name: ServoUrl,
    initiator_type: DOMString,
    next_hop: DOMString,
    worker_start: f64,
    redirect_start: f64,
    fetch_start: f64,
    domain_lookup_start: f64,
    domain_lookup_end: f64,
    connect_start: f64,
    connect_end: f64,
    secure_connection_start: f64,
    request_start: f64,
    response_start: f64,
    redirect_start: f64,
    transfer_size: f64,	//size in octets
    encoded_body_size: f64, //size in octets
    decoded_body_size: f64, //size in octets
}

impl PerformanceResourceTiming {
    // TODO
}

// https://w3c.github.io/resource-timing/
impl PerformanceResourceTimingMethods for PerformanceResourceTiming {
    // https://w3c.github.io/resource-timing/
    fn InitiatorType(&self) -> DOMString {
        // TODO
        Default::default()
    }

    // https://w3c.github.io/resource-timing/
    fn NextHopProtocol(&self) -> DOMString {
        // TODO
        Default::default()
    }

    // https://w3c.github.io/resource-timing/
    fn WorkerStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn RedirectStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn RedirectEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn FetchStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn DomainLookupStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn DomainLookupEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ConnectStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ConnectEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn SecureConnectionStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn RequestStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ResponseStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/resource-timing/
    fn ResponseEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

}

impl PerformanceEntryMethods for PerformanceEntry {
    // https://w3c.github.io/resource-timing/#sec-performanceresourcetiming
    // This attribute MUST return the resolved URL of the requested resource. This attribute MUST NOT change even if the fetch redirected to a different URL
    fn Name(&self) -> DOMString {
        DOMString::from(self.name().url.as_url())
    }

    // https://w3c.github.io/resource-timing/#sec-performanceresourcetiming
    fn EntryType(&self) -> DOMString {
        DOMString::from("resource")
    }

    // https://w3c.github.io/resource-timing/#sec-performanceresourcetiming
    fn StartTime(&self) -> Finite<f64> {
    	// TODO time immediately before UA queues resource for fetching
        Finite::wrap(self.startTime)
    }

    // https://w3c.github.io/resource-timing/#sec-performanceresourcetiming
    fn Duration(&self) -> Finite<f64> {
    	// TODO responseEnd-startTime
        self.performance_entry.Duration()
    }
}
