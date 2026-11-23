/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use net_traits::ResourceFetchTiming;
use script_bindings::domstring::TracedDOMString;
use script_bindings::reflector::reflect_dom_object;
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_url::ServoUrl;
use time::Duration;
use xml5ever::LocalName;

use super::performanceentry::{EntryType, PerformanceEntry};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::PerformanceResourceTimingMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
// TODO UA may choose to limit how many resources are included as PerformanceResourceTiming objects
// recommended minimum is 150, can be changed by setResourceTimingBufferSize in performance
// https://w3c.github.io/resource-timing/#sec-extensions-performance-interface

// TODO Cross origin resources MUST BE INCLUDED as PerformanceResourceTiming objects
// https://w3c.github.io/resource-timing/#sec-cross-origin-resources

// TODO CSS
#[derive(Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum InitiatorType {
    Beacon,
    Css,
    LocalName(#[no_trace] LocalName),
    Navigation,
    XMLHttpRequest,
    Fetch,
    Track,
    Other,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
enum TimingSource {
    /// Resource timing entries keep the timing captured when the entry was
    /// created.
    Snapshot(#[no_trace] ResourceFetchTiming),
    /// Navigation timing entries read the timing live from their document.
    Document(Dom<Document>),
}

/// A borrowed timing source, used to construct an owned [`TimingSource`].
pub(super) enum TimingSourceRef<'a> {
    Snapshot(&'a ResourceFetchTiming),
    Document(&'a Document),
}

#[dom_struct]
pub(crate) struct PerformanceResourceTiming {
    entry: PerformanceEntry,
    initiator_type: InitiatorType,
    timing: TimingSource,
    next_hop: Option<TracedDOMString>,
    #[no_trace]
    domain_lookup_end: Option<CrossProcessInstant>,
    transfer_size: u64,     // size in octets
    encoded_body_size: u64, // size in octets
    decoded_body_size: u64, // size in octets
}

// TODO(#21269): next_hop
// TODO(#21260): domain_lookup_end
impl PerformanceResourceTiming {
    pub(super) fn new_inherited(
        url: ServoUrl,
        initiator_type: InitiatorType,
        source: TimingSourceRef<'_>,
    ) -> PerformanceResourceTiming {
        let (entry_type, start_time) = match &source {
            TimingSourceRef::Snapshot(resource_timing) => {
                (EntryType::Resource, resource_timing.start_time)
            },
            TimingSourceRef::Document(_) => (EntryType::Navigation, None),
        };

        PerformanceResourceTiming {
            entry: PerformanceEntry::new_inherited(
                url.into(),
                entry_type,
                start_time,
                Duration::ZERO, // duration is calculated dynamically in PerformanceEntry::Duration
            ),
            initiator_type,
            timing: match source {
                TimingSourceRef::Snapshot(resource_timing) => {
                    TimingSource::Snapshot(resource_timing.clone())
                },
                TimingSourceRef::Document(document) => {
                    TimingSource::Document(Dom::from_ref(document))
                },
            },
            next_hop: None,
            domain_lookup_end: None,
            transfer_size: 0,
            encoded_body_size: 0,
            decoded_body_size: 0,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        url: ServoUrl,
        initiator_type: InitiatorType,
        resource_timing: &ResourceFetchTiming,
    ) -> DomRoot<PerformanceResourceTiming> {
        reflect_dom_object(
            cx,
            Box::new(PerformanceResourceTiming::new_inherited(
                url,
                initiator_type,
                TimingSourceRef::Snapshot(resource_timing),
            )),
            global,
        )
    }

    /// Convert an optional [`CrossProcessInstant`] to a [`DOMHighResTimeStamp`].
    /// `None` is converted to zero.
    pub(crate) fn to_dom_high_res_time_stamp(
        &self,
        cx: &mut JSContext,
        instant: Option<CrossProcessInstant>,
    ) -> DOMHighResTimeStamp {
        self.global()
            .performance(cx)
            .maybe_to_dom_high_res_time_stamp(instant)
    }

    /// Return a resource timing timestamp.
    fn to_dom_high_res_time_stamp_from_timing_source(
        &self,
        cx: &mut JSContext,
        live: impl FnOnce(&ResourceFetchTiming) -> Option<CrossProcessInstant>,
    ) -> DOMHighResTimeStamp {
        let instant = match &self.timing {
            TimingSource::Snapshot(resource_timing) => live(resource_timing),
            TimingSource::Document(document) => {
                document.resource_fetch_timing().as_ref().and_then(live)
            },
        };
        self.to_dom_high_res_time_stamp(cx, instant)
    }
}

// https://w3c.github.io/resource-timing/
impl PerformanceResourceTimingMethods<crate::DomTypeHolder> for PerformanceResourceTiming {
    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-initiatortype>
    fn InitiatorType(&self) -> DOMString {
        match self.initiator_type {
            InitiatorType::Beacon => DOMString::from_static("beacon"),
            InitiatorType::Css => DOMString::from_static("css"),
            InitiatorType::LocalName(ref n) => TracedDOMString::from(n.clone()).root(),
            InitiatorType::Navigation => DOMString::from_static("navigation"),
            InitiatorType::XMLHttpRequest => DOMString::from_static("xmlhttprequest"),
            InitiatorType::Fetch => DOMString::from_static("fetch"),
            InitiatorType::Track => DOMString::from_static("track"),
            InitiatorType::Other => DOMString::from_static("other"),
        }
    }

    // https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-nexthopprotocol
    // returns the ALPN protocol ID of the network protocol used to fetch the resource
    // when a proxy is configured
    fn NextHopProtocol(&self) -> DOMString {
        match self.next_hop {
            Some(ref protocol) => protocol.clone().root(),
            None => DOMString::new(),
        }
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupstart>
    fn DomainLookupStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.domain_lookup_start)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-domainlookupend>
    fn DomainLookupEnd(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(cx, self.domain_lookup_end)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-secureconnectionstart>
    fn SecureConnectionStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| {
            timing.secure_connection_start
        })
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-transfersize>
    fn TransferSize(&self) -> u64 {
        self.transfer_size
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-encodedbodysize>
    fn EncodedBodySize(&self) -> u64 {
        self.encoded_body_size
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-decodedbodysize>
    fn DecodedBodySize(&self) -> u64 {
        self.decoded_body_size
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-requeststart>
    fn RequestStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.request_start)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectstart>
    fn RedirectStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.redirect_start)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-redirectend>
    fn RedirectEnd(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.redirect_end)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responsestart>
    fn ResponseStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.response_start)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-fetchstart>
    fn FetchStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.fetch_start)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectstart>
    fn ConnectStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.connect_start)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-connectend>
    fn ConnectEnd(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.connect_end)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-responseend>
    fn ResponseEnd(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| timing.response_end)
    }

    /// <https://w3c.github.io/resource-timing/#dom-performanceresourcetiming-workerstart>
    /// > The workerStart getter steps are to convert fetch timestamp for this’s timing
    /// > info’s final service worker start time and the relevant global object for this.
    fn WorkerStart(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp_from_timing_source(cx, |timing| {
            timing.final_service_worker_start_time
        })
    }
}
