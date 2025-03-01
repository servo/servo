/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use net_traits::ResourceFetchTiming;
use servo_url::ServoUrl;
use time::Duration;

use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::PerformanceResourceTimingMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;
use crate::script_runtime::CanGc;
// TODO UA may choose to limit how many resources are included as PerformanceResourceTiming objects
// recommended minimum is 150, can be changed by setResourceTimingBufferSize in performance
// https://w3c.github.io/resource-timing/#sec-extensions-performance-interface

// TODO Cross origin resources MUST BE INCLUDED as PerformanceResourceTiming objects
// https://w3c.github.io/resource-timing/#sec-cross-origin-resources

// TODO CSS, Beacon
#[derive(Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum InitiatorType {
    LocalName(String),
    Navigation,
    XMLHttpRequest,
    Fetch,
    Other,
}

#[dom_struct]
pub(crate) struct PerformanceResourceTiming {
    entry: PerformanceEntry,
    initiator_type: InitiatorType,
    next_hop: Option<DOMString>,
    #[no_trace]
    worker_start: Option<CrossProcessInstant>,
    #[no_trace]
    redirect_start: Option<CrossProcessInstant>,
    #[no_trace]
    redirect_end: Option<CrossProcessInstant>,
    #[no_trace]
    fetch_start: Option<CrossProcessInstant>,
    #[no_trace]
    domain_lookup_start: Option<CrossProcessInstant>,
    #[no_trace]
    domain_lookup_end: Option<CrossProcessInstant>,
    #[no_trace]
    connect_start: Option<CrossProcessInstant>,
    #[no_trace]
    connect_end: Option<CrossProcessInstant>,
    #[no_trace]
    secure_connection_start: Option<CrossProcessInstant>,
    #[no_trace]
    request_start: Option<CrossProcessInstant>,
    #[no_trace]
    response_start: Option<CrossProcessInstant>,
    #[no_trace]
    response_end: Option<CrossProcessInstant>,
    transfer_size: u64,     //size in octets
    encoded_body_size: u64, //size in octets
    decoded_body_size: u64, //size in octets
}

// TODO(#21269): next_hop
// TODO(#21264): worker_start
// TODO(#21258): fetch_start
// TODO(#21259): domain_lookup_start
// TODO(#21260): domain_lookup_end
// TODO(#21261): connect_start
// TODO(#21262): connect_end
impl PerformanceResourceTiming {
    pub(crate) fn new_inherited(
        url: ServoUrl,
        initiator_type: InitiatorType,
        next_hop: Option<DOMString>,
        fetch_start: Option<CrossProcessInstant>,
    ) -> PerformanceResourceTiming {
        let entry_type = if initiator_type == InitiatorType::Navigation {
            DOMString::from("navigation")
        } else {
            DOMString::from("resource")
        };
        PerformanceResourceTiming {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(url.into_string()),
                entry_type,
                None,
                Duration::ZERO,
            ),
            initiator_type,
            next_hop,
            worker_start: None,
            redirect_start: None,
            redirect_end: None,
            fetch_start,
            domain_lookup_end: None,
            domain_lookup_start: None,
            connect_start: None,
            connect_end: None,
            secure_connection_start: None,
            request_start: None,
            response_start: None,
            response_end: None,
            transfer_size: 0,
            encoded_body_size: 0,
            decoded_body_size: 0,
        }
    }

    //TODO fetch start should be in RFT
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn from_resource_timing(
        url: ServoUrl,
        initiator_type: InitiatorType,
        next_hop: Option<DOMString>,
        resource_timing: &ResourceFetchTiming,
    ) -> PerformanceResourceTiming {
        let duration = match (resource_timing.start_time, resource_timing.response_end) {
            (Some(start_time), Some(end_time)) => end_time - start_time,
            _ => Duration::ZERO,
        };
        PerformanceResourceTiming {
            entry: PerformanceEntry::new_inherited(
                DOMString::from(url.into_string()),
                DOMString::from("resource"),
                resource_timing.start_time,
                duration,
            ),
            initiator_type,
            next_hop,
            worker_start: None,
            redirect_start: resource_timing.redirect_start,
            redirect_end: resource_timing.redirect_end,
            fetch_start: resource_timing.fetch_start,
            domain_lookup_start: resource_timing.domain_lookup_start,
            //TODO (#21260)
            domain_lookup_end: None,
            connect_start: resource_timing.connect_start,
            connect_end: resource_timing.connect_end,
            secure_connection_start: resource_timing.secure_connection_start,
            request_start: resource_timing.request_start,
            response_start: resource_timing.response_start,
            response_end: resource_timing.response_end,
            transfer_size: 0,
            encoded_body_size: 0,
            decoded_body_size: 0,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        url: ServoUrl,
        initiator_type: InitiatorType,
        next_hop: Option<DOMString>,
        resource_timing: &ResourceFetchTiming,
        can_gc: CanGc,
    ) -> DomRoot<PerformanceResourceTiming> {
        reflect_dom_object(
            Box::new(PerformanceResourceTiming::from_resource_timing(
                url,
                initiator_type,
                next_hop,
                resource_timing,
            )),
            global,
            can_gc,
        )
    }

    /// Convert an optional [`CrossProcessInstant`] to a [`DOMHighResTimeStamp`]. If none
    /// return a timestamp for [`Self::fetch_start`] instead, so that timestamps are
    /// always after that time.
    pub(crate) fn to_dom_high_res_time_stamp(
        &self,
        instant: Option<CrossProcessInstant>,
    ) -> DOMHighResTimeStamp {
        self.global()
            .performance()
            .maybe_to_dom_high_res_time_stamp(instant)
    }
}

// https://w3c.github.io/resource-timing/
impl PerformanceResourceTimingMethods<crate::DomTypeHolder> for PerformanceResourceTiming {
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
            Some(ref protocol) => protocol.clone(),
            None => DOMString::from(""),
        }
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupstart
    fn DomainLookupStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.domain_lookup_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupend
    fn DomainLookupEnd(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.domain_lookup_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-secureconnectionstart
    fn SecureConnectionStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.secure_connection_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-transfersize
    fn TransferSize(&self) -> u64 {
        self.transfer_size
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-encodedbodysize
    fn EncodedBodySize(&self) -> u64 {
        self.encoded_body_size
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-decodedbodysize
    fn DecodedBodySize(&self) -> u64 {
        self.decoded_body_size
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-requeststart
    fn RequestStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.request_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectstart
    fn RedirectStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.redirect_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectend
    fn RedirectEnd(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.redirect_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responsestart
    fn ResponseStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.response_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-fetchstart
    fn FetchStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.fetch_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectstart
    fn ConnectStart(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.connect_start)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectend
    fn ConnectEnd(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.connect_end)
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responseend
    fn ResponseEnd(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(self.response_end)
    }
}
